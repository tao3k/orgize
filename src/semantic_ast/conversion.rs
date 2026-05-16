//! Conversion from the lossless syntax tree into the semantic AST.

use rowan::ast::AstNode;
use rowan::{NodeOrToken, TextRange};

use crate::{
    config::{ParseConfig, RadioLinkProjection},
    syntax::{
        combinator::node,
        object::{minimal_object_nodes, standard_object_nodes},
        SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken,
    },
    syntax_ast,
};

use super::attachment_model::attachment_link_from_path;
use super::block_metadata::{
    block_code_refs, parse_block_header_args, parse_block_lines, parse_block_switches,
    split_block_lines, BlockLineOptions,
};
use super::block_syntax::{
    block_name_from_begin, block_parts, block_switches_from_begin, semantic_block_name,
};
use super::citation_metadata::{citation_key_range, citation_style};
use super::conversion_util::{
    offset_range, range_from_elements, strip_pair, strip_wrapping, text_range, trimmed_range,
};
use super::footnote_parts::{FootnoteDefParts, FootnoteRefParts};
use super::headline_metadata::{
    headline_level, headline_priority, headline_raw_title, headline_tags, headline_todo,
};
use super::postprocess::finalize_document;
use super::preprocessing::{include_directive, macro_definition, split_macro_args};
use super::prescan::{collect_document_keyword, SemanticPrescan};
use super::radio_links::{is_semantic_radio_link_candidate, next_char_boundary, next_radio_link};
use super::settings::{
    expand_link_abbreviation, file_link, is_parsed_keyword, keyword_attributes, link_search,
};
use super::source_position::LineIndex;
use super::table_metadata::{parsed_table_formulas, table_column_alignments};
use super::targets::{
    collect_target_node, is_strict_internal_link_path, TargetIndex, TargetLookup,
};
use super::timestamp_metadata::{timestamp_moment_range, timestamp_repeater, timestamp_warning};
use super::{
    Block, BlockKind, BlockSwitches, Checkbox, Citation, CiteReference, Clock, Diagnostic,
    DiagnosticKind, Document, Drawer, Element, ElementData, FootnoteDef, Inlinetask, InlinetaskEnd,
    Keyword, Link, LinkAbbreviation, LinkDescriptionState, LinkMediaKind, LinkPath, LinkTarget,
    List, ListItem, ListType, MarkupKind, Object, ObjectData, OrgDuration, ParsedAnnotation,
    ParsedAst, Planning, Priority, Property, Section, SemanticFixedWidth, Table, TableCell,
    TableRow, Timestamp, TimestampKind, TodoKeyword, TodoState, UnsupportedSyntaxKind,
};

impl ParsedAst {
    pub fn from_syntax_tree(root: &SyntaxNode, source: &str) -> Self {
        let config = ParseConfig::default();
        Self::from_syntax_tree_with_config(root, source, &config)
    }

    pub fn from_syntax_tree_with_config(
        root: &SyntaxNode,
        source: &str,
        config: &ParseConfig,
    ) -> Self {
        Converter::new(source, config).document(root)
    }
}

struct Converter<'a> {
    source: &'a str,
    config: &'a ParseConfig,
    lines: LineIndex<'a>,
    diagnostics: Vec<Diagnostic>,
    radio_targets: Vec<String>,
    target_index: TargetIndex,
    link_abbreviations: Vec<LinkAbbreviation>,
}

#[derive(Default)]
struct DocumentParts {
    properties: Vec<Property<ParsedAnnotation>>,
    children: Vec<Element<ParsedAnnotation>>,
    sections: Vec<Section<ParsedAnnotation>>,
}

impl<'a> Converter<'a> {
    fn new(source: &'a str, config: &'a ParseConfig) -> Self {
        Self {
            source,
            config,
            lines: LineIndex::new(source),
            diagnostics: Vec::new(),
            radio_targets: Vec::new(),
            target_index: TargetIndex::default(),
            link_abbreviations: Vec::new(),
        }
    }

    fn document(mut self, root: &SyntaxNode) -> ParsedAst {
        let prescan = self.semantic_prescan(root);
        let target_index = prescan.target_index;
        self.radio_targets = target_index.radio_targets();
        self.target_index = target_index;
        self.link_abbreviations = prescan.link_abbreviations.clone();
        self.diagnostics = prescan.diagnostics;
        let ann = self.node_ann(root);
        let parts = root
            .children()
            .fold(DocumentParts::default(), |mut parts, node| {
                self.push_document_child(&mut parts, &node);
                parts
            });
        let targets = std::mem::take(&mut self.target_index.definitions);

        let mut properties = prescan.properties;
        properties.extend(parts.properties);

        let mut document = Document {
            ann,
            properties,
            archive_locations: prescan.archive_locations,
            metadata: prescan.metadata,
            filetags: prescan.filetags,
            export_settings: prescan.export_settings,
            link_abbreviations: prescan.link_abbreviations,
            includes: prescan.includes,
            macro_definitions: prescan.macro_definitions,
            targets,
            footnotes: prescan.footnotes,
            children: parts.children,
            sections: parts.sections,
            diagnostics: self.diagnostics,
        };
        finalize_document(&mut document);
        document
    }

    fn push_document_child(&mut self, parts: &mut DocumentParts, node: &SyntaxNode) {
        match node.kind() {
            SyntaxKind::SECTION => {
                let section_children = self.elements_from_container(node);
                parts.properties.extend(
                    section_children
                        .iter()
                        .filter_map(|child| match &child.data {
                            ElementData::PropertyDrawer(properties) => Some(properties),
                            _ => None,
                        })
                        .flatten()
                        .cloned(),
                );
                parts.children.extend(section_children);
            }
            SyntaxKind::HEADLINE => parts.sections.push(self.section(node)),
            _ => {}
        }
    }

