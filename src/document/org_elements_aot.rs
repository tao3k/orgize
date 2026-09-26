//! Org document facts projected from the Scheme-AOT Element graph.

use std::collections::HashMap;
use std::path::Path;

use gerbil_parser_rowan::GraphRecord;
use rowan::TextRange;

use crate::org_aot::{OrgAotDocument, org_image_link, org_language_spec, parse_org_aot};

use super::{
    line_index::LineIndex,
    model::{DocumentElement, document_structural_selector, selector_component},
};

struct IndexContext<'a> {
    path: &'a Path,
    source: &'a str,
    lines: LineIndex,
    records: &'a [GraphRecord],
    ordinals: Vec<usize>,
}

impl<'a> IndexContext<'a> {
    fn new(path: &'a Path, source: &'a str, records: &'a [GraphRecord]) -> Self {
        let mut sibling_counts = HashMap::new();
        let ordinals = records
            .iter()
            .map(|record| {
                let count = sibling_counts
                    .entry((record.parent_id, record.syntax_kind))
                    .or_insert(0usize);
                *count += 1;
                *count
            })
            .collect();
        Self {
            path,
            source,
            lines: LineIndex::new(source),
            records,
            ordinals,
        }
    }

    fn fact(
        &self,
        kind: &'static str,
        source_kind: &'static str,
        record: &GraphRecord,
        range: TextRange,
        fields: Vec<(String, String)>,
        content: Option<String>,
    ) -> DocumentElement {
        let start = usize::from(range.start());
        let end = usize::from(range.end());
        let raw = self.source.get(start..end).unwrap_or_default();
        let text = if kind == "paragraph" {
            normalize_inline_text(raw.trim())
        } else {
            raw.lines().next().unwrap_or_default().trim().to_string()
        };
        let content = content.unwrap_or_else(|| {
            if matches!(kind, "list" | "listItem" | "checklistItem" | "paragraph") {
                raw.trim().to_string()
            } else {
                text.clone()
            }
        });
        let mut ancestry = Vec::new();
        let mut cursor = Some(record.id);
        while let Some(id) = cursor {
            let ancestor = &self.records[id];
            ancestry.push(format!(
                "{}[{}]",
                selector_component(ancestor.kind),
                self.ordinals[id]
            ));
            cursor = ancestor.parent_id;
        }
        ancestry.reverse();
        let mut selector_parts = vec![
            selector_component(source_kind),
            selector_component(kind),
            ancestry.join("/"),
        ];
        for key in ["key", "target", "title", "tag", "lang"] {
            if let Some((_, value)) = fields
                .iter()
                .find(|(field, value)| field == key && !value.trim().is_empty())
            {
                selector_parts.push(format!("{key}={}", selector_component(value)));
                break;
            }
        }
        DocumentElement {
            kind,
            source_kind,
            path: self.path.display().to_string(),
            structural_selector: document_structural_selector("org", self.path, &selector_parts),
            line: self.lines.line_for(start),
            end_line: self.lines.line_for(end.saturating_sub(1)),
            start_byte: start,
            end_byte: end,
            fields,
            text,
            content,
        }
    }
}

