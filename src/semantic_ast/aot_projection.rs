//! Typed, owned values projected from the Scheme-generated Org Element graph.
//!
//! This adapter consumes parser-owned records. It does not recognize Org text
//! or invoke the displaced handwritten Rust parser.

use std::collections::HashSet;

use gerbil_parser_rowan::GraphRecord;
use rowan::TextRange;

use crate::org_aot::{OrgAotDocument, org_image_link};

use super::block_metadata::{
    BlockLineOptions, parse_block_header_args, parse_block_lines, split_block_lines,
};
use super::block_model::{BlockSwitches, SemanticFixedWidth};
use super::link_model::{LinkDescriptionState, LinkMediaKind, LinkPath, LinkTarget};
use super::model::{
    Block, BlockKind, Checkbox, Citation, CiteReference, Diagnostic, DiagnosticKind, Document,
    Drawer, Element, ElementData, FootnoteDef, Inlinetask, InlinetaskEnd, Keyword, Link, List,
    ListItem, ListType, MarkupKind, Object, ObjectData, ParsedAnnotation, ParsedAst, Planning,
    Property, Section, Table, TableCell, TableRow, TodoKeyword, TodoState, UnsupportedSyntaxKind,
};
use super::preprocessing::{macro_definition, split_macro_args};
use super::prescan::{SemanticPrescan, collect_document_keyword};
use super::property_model::Priority;
use super::source_position::LineIndex;
use super::timestamp_model::{Timestamp, TimestampKind};

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
            if matches!(self.record(ancestor).kind, "headline" | "inlinetask") {
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
        let raw_title = self.document.headline_display_title(id).unwrap_or_default();
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
        let title = vec![self.plain(range, &raw_title)];
        let planning = child_ids
            .iter()
            .copied()
            .find(|&child| self.record(child).kind == "planning")
            .map(|child| self.planning(child))
            .unwrap_or_default();
        let mut children = Vec::new();
        let mut subsections = Vec::new();
        for child in child_ids {
            if self.attached_keyword_ids.contains(&child) {
                continue;
            }
            if matches!(self.record(child).kind, "planning" | "property-drawer") {
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
            priority: Priority::from_cookie(self.document.headline_priority_cookie(id)),
            title,
            raw_title,
            anchor: None,
            tags: tags.clone(),
            effective_tags: tags,
            planning,
            children,
            subsections,
        }
    }

    fn keyword(&self, record: &GraphRecord) -> Option<Keyword<ParsedAnnotation>> {
        Some(Keyword {
            ann: self.annotation(record.range),
            key: record.field("key")?.to_owned(),
            optional: None,
            value: record.field("raw-value")?.to_owned(),
            parsed: Vec::new(),
            attributes: Vec::new(),
        })
    }

    fn inlinetask(&mut self, id: usize) -> Inlinetask<ParsedAnnotation> {
        let (range, child_ids, title_body) = {
            let record = self.record(id);
            (
                record.range,
                record.child_ids.clone(),
                record.field("title-body").map(str::to_owned),
            )
        };
        let raw_title = self.document.headline_source_title(id).unwrap_or_default();
        let title = title_body
            .as_deref()
            .filter(|body| !body.is_empty() && body.ends_with(&raw_title))
            .and_then(|body| {
                let source_start = usize::from(range.start());
                let header = self.raw(range).split_once('\n')?.0;
                let title_start = source_start + header.find(body)? + body.len() - raw_title.len();
                let title_end = title_start + raw_title.len();
                Some(self.objects_in_span(
                    TextRange::new((title_start as u32).into(), (title_end as u32).into()),
                    &child_ids,
                ))
            })
            .unwrap_or_else(|| vec![self.plain(range, &raw_title)]);
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
        let properties = child_ids
            .iter()
            .filter_map(|&child| (self.record(child).kind == "property-drawer").then_some(child))
            .flat_map(|drawer| self.record(drawer).child_ids.iter())
            .filter_map(|&property| self.property(self.record(property)))
            .collect();
        let end = child_ids
            .iter()
            .map(|&child| self.record(child))
            .find(|child| child.kind == "inlinetask-end")
            .map(|end| InlinetaskEnd {
                ann: self.annotation(end.range),
                level: end.field("markers").map_or(0, str::len),
                raw: self.raw(end.range).to_owned(),
            });
        let planning = child_ids
            .iter()
            .copied()
            .find(|&child| self.record(child).kind == "planning")
            .map(|child| self.planning(child))
            .unwrap_or_default();
        let mut children = Vec::new();
        for child in child_ids {
            if !matches!(
                self.record(child).kind,
                "planning" | "property-drawer" | "inlinetask-end"
            ) && let Some(element) = self.element(child)
            {
                children.push(element);
            }
        }
        Inlinetask {
            level: self.record(id).field("markers").map_or(0, str::len),
            todo,
            priority: Priority::from_cookie(self.document.headline_priority_cookie(id)),
            title,
            raw_title,
            tags: self.record(id).values("tag").map(str::to_owned).collect(),
            planning,
            properties,
            children,
            end,
        }
    }

    fn property(&self, record: &GraphRecord) -> Option<Property<ParsedAnnotation>> {
        Some(Property {
            ann: self.annotation(record.range),
            key: record.field("key")?.to_owned(),
            value: record.field("value")?.to_owned(),
            duration: None,
        })
    }

    fn planning(&self, id: usize) -> Planning {
        let record = self.record(id);
        let mut planning = Planning::default();
        for (key, value) in record.values("key").zip(record.values("value")) {
            let kind = match OrgAotDocument::planning_timestamp_kind(value) {
                "active" => TimestampKind::Active,
                "inactive" => TimestampKind::Inactive,
                _ => continue,
            };
            let timestamp = Timestamp {
                kind,
                raw: value.to_owned(),
                is_range: false,
                start: None,
                end: None,
                repeater: None,
                warning: None,
            };
            match OrgAotDocument::planning_key_kind(key) {
                "scheduled" => planning.scheduled = Some(timestamp),
                "deadline" => planning.deadline = Some(timestamp),
                "closed" => planning.closed = Some(timestamp),
                _ => {}
            }
        }
        planning
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
            "babel-call" => ElementData::BabelCall(self.keyword(record)?),
            "inlinetask" => ElementData::Inlinetask(Box::new(self.inlinetask(id))),
            "paragraph" => ElementData::Paragraph(self.paragraph_objects(id)),
            "plain-list" => ElementData::List(self.list(id)),
            "table" => ElementData::Table(self.table(id)),
            "property-drawer" => ElementData::PropertyDrawer(
                record
                    .child_ids
                    .iter()
                    .filter_map(|&child| self.property(self.record(child)))
                    .collect(),
            ),
            "drawer" => ElementData::Drawer(Drawer {
                name: record.field("name").unwrap_or_default().to_owned(),
                children: record
                    .child_ids
                    .clone()
                    .into_iter()
                    .filter_map(|child| self.element(child))
                    .collect(),
                raw: self.raw(range).to_owned(),
            }),
            "footnote-definition" => ElementData::FootnoteDef(FootnoteDef {
                label: record.field("label").unwrap_or_default().to_owned(),
                children: record
                    .child_ids
                    .clone()
                    .into_iter()
                    .filter_map(|child| self.element(child))
                    .collect(),
            }),
            "src-block" | "example-block" | "export-block" | "quote-block" | "verse-block"
            | "center-block" | "comment-block" | "dynamic-block" | "special-block" => {
                ElementData::Block(self.block(id))
            }
            "comment" => ElementData::Comment(self.raw(range).to_owned()),
            "diary-sexp" => ElementData::DiarySexp(self.raw(range).to_owned()),
            "fixed-width" => ElementData::FixedWidth(self.fixed_width(range)),
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
        let header_args = parse_block_header_args(parameters.as_deref());
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
            header_args,
            value,
            children: children
                .into_iter()
                .filter_map(|child| self.element(child))
                .collect(),
        }
    }

    fn fixed_width(&self, range: TextRange) -> SemanticFixedWidth<ParsedAnnotation> {
        let source = self.raw(range);
        let source_lines = split_block_lines(source);
        let mut value = String::with_capacity(source.len());
        for line in &source_lines {
            let content = line
                .text
                .split_once(':')
                .map_or(line.text, |(_, text)| text);
            let content = content.strip_prefix(' ').unwrap_or(content);
            value.push_str(content);
            if let Some(ending) = line.ending {
                value.push_str(ending);
            }
        }
        let switches = BlockSwitches::default();
        let lines = parse_block_lines(
            &value,
            Some(source),
            BlockLineOptions {
                switches: &switches,
                tab_width: self.document.config().src_tab_width,
                preserve_indentation: false,
            },
            |index| {
                let line = &source_lines[index];
                let start = usize::from(range.start()) + line.start;
                let end = usize::from(range.start()) + line.end;
                self.annotation(TextRange::new((start as u32).into(), (end as u32).into()))
            },
        );
        SemanticFixedWidth { value, lines }
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

    fn list(&mut self, id: usize) -> List<ParsedAnnotation> {
        let item_ids = self.record(id).child_ids.clone();
        let descriptive = item_ids
            .iter()
            .any(|&item| self.record(item).field("tag").is_some());
        let ordered = item_ids
            .first()
            .and_then(|&item| self.record(item).field("bullet"))
            .and_then(|bullet| bullet.chars().next())
            .is_some_and(char::is_alphanumeric);
        let list_type = if descriptive {
            ListType::Descriptive
        } else if ordered {
            ListType::Ordered
        } else {
            ListType::Unordered
        };
        let mut items = Vec::with_capacity(item_ids.len());
        for item in item_ids {
            if self.record(item).kind == "item" {
                items.push(self.list_item(item));
            }
        }
        List { list_type, items }
    }

    fn list_item(&mut self, id: usize) -> ListItem<ParsedAnnotation> {
        let record = self.record(id);
        let range = record.range;
        let bullet = record.field("bullet").unwrap_or_default().to_owned();
        let counter = record.field("counter").map(str::to_owned);
        let checkbox = match record.field("checkbox") {
            Some("X" | "x") => Some(Checkbox::On),
            Some(" ") => Some(Checkbox::Off),
            Some("-") => Some(Checkbox::Trans),
            _ => None,
        };
        let tag = record
            .field("tag")
            .map(|value| vec![self.plain(range, value)])
            .unwrap_or_default();
        let child_ids = record.child_ids.clone();
        ListItem {
            ann: self.annotation(range),
            bullet,
            counter,
            checkbox,
            tag,
            children: child_ids
                .into_iter()
                .filter_map(|child| self.element(child))
                .collect(),
        }
    }

    fn paragraph_objects(&mut self, id: usize) -> Vec<Object<ParsedAnnotation>> {
        let record = self.record(id);
        self.objects_in_span(record.range, &record.child_ids.clone())
    }

    fn objects_in_span(
        &mut self,
        span: TextRange,
        children: &[usize],
    ) -> Vec<Object<ParsedAnnotation>> {
        let mut cursor = usize::from(span.start());
        let end = usize::from(span.end());
        let mut objects = Vec::new();
        for &child in children {
            let range = self.record(child).range;
            let start = usize::from(range.start());
            if start < cursor || usize::from(range.end()) > end {
                continue;
            }
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
            "link" => self.link(id),
            "timestamp" => ObjectData::Timestamp(self.timestamp(id)),
            "latex-fragment" => ObjectData::LatexFragment(self.raw(range).to_owned()),
            "target" => ObjectData::Target(record.field("value").unwrap_or_default().to_owned()),
            "radio-target" => {
                ObjectData::RadioTarget(record.field("value").unwrap_or_default().to_owned())
            }
            "macro" => ObjectData::Macro {
                name: record.field("name").unwrap_or_default().to_owned(),
                arguments: record
                    .field("arguments")
                    .map(split_macro_args)
                    .unwrap_or_default(),
            },
            "statistics-cookie" => ObjectData::StatisticCookie(self.raw(range).to_owned()),
            "line-break" => ObjectData::LineBreak,
            "inline-src-block" => ObjectData::InlineSrc {
                language: record.field("language").unwrap_or_default().to_owned(),
                parameters: record.field("parameters").map(str::to_owned),
                value: record.field("value").unwrap_or_default().to_owned(),
                raw: self.raw(range).to_owned(),
            },
            "inline-babel-call" => ObjectData::InlineCall {
                name: record.field("call").unwrap_or_default().to_owned(),
                arguments: record.field("arguments").unwrap_or_default().to_owned(),
                header: record.field("inside-header").map(str::to_owned),
                end_header: record.field("end-header").map(str::to_owned),
                raw: self.raw(range).to_owned(),
            },
            "footnote-reference" => ObjectData::FootnoteRef {
                label: record.field("label").map(str::to_owned),
                resolved_label: None,
                definition: record
                    .field("definition")
                    .map(|text| vec![self.plain(range, text)])
                    .unwrap_or_default(),
            },
            "citation" => ObjectData::Citation(self.citation(id)),
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

    fn link(&mut self, id: usize) -> ObjectData<ParsedAnnotation> {
        let record = self.record(id);
        let path = record.field("path").unwrap_or_default().to_owned();
        let description = record.field("description").map(str::to_owned);
        let child_ids = record.child_ids.clone();
        let range = record.range;
        let description_objects = if let Some(ref text) = description {
            let end = usize::from(range.end()).saturating_sub(2);
            let start = end.saturating_sub(text.len());
            self.objects_in_span(
                TextRange::new((start as u32).into(), (end as u32).into()),
                &child_ids,
            )
        } else {
            Vec::new()
        };
        let media_kind = if org_image_link(path.trim_start_matches("file:")) {
            LinkMediaKind::Image
        } else {
            LinkMediaKind::Normal
        };
        ObjectData::Link(Box::new(Link {
            path: LinkPath::new(path.clone()),
            target: LinkTarget::Unresolved(path.clone()),
            description: description_objects,
            default_description: vec![self.plain(range, &path)],
            raw_description: description.clone().unwrap_or_default(),
            description_state: if description.is_some() {
                LinkDescriptionState::Explicit
            } else {
                LinkDescriptionState::None
            },
            media_kind,
            caption: None,
            search: None,
            attachment: None,
            file: None,
        }))
    }

    fn citation(&mut self, id: usize) -> Citation<ParsedAnnotation> {
        let (range, head, prefix, suffix, children) = {
            let record = self.record(id);
            (
                record.range,
                record.field("head").unwrap_or_default().to_owned(),
                record.field("global-prefix").unwrap_or_default().to_owned(),
                record.field("global-suffix").unwrap_or_default().to_owned(),
                record.child_ids.clone(),
            )
        };
        let range_start = usize::from(range.start());
        let range_end = usize::from(range.end());
        let prefix_start = range_start + head.len().saturating_sub(1);
        let suffix_start = range_end.saturating_sub(1 + suffix.len());
        let prefix_objects = self.citation_affix(prefix_start, &prefix, &children);
        let suffix_objects = self.citation_affix(suffix_start, &suffix, &children);
        let mut references = Vec::new();
        for child in children {
            let (ref_range, key, ref_prefix, ref_suffix, ref_children) = {
                let record = self.record(child);
                if record.kind != "citation-reference" {
                    continue;
                }
                (
                    record.range,
                    record.field("key").unwrap_or_default().to_owned(),
                    record.field("prefix").unwrap_or_default().to_owned(),
                    record.field("suffix").unwrap_or_default().to_owned(),
                    record.child_ids.clone(),
                )
            };
            let ref_start = usize::from(ref_range.start());
            let ref_end = usize::from(ref_range.end());
            references.push(CiteReference {
                ann: self.annotation(ref_range),
                id: key,
                prefix: self.citation_affix(ref_start, &ref_prefix, &ref_children),
                suffix: self.citation_affix(
                    ref_end.saturating_sub(ref_suffix.len()),
                    &ref_suffix,
                    &ref_children,
                ),
            });
        }
        Citation {
            style: OrgAotDocument::citation_style(&head),
            variant: OrgAotDocument::citation_variant(&head),
            prefix: prefix_objects,
            suffix: suffix_objects,
            references,
        }
    }

    fn citation_affix(
        &mut self,
        start: usize,
        text: &str,
        children: &[usize],
    ) -> Vec<Object<ParsedAnnotation>> {
        let trimmed = text.trim_start_matches([' ', '\t']);
        let start = start + text.len() - trimmed.len();
        let text = trimmed;
        if text.is_empty() {
            return Vec::new();
        }
        let end = start + text.len();
        let Some(source) = self.source.get(start..end) else {
            return Vec::new();
        };
        if source != text {
            return Vec::new();
        }
        self.objects_in_span(
            TextRange::new((start as u32).into(), (end as u32).into()),
            children,
        )
    }

    fn timestamp(&self, id: usize) -> Timestamp {
        let record = self.record(id);
        let kind = if record.field("diary-expression").is_some() {
            TimestampKind::Diary
        } else if record.values("delimiter").next() == Some("[") {
            TimestampKind::Inactive
        } else {
            TimestampKind::Active
        };
        Timestamp {
            kind,
            raw: self.raw(record.range).to_owned(),
            is_range: record.values("range-separator").next().is_some(),
            start: None,
            end: None,
            repeater: None,
            warning: None,
        }
    }

    fn unsupported(&mut self, range: TextRange, kind: &str, category: DiagnosticKind) {
        self.diagnostics.push(Diagnostic {
            range,
            kind: category,
            message: format!("Scheme AOT graph node {kind} is not yet projected to the owned AST"),
        });
    }
}
