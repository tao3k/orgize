//! Typed, owned values projected from the Scheme-generated Org Element graph.
//!
//! This adapter consumes parser-owned records. It does not recognize Org text
//! or invoke the displaced handwritten Rust parser.

use std::collections::HashSet;

use gerbil_parser_rowan::GraphRecord;
use rowan::TextRange;

use crate::org_aot::OrgAotDocument;

use super::model::{
    Block, BlockKind, Diagnostic, DiagnosticKind, Document, Element, ElementData, Keyword,
    MarkupKind, Object, ObjectData, ParsedAnnotation, ParsedAst, Planning, Property, Section,
    Table, TableCell, TableRow, TodoKeyword, TodoState, UnsupportedSyntaxKind,
};
use super::preprocessing::macro_definition;
use super::prescan::{SemanticPrescan, collect_document_keyword};
use super::source_position::LineIndex;

impl OrgAotDocument {
    /// Project the Scheme-owned Element graph into the owned semantic API.
    #[must_use]
    pub fn document(&self) -> ParsedAst {
        let source = self.to_org();
        GraphProjector::new(self, &source).document()
    }
}

struct GraphProjector<'a> {
    document: &'a OrgAotDocument,
    source: &'a str,
    lines: LineIndex<'a>,
    attached_keyword_ids: HashSet<usize>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> GraphProjector<'a> {
    fn new(document: &'a OrgAotDocument, source: &'a str) -> Self {
        let attached_keyword_ids = document
            .records()
            .iter()
            .flat_map(|record| document.affiliated_keyword_ids(record.id).iter().copied())
            .collect();
        Self {
            document,
            source,
            lines: LineIndex::new(source),
            attached_keyword_ids,
            diagnostics: Vec::new(),
        }
    }

    fn record(&self, id: usize) -> &GraphRecord {
        &self.document.records()[id]
    }

    fn raw(&self, range: TextRange) -> &str {
        self.source
            .get(usize::from(range.start())..usize::from(range.end()))
            .expect("AOT graph range is source-bound")
    }

    fn annotation(&self, range: TextRange) -> ParsedAnnotation {
        ParsedAnnotation {
            range,
            start: self.lines.position(range.start()),
            end: self.lines.position(range.end()),
            raw: self.raw(range).to_owned(),
        }
    }

    fn document(mut self) -> ParsedAst {
        let root = self.document.records().first().expect("AOT document root");
        let ann = self.annotation(root.range);
        let mut children = Vec::new();
        let mut sections = Vec::new();
        for &id in &root.child_ids {
            if self.attached_keyword_ids.contains(&id) {
                continue;
            }
            if self.record(id).kind == "headline" {
                sections.push(self.section(id));
            } else if let Some(element) = self.element(id) {
                children.push(element);
            }
        }

        let mut prescan = SemanticPrescan::default();
        for record in self
            .document
            .records()
            .iter()
            .filter(|record| record.kind == "keyword" && !self.within_headline(record.id))
        {
            let Some(keyword) = self.keyword(record) else {
                continue;
            };
            if keyword.key.eq_ignore_ascii_case("MACRO") {
                match macro_definition(keyword) {
                    Ok(definition) => prescan.macro_definitions.push(definition),
                    Err((range, message)) => prescan.diagnostics.push(Diagnostic {
                        range,
                        kind: DiagnosticKind::Conversion,
                        message,
                    }),
                }
            } else {
                collect_document_keyword(keyword, &mut prescan);
            }
        }
        let properties = self
            .document
            .records()
            .iter()
            .filter(|record| record.kind == "node-property" && !self.within_headline(record.id))
            .filter_map(|record| self.property(record))
            .chain(prescan.properties)
            .collect();
        self.diagnostics.extend(prescan.diagnostics);
        Document {
            ann,
            properties,
            archive_locations: prescan.archive_locations,
            metadata: prescan.metadata,
            filetags: prescan.filetags,
            tag_definitions: prescan.tag_definitions,
            export_settings: prescan.export_settings,
            link_abbreviations: prescan.link_abbreviations,
            includes: prescan.includes,
            macro_definitions: prescan.macro_definitions,
            targets: Vec::new(),
            footnotes: Vec::new(),
            children,
            sections,
            diagnostics: self.diagnostics,
        }
    }

    fn within_headline(&self, id: usize) -> bool {
        self.nearest_headline(id).is_some()
    }

    fn nearest_headline(&self, id: usize) -> Option<usize> {
        let mut parent = self.record(id).parent_id;
        while let Some(ancestor) = parent {
            if self.record(ancestor).kind == "headline" {
                return Some(ancestor);
            }
            parent = self.record(ancestor).parent_id;
        }
        None
    }

    fn section(&mut self, id: usize) -> Section<ParsedAnnotation> {
        let record = self.record(id);
        let range = record.range;
        let child_ids = record.child_ids.clone();
        let level = record.field("markers").map_or(1, str::len);
        let raw_title = record.field("title").unwrap_or_default().to_owned();
        let tags = record.values("tag").map(str::to_owned).collect::<Vec<_>>();
        let todo = self
            .document
            .headline_todo_keyword(id)
            .map(|name| TodoKeyword {
                state: if self.document.headline_todo_type(id) == Some("done") {
                    TodoState::Done
                } else {
                    TodoState::Todo
                },
                name,
            });
        let is_comment = self.document.headline_is_comment(id).unwrap_or(false);
        let title = self
            .document
            .headline_display_title(id)
            .map(|text| vec![self.plain(range, &text)])
            .unwrap_or_default();
        let mut children = Vec::new();
        let mut subsections = Vec::new();
        for child in child_ids {
            if self.attached_keyword_ids.contains(&child) {
                continue;
            }
            if self.record(child).kind == "headline" {
                subsections.push(self.section(child));
            } else if let Some(element) = self.element(child) {
                children.push(element);
            }
        }
        let properties = self
            .document
            .records()
            .iter()
            .filter(|property| {
                property.kind == "node-property" && self.nearest_headline(property.id) == Some(id)
            })
            .filter_map(|property| self.property(property))
            .collect::<Vec<_>>();
        Section {
            ann: self.annotation(range),
            body_ann: None,
            level,
            properties: properties.clone(),
            effective_properties: properties,
            archive: Default::default(),
            attachment: Default::default(),
            todo,
            is_comment,
            priority: Default::default(),
            title,
            raw_title,
            anchor: None,
            tags: tags.clone(),
            effective_tags: tags,
            planning: Planning::default(),
            children,
            subsections,
        }
    }

    fn keyword(&self, record: &GraphRecord) -> Option<Keyword<ParsedAnnotation>> {
        Some(Keyword {
            ann: self.annotation(record.range),
            key: record.field("key")?.to_owned(),
            optional: None,
            value: record.field("value")?.to_owned(),
            parsed: Vec::new(),
            attributes: Vec::new(),
        })
    }

    fn property(&self, record: &GraphRecord) -> Option<Property<ParsedAnnotation>> {
        Some(Property {
            ann: self.annotation(record.range),
            key: record.field("key")?.to_owned(),
            value: record.field("value")?.to_owned(),
            duration: None,
        })
    }

    fn element(&mut self, id: usize) -> Option<Element<ParsedAnnotation>> {
        let record = self.record(id);
        if record.category != "element" {
            return None;
        }
        let range = record.range;
        let kind = record.kind;
        let affiliated_keywords = self
            .document
            .affiliated_keyword_ids(id)
            .iter()
            .filter_map(|&keyword_id| self.keyword(self.record(keyword_id)))
            .collect();
        let data = match kind {
            "keyword" => ElementData::Keyword(self.keyword(record)?),
            "paragraph" => ElementData::Paragraph(self.paragraph_objects(id)),
            "table" => ElementData::Table(self.table(id)),
            "property-drawer" => ElementData::PropertyDrawer(
                record
                    .child_ids
                    .iter()
                    .filter_map(|&child| self.property(self.record(child)))
                    .collect(),
            ),
            "src-block" | "example-block" | "export-block" | "quote-block" | "verse-block"
            | "center-block" | "comment-block" | "dynamic-block" | "special-block" => {
                ElementData::Block(self.block(id))
            }
            "comment" => ElementData::Comment(self.raw(range).to_owned()),
            "diary-sexp" => ElementData::DiarySexp(self.raw(range).to_owned()),
            "horizontal-rule" => ElementData::Rule,
            "latex-environment" => ElementData::LatexEnvironment(self.raw(range).to_owned()),
            _ => {
                self.unsupported(range, kind, DiagnosticKind::UnsupportedElement);
                ElementData::Unknown {
                    kind: UnsupportedSyntaxKind::new(kind),
                    raw: self.raw(range).to_owned(),
                }
            }
        };
        Some(Element {
            ann: self.annotation(range),
            affiliated_keywords,
            data,
        })
    }

    fn block(&mut self, id: usize) -> Block<ParsedAnnotation> {
        let record = self.record(id);
        let kind = match record.kind {
            "src-block" => BlockKind::Source,
            "example-block" => BlockKind::Example,
            "export-block" => BlockKind::Export,
            "quote-block" => BlockKind::Quote,
            "verse-block" => BlockKind::Verse,
            "center-block" => BlockKind::Center,
            "comment-block" => BlockKind::Comment,
            "dynamic-block" => BlockKind::Dynamic,
            _ => BlockKind::Special(record.field("name").unwrap_or_default().to_owned()),
        };
        let children = record.child_ids.clone();
        let name = record.field("name").map(str::to_owned);
        let language = record.field("language").map(str::to_owned);
        let parameters = record.field("header").map(str::to_owned);
        let value = record.field("body").unwrap_or_default().to_owned();
        Block {
            kind,
            name,
            language,
            switches: None,
            switch_options: Default::default(),
            line_numbering: None,
            preserve_indentation: false,
            lines: Vec::new(),
            code_refs: Vec::new(),
            parameters,
            header_args: Vec::new(),
            value,
            children: children
                .into_iter()
                .filter_map(|child| self.element(child))
                .collect(),
        }
    }

    fn table(&self, id: usize) -> Table<ParsedAnnotation> {
        let rows = self
            .record(id)
            .child_ids
            .iter()
            .filter_map(|&row_id| {
                let row = self.record(row_id);
                if !matches!(row.kind, "table-row" | "table-rule-row") {
                    return None;
                }
                let cells =
                    row.child_ids
                        .iter()
                        .filter_map(|&cell_id| {
                            let cell = self.record(cell_id);
                            (cell.kind == "table-cell").then(|| TableCell {
                                ann: self.annotation(cell.range),
                                objects: vec![self.plain(
                                    cell.range,
                                    cell.field("text").unwrap_or_default().trim(),
                                )],
                            })
                        })
                        .collect();
                Some(TableRow {
                    ann: self.annotation(row.range),
                    is_rule: row.kind == "table-rule-row",
                    cells,
                })
            })
            .collect();
        Table {
            rows,
            column_alignments: Vec::new(),
            formulas: Vec::new(),
            parsed_formulas: Vec::new(),
        }
    }

    fn paragraph_objects(&mut self, id: usize) -> Vec<Object<ParsedAnnotation>> {
        let record = self.record(id);
        let mut cursor = usize::from(record.range.start());
        let end = usize::from(record.range.end());
        let children = record.child_ids.clone();
        let mut objects = Vec::new();
        for child in children {
            let range = self.record(child).range;
            let start = usize::from(range.start());
            if cursor < start {
                objects.push(self.plain_bytes(cursor, start));
            }
            if let Some(object) = self.object(child) {
                objects.push(object);
            }
            cursor = usize::from(range.end());
        }
        if cursor < end {
            objects.push(self.plain_bytes(cursor, end));
        }
        objects
    }

    fn plain_bytes(&self, start: usize, end: usize) -> Object<ParsedAnnotation> {
        let range = TextRange::new((start as u32).into(), (end as u32).into());
        self.plain(range, self.raw(range))
    }

    fn plain(&self, range: TextRange, value: &str) -> Object<ParsedAnnotation> {
        Object {
            ann: self.annotation(range),
            data: ObjectData::Plain(value.to_owned()),
        }
    }

    fn object(&mut self, id: usize) -> Option<Object<ParsedAnnotation>> {
        let record = self.record(id);
        if record.category != "object" {
            return None;
        }
        let range = record.range;
        let kind = record.kind;
        let data = match kind {
            "code" => ObjectData::Code(record.field("value").unwrap_or_default().to_owned()),
            "verbatim" => {
                ObjectData::Verbatim(record.field("value").unwrap_or_default().to_owned())
            }
            "entity" => ObjectData::Entity(self.raw(range).to_owned()),
            "latex-fragment" => ObjectData::LatexFragment(self.raw(range).to_owned()),
            "target" => ObjectData::Target(record.field("value").unwrap_or_default().to_owned()),
            "radio-target" => {
                ObjectData::RadioTarget(record.field("value").unwrap_or_default().to_owned())
            }
            "statistics-cookie" => ObjectData::StatisticCookie(self.raw(range).to_owned()),
            "line-break" => ObjectData::LineBreak,
            "export-snippet" => ObjectData::ExportSnippet {
                backend: record.field("backend").unwrap_or_default().to_owned(),
                value: record.field("value").unwrap_or_default().to_owned(),
            },
            "bold" | "italic" | "underline" | "strike-through" | "subscript" | "superscript" => {
                let markup = match kind {
                    "bold" => MarkupKind::Bold,
                    "italic" => MarkupKind::Italic,
                    "underline" => MarkupKind::Underline,
                    "strike-through" => MarkupKind::Strike,
                    "subscript" => MarkupKind::Subscript,
                    _ => MarkupKind::Superscript,
                };
                ObjectData::Markup {
                    kind: markup,
                    children: vec![self.plain(range, record.field("value").unwrap_or_default())],
                }
            }
            _ => {
                self.unsupported(range, kind, DiagnosticKind::UnsupportedObject);
                ObjectData::Unknown {
                    kind: UnsupportedSyntaxKind::new(kind),
                    raw: self.raw(range).to_owned(),
                }
            }
        };
        Some(Object {
            ann: self.annotation(range),
            data,
        })
    }

    fn unsupported(&mut self, range: TextRange, kind: &str, category: DiagnosticKind) {
        self.diagnostics.push(Diagnostic {
            range,
            kind: category,
            message: format!("Scheme AOT graph node {kind} is not yet projected to the owned AST"),
        });
    }
}