pub(super) fn index_org(path: &Path, source: &str) -> Result<Vec<DocumentElement>, String> {
    let document = parse_org_aot(source).map_err(|error| format!("Org AOT parse: {error:?}"))?;
    let records = document.records();
    let context = IndexContext::new(path, source, records);
    let headline_ranges = document
        .syntax()
        .descendants()
        .filter(|node| org_language_spec().kinds[usize::from(node.kind().0)].name == "OrgHeadline")
        .map(|node| (usize::from(node.text_range().start()), node.text_range()))
        .collect::<HashMap<_, _>>();
    let mut facts = Vec::new();
    for record in records {
        match record.kind {
            "headline" => {
                let range = headline_ranges
                    .get(&usize::from(record.range.start()))
                    .copied()
                    .ok_or_else(|| "AOT section lacks its headline node".to_string())?;
                push_headline(&context, &document, record, range, &mut facts);
            }
            "property-drawer" => {
                for property in record
                    .child_ids
                    .iter()
                    .filter_map(|id| records.get(*id))
                    .filter(|child| child.kind == "node-property")
                {
                    let fields = vec![
                        (
                            "key".to_string(),
                            property.field("key").unwrap_or_default().to_string(),
                        ),
                        (
                            "value".to_string(),
                            property.field("value").unwrap_or_default().to_string(),
                        ),
                    ];
                    facts.push(context.fact(
                        "property",
                        "PropertyDrawer",
                        record,
                        record.range,
                        fields,
                        None,
                    ));
                }
            }
            "planning" => {
                let fields = record
                    .values("key")
                    .map(|key| (key.to_ascii_lowercase(), "true".to_string()))
                    .collect();
                facts.push(context.fact(
                    "planning",
                    "SyntaxPlanning",
                    record,
                    record.range,
                    fields,
                    None,
                ));
            }
            "table" => {
                let rows = record
                    .child_ids
                    .iter()
                    .filter_map(|id| records.get(*id))
                    .map(|child| child.kind)
                    .collect::<Vec<_>>();
                let header = rows
                    .windows(3)
                    .any(|window| window == ["table-row", "table-rule-row", "table-row"]);
                facts.push(context.fact(
                    "table",
                    "OrgTable",
                    record,
                    record.range,
                    vec![("header".to_string(), header.to_string())],
                    None,
                ));
            }
            "paragraph" => facts.push(context.fact(
                "paragraph",
                "Paragraph",
                record,
                record.range,
                Vec::new(),
                None,
            )),
            "src-block" | "export-block" => {
                let source_kind = if record.kind == "src-block" {
                    "SourceBlock"
                } else {
                    "ExportBlock"
                };
                let mut fields = vec![(
                    "kind".to_string(),
                    if record.kind == "src-block" {
                        "source"
                    } else {
                        "export"
                    }
                    .to_string(),
                )];
                let header_field = if record.kind == "src-block" {
                    "language"
                } else {
                    "backend"
                };
                if let Some(value) = record.field(header_field) {
                    fields.push((
                        if record.kind == "src-block" {
                            "lang"
                        } else {
                            "backend"
                        }
                        .to_string(),
                        value.to_string(),
                    ));
                }
                facts.push(context.fact(
                    "block",
                    source_kind,
                    record,
                    record.range,
                    fields,
                    Some(record.field("body").unwrap_or_default().to_string()),
                ));
            }
            "plain-list" => {
                let items = record
                    .child_ids
                    .iter()
                    .filter_map(|id| records.get(*id))
                    .filter(|child| child.kind == "item")
                    .collect::<Vec<_>>();
                let ordered = items
                    .first()
                    .and_then(|item| item.field("bullet"))
                    .is_some_and(|bullet| bullet.starts_with(|ch: char| ch.is_ascii_digit()));
                let descriptive = items.iter().any(|item| item.field("tag").is_some());
                facts.push(context.fact(
                    "list",
                    "SyntaxList",
                    record,
                    record.range,
                    vec![
                        (
                            "listKind".to_string(),
                            if ordered { "ordered" } else { "unordered" }.to_string(),
                        ),
                        ("descriptive".to_string(), descriptive.to_string()),
                    ],
                    None,
                ));
            }
            "item" => {
                let trivia = record.values("trivia").collect::<Vec<_>>();
                let indent = trivia
                    .first()
                    .filter(|value| {
                        context.source[usize::from(record.range.start())..].starts_with(**value)
                    })
                    .map_or(0, |value| value.len());
                let spacing = trivia
                    .get(usize::from(indent > 0))
                    .copied()
                    .unwrap_or_default();
                let mut fields = vec![
                    (
                        "bullet".to_string(),
                        format!("{}{}", record.field("bullet").unwrap_or_default(), spacing),
                    ),
                    ("indent".to_string(), indent.to_string()),
                ];
                if let Some(counter) = record.field("counter") {
                    fields.push(("counter".to_string(), counter.to_string()));
                }
                let checkbox = record.field("checkbox");
                if let Some(value) = checkbox {
                    fields.push(("checkbox".to_string(), value.to_string()));
                    fields.push(("checked".to_string(), (value == "X").to_string()));
                }
                if let Some(tag) = record.field("tag") {
                    fields.push(("tag".to_string(), tag.trim_end().to_string()));
                }
                facts.push(context.fact(
                    if checkbox.is_some() {
                        "checklistItem"
                    } else {
                        "listItem"
                    },
                    "SyntaxListItem",
                    record,
                    record.range,
                    fields,
                    None,
                ));
            }
            "link" => {
                let target = record.field("path").unwrap_or_default();
                let description = record.field("description").unwrap_or_default();
                let mut fields = vec![("target".to_string(), target.to_string())];
                if !description.is_empty() {
                    fields.push(("description".to_string(), description.to_string()));
                }
                let image = description.is_empty() && org_image_link(target);
                facts.push(context.fact(
                    if image { "image" } else { "link" },
                    "SyntaxLink",
                    record,
                    record.range,
                    fields,
                    None,
                ));
            }
            _ => {}
        }
    }
    Ok(facts)
}

fn push_headline(
    context: &IndexContext<'_>,
    document: &OrgAotDocument,
    record: &GraphRecord,
    range: TextRange,
    facts: &mut Vec<DocumentElement>,
) {
    let mut fields = vec![
        (
            "level".to_string(),
            record
                .field("markers")
                .unwrap_or_default()
                .len()
                .to_string(),
        ),
        (
            "title".to_string(),
            document
                .headline_display_title(record.id)
                .unwrap_or_default(),
        ),
    ];
    if let Some(todo) = document.headline_todo_keyword(record.id) {
        fields.push(("todo".to_string(), todo));
    }
    if let Some(todo_type) = document.headline_todo_type(record.id) {
        fields.push((
            "todoType".to_string(),
            if todo_type == "done" { "Done" } else { "Todo" }.to_string(),
        ));
    }
    facts.push(context.fact("heading", "Headline", record, range, fields.clone(), None));
    if document.headline_todo_keyword(record.id).is_some() {
        facts.push(context.fact("task", "Headline", record, range, fields, None));
    }
}

fn normalize_inline_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