    fn semantic_prescan(&mut self, root: &SyntaxNode) -> SemanticPrescan {
        let mut prescan = SemanticPrescan::default();

        for node in root.descendants() {
            collect_target_node(
                &node,
                &mut prescan.target_index,
                &|node| self.node_ann(node),
                &|token| self.token_ann(token),
            );

            if node.kind() != SyntaxKind::KEYWORD {
                continue;
            }

            let keyword = self.keyword(&node, false);
            if keyword.key.eq_ignore_ascii_case("INCLUDE") {
                match include_directive(keyword) {
                    Ok(include) => prescan.includes.push(include),
                    Err((range, message)) => prescan.diagnostics.push(Diagnostic {
                        range,
                        kind: DiagnosticKind::Conversion,
                        message,
                    }),
                }
            } else if keyword.key.eq_ignore_ascii_case("MACRO") {
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

        prescan
    }

    fn section(&mut self, node: &SyntaxNode) -> Section<ParsedAnnotation> {
        let legacy = syntax_ast::Headline::cast(node.clone()).expect("headline node");
        let properties = legacy
            .properties()
            .map(|drawer| self.properties(&drawer.syntax))
            .unwrap_or_default();
        let anchor = properties
            .iter()
            .find(|property| property.key.eq_ignore_ascii_case("CUSTOM_ID"))
            .map(|property| property.value.clone());
        let planning = legacy
            .planning()
            .map(|planning| self.planning(&planning.syntax))
            .unwrap_or_default();
        let children = legacy
            .section()
            .map(|section| self.elements_from_container(&section.syntax))
            .unwrap_or_default();
        let subsections = node
            .children()
            .filter(|child| child.kind() == SyntaxKind::HEADLINE)
            .map(|child| self.section(&child))
            .collect();
        let todo = legacy.todo_keyword().map(|name| TodoKeyword {
            state: match legacy.todo_type() {
                Some(syntax_ast::TodoType::Done) => TodoState::Done,
                _ => TodoState::Todo,
            },
            name: name.to_string(),
        });
        let title = legacy.title().collect::<Vec<_>>();

        Section {
            ann: self.node_ann(node),
            level: legacy.level(),
            properties,
            effective_properties: Vec::new(),
            archive: Default::default(),
            attachment: Default::default(),
            todo,
            is_comment: legacy.is_commented(),
            priority: Priority::from_cookie(legacy.priority().map(|x| x.to_string())),
            title: self.objects_from_elements(title),
            raw_title: legacy.title_raw(),
            anchor,
            tags: legacy.tags().map(|x| x.to_string()).collect(),
            effective_tags: legacy.tags().map(|x| x.to_string()).collect(),
            planning,
            children,
            subsections,
        }
    }

    fn inlinetask(&mut self, node: &SyntaxNode) -> Inlinetask<ParsedAnnotation> {
        let title = node
            .children()
            .find(|child| child.kind() == SyntaxKind::HEADLINE_TITLE)
            .map(|title| self.objects_from_elements(title.children_with_tokens()))
            .unwrap_or_default();
        let planning = node
            .children()
            .find(|child| child.kind() == SyntaxKind::PLANNING)
            .map(|planning| self.planning(&planning))
            .unwrap_or_default();
        let properties = node
            .children()
            .find(|child| child.kind() == SyntaxKind::PROPERTY_DRAWER)
            .map(|drawer| self.properties(&drawer))
            .unwrap_or_default();
        let children = node
            .children()
            .find(|child| child.kind() == SyntaxKind::SECTION)
            .map(|section| self.elements_from_container(&section))
            .unwrap_or_default();
        let end = node
            .children()
            .find(|child| child.kind() == SyntaxKind::INLINETASK_END)
            .map(|end| InlinetaskEnd {
                ann: self.node_ann(&end),
                level: headline_level(&end),
                raw: end.to_string(),
            });

        Inlinetask {
            level: headline_level(node),
            todo: headline_todo(node),
            priority: Priority::from_cookie(headline_priority(node)),
            title,
            raw_title: headline_raw_title(node),
            tags: headline_tags(node),
            planning,
            properties,
            children,
            end,
        }
    }

    fn elements_from_container(&mut self, node: &SyntaxNode) -> Vec<Element<ParsedAnnotation>> {
        node.children()
            .filter_map(|child| self.element(&child))
            .collect()
    }

    fn element(&mut self, node: &SyntaxNode) -> Option<Element<ParsedAnnotation>> {
        let affiliated_keywords = self.affiliated_keywords(node);
        let data = match node.kind() {
            SyntaxKind::AFFILIATED_KEYWORD => return None,
            SyntaxKind::PARAGRAPH => {
                ElementData::Paragraph(self.objects_from_elements(node.children_with_tokens()))
            }
            SyntaxKind::KEYWORD => ElementData::Keyword(self.keyword(node, false)),
            SyntaxKind::BABEL_CALL => ElementData::BabelCall(self.keyword(node, false)),
            SyntaxKind::CLOCK => ElementData::Clock(self.clock(node)),
            SyntaxKind::DRAWER => ElementData::Drawer(self.drawer(node)),
            SyntaxKind::PROPERTY_DRAWER => ElementData::PropertyDrawer(self.properties(node)),
            SyntaxKind::LIST => ElementData::List(self.list(node)),
            SyntaxKind::ORG_TABLE => ElementData::Table(self.table(node)),
            SyntaxKind::TABLE_EL => ElementData::TableEl {
                raw: self.table_el(node),
            },
            SyntaxKind::SOURCE_BLOCK
            | SyntaxKind::EXAMPLE_BLOCK
            | SyntaxKind::EXPORT_BLOCK
            | SyntaxKind::QUOTE_BLOCK
            | SyntaxKind::VERSE_BLOCK
            | SyntaxKind::CENTER_BLOCK
            | SyntaxKind::COMMENT_BLOCK
            | SyntaxKind::SPECIAL_BLOCK
            | SyntaxKind::DYN_BLOCK => ElementData::Block(self.block(node)),
            SyntaxKind::FN_DEF => ElementData::FootnoteDef(self.footnote_def(node)),
            SyntaxKind::INLINETASK => ElementData::Inlinetask(Box::new(self.inlinetask(node))),
            SyntaxKind::COMMENT => ElementData::Comment(node.to_string()),
            SyntaxKind::FIXED_WIDTH => ElementData::FixedWidth(self.fixed_width(node)),
            SyntaxKind::RULE => ElementData::Rule,
            SyntaxKind::LATEX_ENVIRONMENT => ElementData::LatexEnvironment(node.to_string()),
            kind => {
                self.diagnostic(
                    node.text_range(),
                    DiagnosticKind::UnsupportedElement,
                    format!("semantic AST has no dedicated element mapping for {kind:?}"),
                );
                ElementData::Unknown {
                    kind: UnsupportedSyntaxKind::new(format!("{kind:?}")),
                    raw: node.to_string(),
                }
            }
        };

        Some(Element {
            ann: self.node_ann(node),
            affiliated_keywords,
            data,
        })
    }

    fn affiliated_keywords(&mut self, node: &SyntaxNode) -> Vec<Keyword<ParsedAnnotation>> {
        node.children()
            .take_while(|child| child.kind() == SyntaxKind::AFFILIATED_KEYWORD)
            .map(|child| self.keyword(&child, true))
            .collect()
    }

    fn keyword(&mut self, node: &SyntaxNode, affiliated: bool) -> Keyword<ParsedAnnotation> {
        if affiliated {
            let legacy = syntax_ast::AffiliatedKeyword::cast(node.clone()).expect("keyword node");
            let key = legacy.key().to_string();
            let value = legacy.value().map(|x| x.to_string()).unwrap_or_default();
            Keyword {
                ann: self.node_ann(node),
                key: key.clone(),
                optional: legacy.optional().map(|x| x.to_string()),
                parsed: self.keyword_parsed_objects(node, &key, &value),
                attributes: keyword_attributes(&key, &value),
                value,
            }
        } else {
            let legacy = syntax_ast::SyntaxKeyword::cast(node.clone());
            if let Some(legacy) = legacy {
                let key = legacy.key().to_string();
                let value = legacy.value().to_string();
                Keyword {
                    ann: self.node_ann(node),
                    key: key.clone(),
                    optional: None,
                    parsed: self.keyword_parsed_objects(node, &key, &value),
                    attributes: keyword_attributes(&key, &value),
                    value,
                }
            } else {
                Keyword {
                    ann: self.node_ann(node),
                    key: format!("{:?}", node.kind()),
                    optional: None,
                    parsed: Vec::new(),
                    attributes: Vec::new(),
                    value: node.to_string(),
                }
            }
        }
    }

    fn keyword_parsed_objects(
        &mut self,
        node: &SyntaxNode,
        key: &str,
        value: &str,
    ) -> Vec<Object<ParsedAnnotation>> {
        if !is_parsed_keyword(key) {
            return Vec::new();
        }

        let node_start = usize::from(node.text_range().start());
        let raw = node.to_string();
        let value_start = raw
            .find(value)
            .map_or(node_start, |offset| node_start + offset);
        self.objects_from_raw(value, value_start)
    }

    fn properties(&self, node: &SyntaxNode) -> Vec<Property<ParsedAnnotation>> {
        syntax_ast::PropertyDrawer::cast(node.clone())
            .map(|drawer| {
                drawer
                    .iter()
                    .map(|(key, value)| Property {
                        ann: self.token_ann(value.syntax()),
                        key: key.to_string(),
                        value: value.to_string(),
                        duration: OrgDuration::parse(value.to_string()),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn planning(&self, node: &SyntaxNode) -> Planning {
        let mut planning = Planning::default();
        for child in node.children() {
            let timestamp = child.children().find_map(|n| self.timestamp_node(&n));
            match child.kind() {
                SyntaxKind::PLANNING_DEADLINE => planning.deadline = timestamp,
                SyntaxKind::PLANNING_SCHEDULED => planning.scheduled = timestamp,
                SyntaxKind::PLANNING_CLOSED => planning.closed = timestamp,
                _ => {}
            }
        }
        planning
    }

    fn clock(&self, node: &SyntaxNode) -> Clock {
        let legacy = syntax_ast::SyntaxClock::cast(node.clone()).expect("clock node");
        let value = node
            .children()
            .find_map(|child| self.timestamp_node(&child));

        Clock {
            value,
            duration: legacy.duration().map(|token| token.to_string()),
            parsed_duration: legacy
                .duration()
                .and_then(|token| OrgDuration::parse(token.to_string())),
            raw: node.to_string(),
        }
    }

    fn drawer(&mut self, node: &SyntaxNode) -> Drawer<ParsedAnnotation> {
        let name = syntax_ast::SyntaxDrawer::cast(node.clone())
            .map(|drawer| drawer.name().to_string())
            .unwrap_or_default();
        let children = node
            .children()
            .find(|child| child.kind() == SyntaxKind::DRAWER_CONTENT)
            .map(|child| self.elements_from_container(&child))
            .unwrap_or_default();

        Drawer {
            name,
            children,
            raw: node.to_string(),
        }
    }

    fn list(&mut self, node: &SyntaxNode) -> List<ParsedAnnotation> {
        let legacy = syntax_ast::SyntaxList::cast(node.clone()).expect("list node");
        let has_descriptive_item = node.children().any(|item| {
            item.kind() == SyntaxKind::LIST_ITEM
                && item
                    .children()
                    .any(|child| child.kind() == SyntaxKind::LIST_ITEM_TAG)
        });
        let list_type = if has_descriptive_item || legacy.is_descriptive() {
            ListType::Descriptive
        } else if legacy.is_ordered() {
            ListType::Ordered
        } else {
            ListType::Unordered
        };
        let items = node
            .children()
            .filter(|child| child.kind() == SyntaxKind::LIST_ITEM)
            .map(|child| self.list_item(&child))
            .collect();

        List { list_type, items }
    }

    fn list_item(&mut self, node: &SyntaxNode) -> ListItem<ParsedAnnotation> {
        let legacy = syntax_ast::SyntaxListItem::cast(node.clone()).expect("list item node");
        let tag = legacy.tag().collect::<Vec<_>>();
        let children = node
            .children()
            .find(|child| child.kind() == SyntaxKind::LIST_ITEM_CONTENT)
            .map(|child| self.elements_from_container(&child))
            .unwrap_or_default();
        let checkbox = legacy.checkbox().and_then(|token| match token.as_ref() {
            "X" => Some(Checkbox::On),
            " " => Some(Checkbox::Off),
            "-" => Some(Checkbox::Trans),
            _ => None,
        });

        ListItem {
            ann: self.node_ann(node),
            bullet: legacy.bullet().to_string(),
            counter: legacy.counter().map(|x| x.to_string()),
            checkbox,
            tag: self.objects_from_elements(tag),
            children,
        }
    }

    fn table(&mut self, node: &SyntaxNode) -> Table<ParsedAnnotation> {
        let row_nodes = node
            .children()
            .filter(|child| {
                matches!(
                    child.kind(),
                    SyntaxKind::ORG_TABLE_RULE_ROW | SyntaxKind::ORG_TABLE_STANDARD_ROW
                )
            })
            .collect::<Vec<_>>();
        let column_alignments = table_column_alignments(&row_nodes);
        let rows = row_nodes
            .iter()
            .map(|child| TableRow {
                ann: self.node_ann(child),
                is_rule: child.kind() == SyntaxKind::ORG_TABLE_RULE_ROW,
                cells: child
                    .children()
                    .filter(|cell| cell.kind() == SyntaxKind::ORG_TABLE_CELL)
                    .map(|cell| TableCell {
                        ann: self.node_ann(&cell),
                        objects: self.objects_from_elements(cell.children_with_tokens()),
                    })
                    .collect(),
            })
            .collect();
        let formulas = node
            .children()
            .filter(|child| child.kind() == SyntaxKind::KEYWORD)
            .map(|child| self.keyword(&child, false))
            .collect::<Vec<_>>();
        let parsed_formulas = parsed_table_formulas(&formulas);

        Table {
            rows,
            column_alignments,
            formulas,
            parsed_formulas,
        }
    }

    fn table_el(&self, node: &SyntaxNode) -> String {
        syntax_ast::TableEl::cast(node.clone())
            .map(|table| table.raw())
            .unwrap_or_else(|| node.to_string())
    }

    fn block(&mut self, node: &SyntaxNode) -> Block<ParsedAnnotation> {
        let parts = block_parts(node);
        let kind = match node.kind() {
            SyntaxKind::SOURCE_BLOCK => BlockKind::Source,
            SyntaxKind::EXAMPLE_BLOCK => BlockKind::Example,
            SyntaxKind::EXPORT_BLOCK => BlockKind::Export,
            SyntaxKind::QUOTE_BLOCK => BlockKind::Quote,
            SyntaxKind::VERSE_BLOCK => BlockKind::Verse,
            SyntaxKind::CENTER_BLOCK => BlockKind::Center,
            SyntaxKind::COMMENT_BLOCK => BlockKind::Comment,
            SyntaxKind::DYN_BLOCK => BlockKind::Dynamic,
            SyntaxKind::SPECIAL_BLOCK => BlockKind::Special(
                block_name_from_begin(parts.begin.as_ref()).unwrap_or_else(|| "special".into()),
            ),
            _ => BlockKind::Special(format!("{:?}", node.kind())),
        };

        let source = syntax_ast::SourceBlock::cast(node.clone());
        let export = syntax_ast::ExportBlock::cast(node.clone());
        let switches = block_switches_from_begin(parts.begin.as_ref());
        let switch_options = parse_block_switches(switches.as_deref());
        let preserve_indentation =
            self.config.src_preserve_indentation || switch_options.preserve_indentation;
        let value = parts
            .content
            .as_ref()
            .map(SyntaxNode::to_string)
            .unwrap_or_default();
        let children = parts
            .content
            .as_ref()
            .map(|content| self.elements_from_container(content))
            .unwrap_or_default();

        let value = source
            .as_ref()
            .map(|block| block.value())
            .or_else(|| export.as_ref().map(|block| block.value()))
            .unwrap_or(value);
        let lines = if matches!(kind, BlockKind::Source | BlockKind::Example) {
            let source = parts.content.as_ref().map(SyntaxNode::to_string);
            let line_ranges = parts
                .content
                .as_ref()
                .zip(source.as_deref())
                .map(|(content, source)| block_content_line_ranges(content, source))
                .unwrap_or_default();
            let fallback_range = parts
                .content
                .as_ref()
                .map(|content| {
                    let end = usize::from(content.text_range().end());
                    text_range(end, end)
                })
                .unwrap_or_else(|| node.text_range());

            parse_block_lines(
                &value,
                source.as_deref(),
                BlockLineOptions {
                    switches: &switch_options,
                    tab_width: self.config.src_tab_width,
                    preserve_indentation,
                },
                |index| self.ann(line_ranges.get(index).copied().unwrap_or(fallback_range)),
            )
        } else {
            Vec::new()
        };
        let code_refs = block_code_refs(&lines);
        let parameters = source
            .as_ref()
            .and_then(|block| block.parameters().map(|x| x.to_string()));

        Block {
            kind,
            name: semantic_block_name(node.kind(), parts.begin.as_ref()),
            language: source
                .as_ref()
                .and_then(|block| block.language().map(|x| x.to_string())),
            line_numbering: switch_options.line_numbering.clone(),
            switch_options,
            preserve_indentation,
            lines,
            code_refs,
            switches,
            header_args: parse_block_header_args(parameters.as_deref()),
            parameters,
            value,
            children,
        }
    }

    fn fixed_width(&mut self, node: &SyntaxNode) -> SemanticFixedWidth<ParsedAnnotation> {
        let syntax = syntax_ast::FixedWidth::cast(node.clone());
        let value = syntax
            .as_ref()
            .map(syntax_ast::FixedWidth::value)
            .unwrap_or_else(|| node.to_string());
        let source = node.to_string();
        let line_ranges = source_line_ranges(usize::from(node.text_range().start()), &source);
        let switches = BlockSwitches::default();
        let fallback_range = {
            let end = usize::from(node.text_range().end());
            text_range(end, end)
        };
        let lines = parse_block_lines(
            &value,
            Some(&source),
            BlockLineOptions {
                switches: &switches,
                tab_width: self.config.src_tab_width,
                preserve_indentation: self.config.src_preserve_indentation,
            },
            |index| self.ann(line_ranges.get(index).copied().unwrap_or(fallback_range)),
        );

        SemanticFixedWidth { value, lines }
    }

    fn footnote_def(&mut self, node: &SyntaxNode) -> FootnoteDef<ParsedAnnotation> {
        let parts = node
            .children_with_tokens()
            .fold(FootnoteDefParts::default(), FootnoteDefParts::push);
        let children = self
            .paragraph_from_elements(parts.content)
            .into_iter()
            .collect::<Vec<_>>();

        FootnoteDef {
            label: parts.label,
            children,
        }
    }

    fn paragraph_from_elements(
        &mut self,
        elements: Vec<SyntaxElement>,
    ) -> Option<Element<ParsedAnnotation>> {
        let range = range_from_elements(&elements)?;
        let objects = self.objects_from_elements(elements);

        Some(Element {
            ann: self.ann(range),
            affiliated_keywords: Vec::new(),
            data: ElementData::Paragraph(objects),
        })
    }

    fn objects_from_elements(
        &mut self,
        elements: impl IntoIterator<Item = SyntaxElement>,
    ) -> Vec<Object<ParsedAnnotation>> {
        let objects = elements
            .into_iter()
            .filter_map(|element| self.object(element))
            .collect();
        self.project_radio_links(objects)
    }

    fn project_radio_links(
        &self,
        objects: Vec<Object<ParsedAnnotation>>,
    ) -> Vec<Object<ParsedAnnotation>> {
        if self.radio_targets.is_empty() {
            return objects;
        }

        match self.config.radio_link_projection {
            RadioLinkProjection::PlainText => self.project_plain_text_radio_links(objects),
            RadioLinkProjection::Semantic => self.project_semantic_radio_links(objects),
        }
    }

    fn project_plain_text_radio_links(
        &self,
        objects: Vec<Object<ParsedAnnotation>>,
    ) -> Vec<Object<ParsedAnnotation>> {
        let mut projected = Vec::with_capacity(objects.len());

        for object in objects {
            match object {
                Object {
                    ann,
                    data: ObjectData::Plain(value),
                } => self.extend_radio_links_in_plain(&mut projected, ann, value),
                _ => projected.push(object),
            }
        }

        projected
    }

    fn project_semantic_radio_links(
        &self,
        objects: Vec<Object<ParsedAnnotation>>,
    ) -> Vec<Object<ParsedAnnotation>> {
        let capacity = objects.len();
        objects
            .into_iter()
            .fold(
                SemanticRadioProjection::new(self, capacity),
                SemanticRadioProjection::push,
            )
            .finish()
    }

    fn project_radio_links_in_object_run(
        &self,
        objects: Vec<Object<ParsedAnnotation>>,
    ) -> Vec<Object<ParsedAnnotation>> {
        let Some(first) = objects.first() else {
            return objects;
        };
        let base = usize::from(first.ann.range.start());
        let raw = objects
            .iter()
            .map(|object| object.ann.raw.as_str())
            .collect::<String>();
        let spans = object_run_spans(&objects);
        let mut projected = Vec::with_capacity(objects.len());
        let mut emitted_until = 0;
        let mut search_cursor = 0;

        while let Some((start, end, target)) =
            next_radio_link(&raw, search_cursor, &self.radio_targets)
        {
            if start < emitted_until {
                search_cursor = end;
                continue;
            }

            let Some(description) =
                self.slice_radio_link_objects(&objects, &spans, base, start, end)
            else {
                search_cursor = next_char_boundary(&raw, start);
                continue;
            };
            let Some(prefix) =
                self.slice_radio_link_objects(&objects, &spans, base, emitted_until, start)
            else {
                return objects;
            };

            projected.extend(prefix);

            let raw_description = raw[start..end].to_string();
            let link_ann = self.ann(text_range(base + start, base + end));
            projected.push(Object {
                ann: link_ann,
                data: ObjectData::Link(Box::new(Link {
                    path: LinkPath::new(target.to_string()),
                    target: LinkTarget::Internal(target.to_string()),
                    description,
                    default_description: Vec::new(),
                    raw_description,
                    description_state: LinkDescriptionState::Explicit,
                    media_kind: LinkMediaKind::Normal,
                    caption: None,
                    search: None,
                    attachment: None,
                    file: None,
                })),
            });

            emitted_until = end;
            search_cursor = end;
        }

        if emitted_until == 0 {
            return objects;
        }

        let Some(suffix) =
            self.slice_radio_link_objects(&objects, &spans, base, emitted_until, raw.len())
        else {
            return objects;
        };
        projected.extend(suffix);
        projected
    }

    fn slice_radio_link_objects(
        &self,
        objects: &[Object<ParsedAnnotation>],
        spans: &[ObjectRunSpan],
        base: usize,
        start: usize,
        end: usize,
    ) -> Option<Vec<Object<ParsedAnnotation>>> {
        if start == end {
            return Some(Vec::new());
        }

        let first = spans.partition_point(|span| span.end <= start);
        spans[first..]
            .iter()
            .zip(&objects[first..])
            .take_while(|(span, _)| span.start < end)
            .map(|(span, object)| self.slice_radio_link_object(object, *span, base, start, end))
            .collect()
    }

    fn slice_radio_link_object(
        &self,
        object: &Object<ParsedAnnotation>,
        span: ObjectRunSpan,
        base: usize,
        start: usize,
        end: usize,
    ) -> Option<Object<ParsedAnnotation>> {
        let slice_start = start.max(span.start);
        let slice_end = end.min(span.end);
        if slice_start == span.start && slice_end == span.end {
            return Some(object.clone());
        }

        let ObjectData::Plain(value) = &object.data else {
            return None;
        };
        let relative_start = slice_start - span.start;
        let relative_end = slice_end - span.start;
        let raw = value.get(relative_start..relative_end)?.to_string();
        Some(Object {
            ann: self.ann(text_range(base + slice_start, base + slice_end)),
            data: ObjectData::Plain(raw),
        })
    }

    fn extend_radio_links_in_plain(
        &self,
        objects: &mut Vec<Object<ParsedAnnotation>>,
        ann: ParsedAnnotation,
        value: String,
    ) {
        let mut cursor = 0;
        let base = usize::from(ann.range.start());

        while let Some((start, end, target)) = next_radio_link(&value, cursor, &self.radio_targets)
        {
            if cursor < start {
                objects.push(Object {
                    ann: self.ann(text_range(base + cursor, base + start)),
                    data: ObjectData::Plain(value[cursor..start].to_string()),
                });
            }

            let raw = value[start..end].to_string();
            let link_ann = self.ann(text_range(base + start, base + end));
            objects.push(Object {
                ann: link_ann.clone(),
                data: ObjectData::Link(Box::new(Link {
                    path: LinkPath::new(target.to_string()),
                    target: LinkTarget::Internal(target.to_string()),
                    description: vec![Object {
                        ann: link_ann,
                        data: ObjectData::Plain(raw.clone()),
                    }],
                    default_description: Vec::new(),
                    raw_description: raw,
                    description_state: LinkDescriptionState::Explicit,
                    media_kind: LinkMediaKind::Normal,
                    caption: None,
                    search: None,
                    attachment: None,
                    file: None,
                })),
            });

            cursor = end;
        }

        if cursor == 0 {
            objects.push(Object {
                ann,
                data: ObjectData::Plain(value),
            });
            return;
        }

        if cursor < value.len() {
            objects.push(Object {
                ann: self.ann(text_range(base + cursor, base + value.len())),
                data: ObjectData::Plain(value[cursor..].to_string()),
            });
        }
    }

    fn object(&mut self, element: SyntaxElement) -> Option<Object<ParsedAnnotation>> {
        match element {
            NodeOrToken::Token(token) => self.object_token(token),
            NodeOrToken::Node(node) => self.object_node(node),
        }
    }

    fn object_token(&self, token: SyntaxToken) -> Option<Object<ParsedAnnotation>> {
        match token.kind() {
            SyntaxKind::TEXT => Some(Object {
                ann: self.token_ann(&token),
                data: ObjectData::Plain(token.text().to_string()),
            }),
            SyntaxKind::NEW_LINE | SyntaxKind::WHITESPACE | SyntaxKind::BLANK_LINE => {
                Some(Object {
                    ann: self.token_ann(&token),
                    data: ObjectData::Plain(token.text().to_string()),
                })
            }
            _ => None,
        }
    }

    fn object_node(&mut self, node: SyntaxNode) -> Option<Object<ParsedAnnotation>> {
        let data = match node.kind() {
            SyntaxKind::AFFILIATED_KEYWORD => return None,
            SyntaxKind::BOLD => self.markup(&node, MarkupKind::Bold),
            SyntaxKind::ITALIC => self.markup(&node, MarkupKind::Italic),
            SyntaxKind::UNDERLINE => self.markup(&node, MarkupKind::Underline),
            SyntaxKind::STRIKE => self.markup(&node, MarkupKind::Strike),
            SyntaxKind::SUPERSCRIPT => self.markup(&node, MarkupKind::Superscript),
            SyntaxKind::SUBSCRIPT => self.markup(&node, MarkupKind::Subscript),
            SyntaxKind::CODE => ObjectData::Code(strip_pair(&node.to_string()).to_string()),
            SyntaxKind::VERBATIM => ObjectData::Verbatim(strip_pair(&node.to_string()).to_string()),
            SyntaxKind::TIMESTAMP_ACTIVE
            | SyntaxKind::TIMESTAMP_INACTIVE
            | SyntaxKind::TIMESTAMP_DIARY => ObjectData::Timestamp(
                self.timestamp_node(&node)
                    .expect("timestamp kind must map to timestamp"),
            ),
            SyntaxKind::ENTITY => ObjectData::Entity(node.to_string()),
            SyntaxKind::LATEX_FRAGMENT => ObjectData::LatexFragment(node.to_string()),
            SyntaxKind::SNIPPET => self.export_snippet(&node),
            SyntaxKind::FN_REF => self.footnote_ref(&node),
            SyntaxKind::CITATION => self.citation(&node),
            #[cfg(feature = "syntax-org-fc")]
            SyntaxKind::CLOZE => self.cloze(&node),
            SyntaxKind::INLINE_CALL => self.inline_call(&node),
            SyntaxKind::INLINE_SRC => self.inline_src(&node),
            SyntaxKind::LINK => self.link(&node),
            SyntaxKind::TARGET => ObjectData::Target(strip_wrapping(&node.to_string(), "<<", ">>")),
            SyntaxKind::RADIO_TARGET => {
                ObjectData::RadioTarget(strip_wrapping(&node.to_string(), "<<<", ">>>"))
            }
            SyntaxKind::MACROS => self.macro_object(&node),
            SyntaxKind::COOKIE => ObjectData::StatisticCookie(node.to_string()),
            SyntaxKind::LINE_BREAK => ObjectData::LineBreak,
            kind => {
                self.diagnostic(
                    node.text_range(),
                    DiagnosticKind::UnsupportedObject,
                    format!("semantic AST has no dedicated object mapping for {kind:?}"),
                );
                ObjectData::Unknown {
                    kind: UnsupportedSyntaxKind::new(format!("{kind:?}")),
                    raw: node.to_string(),
                }
            }
        };

        Some(Object {
            ann: self.node_ann(&node),
            data,
        })
    }

    fn markup(&mut self, node: &SyntaxNode, kind: MarkupKind) -> ObjectData<ParsedAnnotation> {
        ObjectData::Markup {
            kind,
            children: self.objects_from_elements(node.children_with_tokens()),
        }
    }

    fn timestamp_node(&self, node: &SyntaxNode) -> Option<Timestamp> {
        let kind = match node.kind() {
            SyntaxKind::TIMESTAMP_ACTIVE => TimestampKind::Active,
            SyntaxKind::TIMESTAMP_INACTIVE => TimestampKind::Inactive,
            SyntaxKind::TIMESTAMP_DIARY => TimestampKind::Diary,
            _ => return None,
        };
        let legacy = syntax_ast::SyntaxTimestamp::cast(node.clone()).expect("timestamp node");
        let is_range = legacy.is_range();
        let (start, end) = timestamp_moment_range(node, is_range);
        Some(Timestamp {
            kind,
            raw: node.to_string(),
            is_range,
            start,
            end,
            repeater: timestamp_repeater(&legacy),
            warning: timestamp_warning(&legacy),
        })
    }

    fn export_snippet(&self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        if let Some(snippet) = syntax_ast::Snippet::cast(node.clone()) {
            ObjectData::ExportSnippet {
                backend: snippet.backend().to_string(),
                value: snippet.value().to_string(),
            }
        } else {
            ObjectData::Unknown {
                kind: UnsupportedSyntaxKind::new("SNIPPET"),
                raw: node.to_string(),
            }
        }
    }

    fn footnote_ref(&mut self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let parts = node
            .children_with_tokens()
            .take_while(|element| element.kind() != SyntaxKind::R_BRACKET)
            .fold(FootnoteRefParts::default(), FootnoteRefParts::push);

        ObjectData::FootnoteRef {
            label: (!parts.label.is_empty()).then_some(parts.label),
            resolved_label: None,
            definition: self.objects_from_elements(parts.definition),
        }
    }

    fn citation(&mut self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let raw = node.to_string();
        let Some((head, body)) = raw
            .strip_prefix('[')
            .and_then(|raw| raw.strip_suffix(']'))
            .and_then(|inner| inner.split_once(':'))
        else {
            self.diagnostic(
                node.text_range(),
                DiagnosticKind::Conversion,
                "citation syntax node could not be split into head and body".into(),
            );
            return ObjectData::Unknown {
                kind: UnsupportedSyntaxKind::new("CITATION"),
                raw,
            };
        };

        let (style, variant) = citation_style(head);
        let node_start = usize::from(node.text_range().start());
        let body_start = node_start + 1 + head.len() + 1;
        let mut prefix = Vec::new();
        let mut suffix = Vec::new();
        let mut references = Vec::new();
        let mut saw_reference = false;
        let mut segment_start = 0;

        for segment in body.split(';') {
            let absolute_start = body_start + segment_start;
            segment_start += segment.len() + 1;

            if let Some((key_start, key_end)) = citation_key_range(segment) {
                saw_reference = true;
                references.push(CiteReference {
                    id: segment[key_start..key_end].to_string(),
                    prefix: self
                        .objects_from_raw_minimal(&segment[..key_start - 1], absolute_start),
                    suffix: self
                        .objects_from_raw_minimal(&segment[key_end..], absolute_start + key_end),
                });
            } else if saw_reference {
                if segment.contains('@') {
                    self.diagnostic(
                        node.text_range(),
                        DiagnosticKind::Conversion,
                        format!("malformed citation segment `{}`", segment.trim()),
                    );
                }
                suffix.extend(self.objects_from_raw_minimal(segment, absolute_start));
            } else {
                if segment.contains('@') {
                    self.diagnostic(
                        node.text_range(),
                        DiagnosticKind::Conversion,
                        format!("malformed citation segment `{}`", segment.trim()),
                    );
                }
                prefix.extend(self.objects_from_raw_minimal(segment, absolute_start));
            }
        }

        if references.is_empty() {
            self.diagnostic(
                node.text_range(),
                DiagnosticKind::Conversion,
                "citation syntax node did not contain a citation reference".into(),
            );
        }

        ObjectData::Citation(Citation {
            style,
            variant,
            prefix,
            suffix,
            references,
        })
    }

    #[cfg(feature = "syntax-org-fc")]
    fn cloze(&mut self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let legacy = syntax_ast::Cloze::cast(node.clone()).expect("cloze node");
        let text = legacy.text().collect::<Vec<_>>();
        ObjectData::Cloze {
            text: self.objects_from_elements(text),
            raw_text: legacy.text_raw(),
            hint: legacy.hint().map(|token| token.to_string()),
            id: legacy.id().map(|token| token.to_string()),
            raw: legacy.raw(),
        }
    }

    fn inline_call(&self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let legacy = syntax_ast::InlineCall::cast(node.clone()).expect("inline call node");
        let raw = node.to_string();
        ObjectData::InlineCall {
            name: legacy.call().to_string(),
            arguments: legacy.arguments().to_string(),
            header: legacy.inside_header().map(|token| token.to_string()),
            end_header: legacy.end_header().map(|token| token.to_string()),
            raw,
        }
    }

    fn inline_src(&self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let legacy = syntax_ast::InlineSrc::cast(node.clone()).expect("inline src node");
        let raw = node.to_string();
        ObjectData::InlineSrc {
            language: legacy.language().to_string(),
            parameters: legacy.parameters().map(|token| token.to_string()),
            value: legacy.value().to_string(),
            raw,
        }
    }

    fn link(&mut self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let legacy = syntax_ast::SyntaxLink::cast(node.clone()).expect("link node");
        let path = legacy.path().to_string();
        let target = self.link_target(&path, node.text_range());
        let search = link_search(&path);
        let attachment = attachment_link_from_path(&path).map(Box::new);
        let file = file_link(&path, search.clone()).map(Box::new);
        let description = legacy.description().collect::<Vec<_>>();
        let caption = legacy
            .caption()
            .map(|caption| self.keyword(&caption.syntax, true));

        ObjectData::Link(Box::new(Link {
            path: LinkPath::new(path),
            target,
            default_description: Vec::new(),
            raw_description: legacy.description_raw(),
            description_state: if legacy.has_description() {
                LinkDescriptionState::Explicit
            } else {
                LinkDescriptionState::None
            },
            media_kind: if legacy.is_image() {
                LinkMediaKind::Image
            } else {
                LinkMediaKind::Normal
            },
            caption,
            search,
            attachment,
            file,
            description: self.objects_from_elements(description),
        }))
    }

    fn link_target(&mut self, path: &str, range: TextRange) -> LinkTarget {
        match self.target_index.resolve(path) {
            TargetLookup::Found { key } => {
                return LinkTarget::Internal(key);
            }
            TargetLookup::Ambiguous { key, count } => {
                self.diagnostic(
                    range,
                    DiagnosticKind::Conversion,
                    format!("internal link target `{key}` is ambiguous across {count} definitions"),
                );
                return LinkTarget::Unresolved(path.to_string());
            }
            TargetLookup::Missing { key } if is_strict_internal_link_path(path) => {
                self.diagnostic(
                    range,
                    DiagnosticKind::Conversion,
                    format!("internal link target `{key}` was not found"),
                );
                return LinkTarget::Unresolved(path.to_string());
            }
            TargetLookup::Missing { .. } => {}
        }

        if let Some((protocol, path)) = path.split_once(':') {
            if let Some(expanded) =
                expand_link_abbreviation(protocol, path, &self.link_abbreviations)
            {
                return self.link_target(&expanded, range);
            }
            LinkTarget::Uri {
                protocol: protocol.to_string(),
                path: path.to_string(),
            }
        } else if path.starts_with('#') {
            LinkTarget::Internal(path.to_string())
        } else {
            LinkTarget::Unresolved(path.to_string())
        }
    }

    fn macro_object(&self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let raw = node.to_string();
        let inner = strip_wrapping(&raw, "{{{", "}}}");
        let (name, args) = inner
            .split_once('(')
            .map(|(name, args)| {
                (
                    name,
                    split_macro_args(args.strip_suffix(')').unwrap_or(args)),
                )
            })
            .unwrap_or((inner.as_str(), Vec::new()));
        ObjectData::Macro {
            name: name.to_string(),
            arguments: args,
        }
    }

    fn node_ann(&self, node: &SyntaxNode) -> ParsedAnnotation {
        self.ann(node.text_range())
    }

    fn token_ann(&self, token: &SyntaxToken) -> ParsedAnnotation {
        self.ann(token.text_range())
    }

    fn ann(&self, range: TextRange) -> ParsedAnnotation {
        let start = self.lines.position(range.start());
        let end = self.lines.position(range.end());
        let raw = self.raw(range).to_string();
        ParsedAnnotation {
            range,
            start,
            end,
            raw,
        }
    }

    fn raw(&self, range: TextRange) -> &str {
        let start: usize = range.start().into();
        let end: usize = range.end().into();
        self.source.get(start..end).unwrap_or_default()
    }

    fn objects_from_raw(
        &mut self,
        value: &str,
        absolute_start: usize,
    ) -> Vec<Object<ParsedAnnotation>> {
        self.objects_from_raw_with(value, absolute_start, false)
    }

    fn objects_from_raw_minimal(
        &mut self,
        value: &str,
        absolute_start: usize,
    ) -> Vec<Object<ParsedAnnotation>> {
        self.objects_from_raw_with(value, absolute_start, true)
    }

    fn objects_from_raw_with(
        &mut self,
        value: &str,
        absolute_start: usize,
        minimal: bool,
    ) -> Vec<Object<ParsedAnnotation>> {
        let Some((start, end)) = trimmed_range(value) else {
            return Vec::new();
        };
        let raw = &value[start..end];
        let base = absolute_start + start;
        let children = if minimal {
            minimal_object_nodes((raw, self.config).into())
        } else {
            standard_object_nodes((raw, self.config).into())
        };
        let root = SyntaxNode::new_root(
            node(SyntaxKind::PARAGRAPH, children)
                .into_node()
                .expect("paragraph node"),
        );
        let mut converter = Converter::new(raw, self.config);
        let objects = converter.objects_from_elements(root.children_with_tokens());
        self.diagnostics.extend(
            converter
                .diagnostics
                .into_iter()
                .map(|diagnostic| Diagnostic {
                    range: offset_range(diagnostic.range, base),
                    kind: diagnostic.kind,
                    message: diagnostic.message,
                }),
        );
        let mut map_ann = |ann: &ParsedAnnotation| self.ann(offset_range(ann.range, base));

        objects
            .iter()
            .map(|object| object.map_ann_with(&mut map_ann))
            .collect()
    }

    fn diagnostic(&mut self, range: TextRange, kind: DiagnosticKind, message: String) {
        self.diagnostics.push(Diagnostic {
            range,
            kind,
            message,
        });
    }
}

#[derive(Clone, Copy)]
struct ObjectRunSpan {
    start: usize,
    end: usize,
}

struct SemanticRadioProjection<'converter, 'source> {
    converter: &'converter Converter<'source>,
    projected: Vec<Object<ParsedAnnotation>>,
    run: Vec<Object<ParsedAnnotation>>,
}

impl<'converter, 'source> SemanticRadioProjection<'converter, 'source> {
    fn new(converter: &'converter Converter<'source>, capacity: usize) -> Self {
        Self {
            converter,
            projected: Vec::with_capacity(capacity),
            run: Vec::new(),
        }
    }

    fn push(mut self, object: Object<ParsedAnnotation>) -> Self {
        if is_semantic_radio_link_candidate(&object.data) {
            self.run.push(object);
        } else {
            self.flush();
            self.projected.push(object);
        }
        self
    }

    fn finish(mut self) -> Vec<Object<ParsedAnnotation>> {
        self.flush();
        self.projected
    }

    fn flush(&mut self) {
        if !self.run.is_empty() {
            self.projected.extend(
                self.converter
                    .project_radio_links_in_object_run(std::mem::take(&mut self.run)),
            );
        }
    }
}

fn object_run_spans(objects: &[Object<ParsedAnnotation>]) -> Vec<ObjectRunSpan> {
    objects
        .iter()
        .scan(0, |cursor, object| {
            let start = *cursor;
            *cursor += object.ann.raw.len();
            Some(ObjectRunSpan {
                start,
                end: *cursor,
            })
        })
        .collect()
}

fn block_content_line_ranges(content: &SyntaxNode, source: &str) -> Vec<TextRange> {
    source_line_ranges(usize::from(content.text_range().start()), source)
}

fn source_line_ranges(base: usize, source: &str) -> Vec<TextRange> {
    split_block_lines(source)
        .into_iter()
        .map(|line| text_range(base + line.start, base + line.end))
        .collect()
}
