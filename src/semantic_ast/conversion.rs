//! Conversion from the lossless syntax tree into the semantic AST.

use rowan::ast::AstNode;
use rowan::{NodeOrToken, TextRange, TextSize};

use crate::{
    config::ParseConfig,
    syntax::{
        combinator::{line_starts_iter, node},
        object::standard_object_nodes,
        SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken,
    },
    syntax_ast,
};

use super::{
    Block, BlockCodeRef, BlockKind, BlockLineNumberMode, BlockLineNumbering, Checkbox, Citation,
    CiteReference, Clock, Diagnostic, DiagnosticKind, Document, Drawer, Element, ElementData,
    FootnoteDef, Keyword, Link, LinkTarget, List, ListItem, ListType, MarkupKind, Object,
    ObjectData, ParsedAnnotation, ParsedAst, Planning, Property, RepeaterKind, Section,
    SourcePosition, Table, TableCell, TableRow, TimeUnit, Timestamp, TimestampKind,
    TimestampMoment, TimestampRepeater, TimestampWarning, TodoKeyword, TodoState, WarningKind,
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
}

impl<'a> Converter<'a> {
    fn new(source: &'a str, config: &'a ParseConfig) -> Self {
        Self {
            source,
            config,
            lines: LineIndex::new(source),
            diagnostics: Vec::new(),
            radio_targets: Vec::new(),
        }
    }

    fn document(mut self, root: &SyntaxNode) -> ParsedAst {
        self.radio_targets = collect_radio_targets(root);
        let ann = self.node_ann(root);
        let mut children = Vec::new();
        let mut sections = Vec::new();
        let mut properties = Vec::new();

        for node in root.children() {
            match node.kind() {
                SyntaxKind::SECTION => {
                    let section_children = self.elements_from_container(&node);
                    for child in section_children {
                        if let ElementData::PropertyDrawer(props) = &child.data {
                            properties.extend(props.clone());
                        }
                        children.push(child);
                    }
                }
                SyntaxKind::HEADLINE => sections.push(self.section(&node)),
                _ => {}
            }
        }

        Document {
            ann,
            properties,
            children,
            sections,
            diagnostics: self.diagnostics,
        }
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
            todo,
            is_comment: legacy.is_commented(),
            priority: legacy.priority().map(|x| x.to_string()),
            title: self.objects_from_elements(title),
            raw_title: legacy.title_raw(),
            anchor,
            tags: legacy.tags().map(|x| x.to_string()).collect(),
            planning,
            children,
            subsections,
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
            SyntaxKind::COMMENT => ElementData::Comment(node.to_string()),
            SyntaxKind::FIXED_WIDTH => ElementData::FixedWidth(node.to_string()),
            SyntaxKind::RULE => ElementData::Rule,
            SyntaxKind::LATEX_ENVIRONMENT => ElementData::LatexEnvironment(node.to_string()),
            kind => {
                self.diagnostic(
                    node.text_range(),
                    DiagnosticKind::UnsupportedElement,
                    format!("semantic AST has no dedicated element mapping for {kind:?}"),
                );
                ElementData::Unknown {
                    kind: format!("{kind:?}"),
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

    fn keyword(&self, node: &SyntaxNode, affiliated: bool) -> Keyword<ParsedAnnotation> {
        if affiliated {
            let legacy = syntax_ast::AffiliatedKeyword::cast(node.clone()).expect("keyword node");
            Keyword {
                ann: self.node_ann(node),
                key: legacy.key().to_string(),
                optional: legacy.optional().map(|x| x.to_string()),
                value: legacy.value().map(|x| x.to_string()).unwrap_or_default(),
            }
        } else {
            let legacy = syntax_ast::Keyword::cast(node.clone());
            if let Some(legacy) = legacy {
                Keyword {
                    ann: self.node_ann(node),
                    key: legacy.key().to_string(),
                    optional: None,
                    value: legacy.value().to_string(),
                }
            } else {
                Keyword {
                    ann: self.node_ann(node),
                    key: format!("{:?}", node.kind()),
                    optional: None,
                    value: node.to_string(),
                }
            }
        }
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
        let legacy = syntax_ast::Clock::cast(node.clone()).expect("clock node");
        let value = node
            .children()
            .find_map(|child| self.timestamp_node(&child));

        Clock {
            value,
            duration: legacy.duration().map(|token| token.to_string()),
            raw: node.to_string(),
        }
    }

    fn drawer(&mut self, node: &SyntaxNode) -> Drawer<ParsedAnnotation> {
        let name = syntax_ast::Drawer::cast(node.clone())
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
        let legacy = syntax_ast::List::cast(node.clone()).expect("list node");
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
        let legacy = syntax_ast::ListItem::cast(node.clone()).expect("list item node");
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
        let rows = node
            .children()
            .filter(|child| {
                matches!(
                    child.kind(),
                    SyntaxKind::ORG_TABLE_RULE_ROW | SyntaxKind::ORG_TABLE_STANDARD_ROW
                )
            })
            .map(|child| TableRow {
                ann: self.node_ann(&child),
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
            .collect();

        Table { rows, formulas }
    }

    fn table_el(&self, node: &SyntaxNode) -> String {
        syntax_ast::TableEl::cast(node.clone())
            .map(|table| table.raw())
            .unwrap_or_else(|| node.to_string())
    }

    fn block(&mut self, node: &SyntaxNode) -> Block<ParsedAnnotation> {
        let kind = match node.kind() {
            SyntaxKind::SOURCE_BLOCK => BlockKind::Source,
            SyntaxKind::EXAMPLE_BLOCK => BlockKind::Example,
            SyntaxKind::EXPORT_BLOCK => BlockKind::Export,
            SyntaxKind::QUOTE_BLOCK => BlockKind::Quote,
            SyntaxKind::VERSE_BLOCK => BlockKind::Verse,
            SyntaxKind::CENTER_BLOCK => BlockKind::Center,
            SyntaxKind::COMMENT_BLOCK => BlockKind::Comment,
            SyntaxKind::DYN_BLOCK => BlockKind::Dynamic,
            SyntaxKind::SPECIAL_BLOCK => {
                BlockKind::Special(block_name(node).unwrap_or_else(|| "special".into()))
            }
            _ => BlockKind::Special(format!("{:?}", node.kind())),
        };

        let source = syntax_ast::SourceBlock::cast(node.clone());
        let export = syntax_ast::ExportBlock::cast(node.clone());
        let switches = semantic_block_switches(node);
        let value = node
            .children()
            .find(|child| child.kind() == SyntaxKind::BLOCK_CONTENT)
            .map(|child| child.to_string())
            .unwrap_or_default();
        let children = node
            .children()
            .find(|child| child.kind() == SyntaxKind::BLOCK_CONTENT)
            .map(|child| self.elements_from_container(&child))
            .unwrap_or_default();

        let value = source
            .as_ref()
            .map(|block| block.value())
            .or_else(|| export.as_ref().map(|block| block.value()))
            .unwrap_or(value);
        let code_refs = if matches!(kind, BlockKind::Source | BlockKind::Example) {
            parse_block_code_refs(&value, switches.as_deref())
        } else {
            Vec::new()
        };

        Block {
            kind,
            name: semantic_block_name(node),
            language: source
                .as_ref()
                .and_then(|block| block.language().map(|x| x.to_string())),
            line_numbering: switches.as_deref().and_then(parse_block_line_numbering),
            code_refs,
            switches,
            parameters: source
                .as_ref()
                .and_then(|block| block.parameters().map(|x| x.to_string())),
            value,
            children,
        }
    }

    fn footnote_def(&mut self, node: &SyntaxNode) -> FootnoteDef<ParsedAnnotation> {
        let mut saw_fn_prefix = false;
        let mut saw_label_colon = false;
        let mut after_marker = false;
        let mut label = String::new();
        let mut content = Vec::new();

        for element in node.children_with_tokens() {
            match element.kind() {
                SyntaxKind::AFFILIATED_KEYWORD | SyntaxKind::L_BRACKET => {}
                SyntaxKind::TEXT if !saw_fn_prefix => {
                    saw_fn_prefix = true;
                }
                SyntaxKind::COLON if saw_fn_prefix && !saw_label_colon => {
                    saw_label_colon = true;
                }
                SyntaxKind::R_BRACKET if saw_label_colon => {
                    after_marker = true;
                }
                _ if after_marker => content.push(element),
                SyntaxKind::TEXT if saw_label_colon => {
                    label.push_str(
                        element
                            .as_token()
                            .map(|token| token.text())
                            .unwrap_or_default(),
                    );
                }
                _ => {}
            }
        }
        let children = self
            .paragraph_from_elements(content)
            .into_iter()
            .collect::<Vec<_>>();

        FootnoteDef { label, children }
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

        objects
            .into_iter()
            .flat_map(|object| match object.data {
                ObjectData::Plain(value) => self.project_radio_links_in_plain(&object.ann, &value),
                _ => vec![object],
            })
            .collect()
    }

    fn project_radio_links_in_plain(
        &self,
        ann: &ParsedAnnotation,
        value: &str,
    ) -> Vec<Object<ParsedAnnotation>> {
        let mut objects = Vec::new();
        let mut cursor = 0;
        let base = usize::from(ann.range.start());

        while let Some((start, end, target)) = next_radio_link(value, cursor, &self.radio_targets) {
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
                data: ObjectData::Link(Link {
                    path: target.to_string(),
                    target: LinkTarget::Internal(target.to_string()),
                    description: vec![Object {
                        ann: link_ann,
                        data: ObjectData::Plain(raw.clone()),
                    }],
                    raw_description: raw,
                    has_description: true,
                    is_image: false,
                    caption: None,
                }),
            });

            cursor = end;
        }

        if cursor == 0 {
            return vec![Object {
                ann: ann.clone(),
                data: ObjectData::Plain(value.to_string()),
            }];
        }

        if cursor < value.len() {
            objects.push(Object {
                ann: self.ann(text_range(base + cursor, base + value.len())),
                data: ObjectData::Plain(value[cursor..].to_string()),
            });
        }

        objects
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
                    kind: format!("{kind:?}"),
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
        let legacy = syntax_ast::Timestamp::cast(node.clone()).expect("timestamp node");
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
                kind: "SNIPPET".into(),
                raw: node.to_string(),
            }
        }
    }

    fn footnote_ref(&mut self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let mut saw_fn_prefix = false;
        let mut saw_label_colon = false;
        let mut in_definition = false;
        let mut label = String::new();
        let mut definition = Vec::new();

        for element in node.children_with_tokens() {
            match element.kind() {
                SyntaxKind::L_BRACKET => {}
                SyntaxKind::TEXT if !saw_fn_prefix => {
                    saw_fn_prefix = true;
                }
                SyntaxKind::COLON if saw_fn_prefix && !saw_label_colon => {
                    saw_label_colon = true;
                }
                SyntaxKind::COLON if saw_label_colon && !in_definition => {
                    in_definition = true;
                }
                SyntaxKind::R_BRACKET => break,
                _ if in_definition => definition.push(element),
                SyntaxKind::TEXT if saw_label_colon => {
                    label.push_str(
                        element
                            .as_token()
                            .map(|token| token.text())
                            .unwrap_or_default(),
                    );
                }
                _ => {}
            }
        }

        ObjectData::FootnoteRef {
            label: (!label.is_empty()).then_some(label),
            definition: self.objects_from_elements(definition),
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
                kind: "CITATION".into(),
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
                    prefix: self.objects_from_raw(&segment[..key_start - 1], absolute_start),
                    suffix: self.objects_from_raw(&segment[key_end..], absolute_start + key_end),
                });
            } else if saw_reference {
                suffix.extend(self.objects_from_raw(segment, absolute_start));
            } else {
                prefix.extend(self.objects_from_raw(segment, absolute_start));
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
        let legacy = syntax_ast::Link::cast(node.clone()).expect("link node");
        let path = legacy.path().to_string();
        let target = if let Some((protocol, path)) = path.split_once(':') {
            LinkTarget::Uri {
                protocol: protocol.to_string(),
                path: path.to_string(),
            }
        } else if path.starts_with('#') {
            LinkTarget::Internal(path)
        } else {
            LinkTarget::Unresolved(path)
        };
        let description = legacy.description().collect::<Vec<_>>();
        let caption = legacy
            .caption()
            .map(|caption| self.keyword(&caption.syntax, true));

        ObjectData::Link(Link {
            path: legacy.path().to_string(),
            target,
            raw_description: legacy.description_raw(),
            has_description: legacy.has_description(),
            is_image: legacy.is_image(),
            caption,
            description: self.objects_from_elements(description),
        })
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
        let Some((start, end)) = trimmed_range(value) else {
            return Vec::new();
        };
        let raw = &value[start..end];
        let base = absolute_start + start;
        let children = standard_object_nodes((raw, self.config).into());
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

struct LineIndex<'a> {
    source: &'a str,
    starts: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            starts: line_starts_iter(source).collect(),
        }
    }

    fn position(&self, offset: TextSize) -> SourcePosition {
        let offset = usize::from(offset).min(self.source.len());
        let line = match self.starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        let line_start = self.starts[line];
        SourcePosition {
            line: line + 1,
            column: self.source[line_start..offset].chars().count() + 1,
        }
    }
}

fn block_name(node: &SyntaxNode) -> Option<String> {
    node.children()
        .find(|child| child.kind() == SyntaxKind::BLOCK_BEGIN)
        .and_then(|begin| {
            begin
                .children_with_tokens()
                .filter_map(|child| child.into_token())
                .find(|token| token.kind() == SyntaxKind::TEXT)
                .map(|token| token.text().to_string())
        })
}

fn semantic_block_name(node: &SyntaxNode) -> Option<String> {
    match node.kind() {
        SyntaxKind::SOURCE_BLOCK => Some("src".into()),
        SyntaxKind::EXAMPLE_BLOCK => Some("example".into()),
        SyntaxKind::EXPORT_BLOCK => Some("export".into()),
        SyntaxKind::QUOTE_BLOCK => Some("quote".into()),
        SyntaxKind::VERSE_BLOCK => Some("verse".into()),
        SyntaxKind::CENTER_BLOCK => Some("center".into()),
        SyntaxKind::COMMENT_BLOCK => Some("comment".into()),
        SyntaxKind::DYN_BLOCK => Some("dynamic".into()),
        SyntaxKind::SPECIAL_BLOCK => block_name(node),
        _ => None,
    }
}

fn semantic_block_switches(node: &SyntaxNode) -> Option<String> {
    node.children()
        .find(|child| child.kind() == SyntaxKind::BLOCK_BEGIN)
        .into_iter()
        .flat_map(|begin| begin.children_with_tokens())
        .filter_map(NodeOrToken::into_token)
        .find(|token| token.kind() == SyntaxKind::SRC_BLOCK_SWITCHES)
        .map(|token| token.text().to_string())
}

fn parse_block_line_numbering(switches: &str) -> Option<BlockLineNumbering> {
    let mut tokens = split_block_switches(switches).into_iter().peekable();
    let mut numbering = None;

    while let Some(token) = tokens.next() {
        let mode = match token.as_str() {
            "-n" => BlockLineNumberMode::New,
            "+n" => BlockLineNumberMode::Continued,
            _ => continue,
        };
        let start = tokens.peek().and_then(|value| value.parse::<usize>().ok());
        if start.is_some() {
            tokens.next();
        }
        numbering = Some(BlockLineNumbering { mode, start });
    }

    numbering
}

fn parse_block_code_refs(value: &str, switches: Option<&str>) -> Vec<BlockCodeRef> {
    let label = switches
        .and_then(block_code_ref_label)
        .unwrap_or_else(default_code_ref_label);

    value
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            code_ref_in_line(line, &label).map(|(name, raw)| BlockCodeRef {
                line: index + 1,
                name,
                raw,
            })
        })
        .collect()
}

fn block_code_ref_label(switches: &str) -> Option<CodeRefLabel> {
    let mut tokens = split_block_switches(switches).into_iter();

    while let Some(token) = tokens.next() {
        if token == "-l" {
            return tokens.next().and_then(CodeRefLabel::from_pattern);
        }
    }

    None
}

fn split_block_switches(switches: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = switches.chars().peekable();
    let mut in_quote = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                in_quote = !in_quote;
            }
            '\\' if in_quote => {
                if let Some(next) = chars.next() {
                    current.push(next);
                } else {
                    current.push(ch);
                }
            }
            ch if ch.is_whitespace() && !in_quote => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CodeRefLabel {
    prefix: String,
    suffix: String,
}

impl CodeRefLabel {
    fn from_pattern(pattern: String) -> Option<Self> {
        let (prefix, suffix) = pattern.split_once("%s")?;
        Some(Self {
            prefix: prefix.to_string(),
            suffix: suffix.to_string(),
        })
    }
}

fn default_code_ref_label() -> CodeRefLabel {
    CodeRefLabel {
        prefix: "(ref:".into(),
        suffix: ")".into(),
    }
}

fn code_ref_in_line(line: &str, label: &CodeRefLabel) -> Option<(String, String)> {
    line.match_indices(&label.prefix)
        .filter_map(|(start, _)| code_ref_at(line, start, label))
        .last()
}

fn code_ref_at(line: &str, start: usize, label: &CodeRefLabel) -> Option<(String, String)> {
    if start > 0 && !line[..start].chars().next_back()?.is_whitespace() {
        return None;
    }

    let name_start = start + label.prefix.len();
    let suffix_start = if label.suffix.is_empty() {
        name_start
            + line[name_start..]
                .char_indices()
                .find_map(|(offset, ch)| (!is_code_ref_name_char(ch)).then_some(offset))
                .unwrap_or(line.len() - name_start)
    } else {
        name_start + line[name_start..].find(&label.suffix)?
    };
    let name = &line[name_start..suffix_start];

    if name.is_empty() || !name.chars().all(is_code_ref_name_char) {
        return None;
    }

    let end = suffix_start + label.suffix.len();
    Some((name.to_string(), line[start..end].to_string()))
}

fn is_code_ref_name_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == ' '
}

#[derive(Default)]
struct TimestampMomentBuilder {
    year: Option<u16>,
    month: Option<u8>,
    day: Option<u8>,
    day_name: Option<String>,
    times: Vec<(u8, u8)>,
    pending_hour: Option<u8>,
}

impl TimestampMomentBuilder {
    fn is_empty(&self) -> bool {
        self.year.is_none()
            && self.month.is_none()
            && self.day.is_none()
            && self.day_name.is_none()
            && self.times.is_empty()
            && self.pending_hour.is_none()
    }

    fn to_moment(&self, time_index: usize) -> Option<TimestampMoment> {
        let (hour, minute) = self
            .times
            .get(time_index)
            .copied()
            .map(|(hour, minute)| (Some(hour), Some(minute)))
            .unwrap_or((None, None));

        Some(TimestampMoment {
            year: self.year?,
            month: self.month?,
            day: self.day?,
            day_name: self.day_name.clone(),
            hour,
            minute,
        })
    }
}

fn timestamp_moment_range(
    node: &SyntaxNode,
    is_range: bool,
) -> (Option<TimestampMoment>, Option<TimestampMoment>) {
    let moments = timestamp_moment_builders(node);
    let start = moments.first().and_then(|moment| moment.to_moment(0));
    let end = if is_range {
        if moments.len() > 1 {
            moments.last().and_then(|moment| moment.to_moment(0))
        } else {
            moments.first().and_then(|moment| moment.to_moment(1))
        }
    } else {
        None
    };

    (start, end)
}

fn timestamp_moment_builders(node: &SyntaxNode) -> Vec<TimestampMomentBuilder> {
    let mut moments = Vec::new();
    let mut current = TimestampMomentBuilder::default();

    for token in node
        .children_with_tokens()
        .filter_map(|element| element.into_token())
    {
        match token.kind() {
            SyntaxKind::TIMESTAMP_YEAR => {
                if !current.is_empty() {
                    moments.push(current);
                    current = TimestampMomentBuilder::default();
                }
                current.year = parse_token(token.text());
            }
            SyntaxKind::TIMESTAMP_MONTH => current.month = parse_token(token.text()),
            SyntaxKind::TIMESTAMP_DAY => current.day = parse_token(token.text()),
            SyntaxKind::TIMESTAMP_DAYNAME => current.day_name = Some(token.text().to_string()),
            SyntaxKind::TIMESTAMP_HOUR => current.pending_hour = parse_token(token.text()),
            SyntaxKind::TIMESTAMP_MINUTE => {
                if let (Some(hour), Some(minute)) =
                    (current.pending_hour.take(), parse_token(token.text()))
                {
                    current.times.push((hour, minute));
                }
            }
            _ => {}
        }
    }

    if !current.is_empty() {
        moments.push(current);
    }

    moments
}

fn timestamp_repeater(timestamp: &syntax_ast::Timestamp) -> Option<TimestampRepeater> {
    Some(TimestampRepeater {
        kind: match timestamp.repeater_type()? {
            syntax_ast::RepeaterType::Cumulate => RepeaterKind::Cumulate,
            syntax_ast::RepeaterType::CatchUp => RepeaterKind::CatchUp,
            syntax_ast::RepeaterType::Restart => RepeaterKind::Restart,
        },
        value: timestamp.repeater_value()?,
        unit: timestamp_time_unit(timestamp.repeater_unit()?),
    })
}

fn timestamp_warning(timestamp: &syntax_ast::Timestamp) -> Option<TimestampWarning> {
    Some(TimestampWarning {
        kind: match timestamp.warning_type()? {
            syntax_ast::DelayType::All => WarningKind::All,
            syntax_ast::DelayType::First => WarningKind::First,
        },
        value: timestamp.warning_value()?,
        unit: timestamp_time_unit(timestamp.warning_unit()?),
    })
}

fn timestamp_time_unit(unit: syntax_ast::TimeUnit) -> TimeUnit {
    match unit {
        syntax_ast::TimeUnit::Hour => TimeUnit::Hour,
        syntax_ast::TimeUnit::Day => TimeUnit::Day,
        syntax_ast::TimeUnit::Week => TimeUnit::Week,
        syntax_ast::TimeUnit::Month => TimeUnit::Month,
        syntax_ast::TimeUnit::Year => TimeUnit::Year,
    }
}

fn parse_token<T>(value: &str) -> Option<T>
where
    T: std::str::FromStr,
{
    value.parse().ok()
}

fn range_from_elements(elements: &[SyntaxElement]) -> Option<TextRange> {
    let start = elements.first()?.text_range().start();
    let end = elements.last()?.text_range().end();
    Some(TextRange::new(start, end))
}

fn strip_pair(value: &str) -> &str {
    value
        .char_indices()
        .nth(1)
        .and_then(|(start, _)| {
            value
                .char_indices()
                .last()
                .map(|(end, _)| &value[start..end])
        })
        .unwrap_or_default()
}

fn split_macro_args(args: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut escaped = false;

    for ch in args.chars() {
        if escaped {
            if ch != ',' && ch != '\\' {
                current.push('\\');
            }
            current.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == ',' {
            let value = current.trim();
            if !value.is_empty() {
                values.push(value.to_string());
            }
            current.clear();
        } else {
            current.push(ch);
        }
    }

    if escaped {
        current.push('\\');
    }

    let value = current.trim();
    if !value.is_empty() {
        values.push(value.to_string());
    }

    values
}

fn citation_style(head: &str) -> (String, String) {
    let Some(rest) = head.strip_prefix("cite/") else {
        return ("nil".into(), String::new());
    };
    let mut parts = rest.split('/');
    let style = parts.next().unwrap_or("nil").to_string();
    let variant = parts.collect::<Vec<_>>().join("/");
    (style, variant)
}

fn citation_key_range(reference: &str) -> Option<(usize, usize)> {
    let at = reference.find('@')?;
    let start = at + 1;
    let end = reference[start..]
        .find(char::is_whitespace)
        .map(|offset| start + offset)
        .unwrap_or(reference.len());

    (start < end).then_some((start, end))
}

fn collect_radio_targets(root: &SyntaxNode) -> Vec<String> {
    fn collect(node: &SyntaxNode, targets: &mut Vec<String>) {
        for child in node.children() {
            if child.kind() == SyntaxKind::RADIO_TARGET {
                let target = strip_wrapping(&child.to_string(), "<<<", ">>>");
                if !target.is_empty() {
                    targets.push(target);
                }
            }
            collect(&child, targets);
        }
    }

    let mut targets = Vec::new();
    collect(root, &mut targets);
    targets.sort_by(|left, right| right.len().cmp(&left.len()).then_with(|| left.cmp(right)));
    targets.dedup();
    targets
}

fn next_radio_link<'a>(
    value: &str,
    cursor: usize,
    targets: &'a [String],
) -> Option<(usize, usize, &'a str)> {
    let mut best: Option<(usize, usize, &'a str)> = None;

    for target in targets {
        for (relative_start, _) in value[cursor..].match_indices(target) {
            let start = cursor + relative_start;
            let end = start + target.len();
            if !is_radio_link_boundary(value, start, end) {
                continue;
            }

            let candidate = (start, end, target.as_str());
            if best.as_ref().is_none_or(|(best_start, best_end, _)| {
                start < *best_start || (start == *best_start && end > *best_end)
            }) {
                best = Some(candidate);
            }
            break;
        }
    }

    best
}

fn is_radio_link_boundary(value: &str, start: usize, end: usize) -> bool {
    let before = value[..start].chars().next_back();
    let after = value[end..].chars().next();
    !before.is_some_and(is_radio_link_word_char) && !after.is_some_and(is_radio_link_word_char)
}

fn is_radio_link_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '-'
}

fn text_range(start: usize, end: usize) -> TextRange {
    TextRange::new((start as u32).into(), (end as u32).into())
}

fn offset_range(range: TextRange, base: usize) -> TextRange {
    text_range(
        usize::from(range.start()) + base,
        usize::from(range.end()) + base,
    )
}

fn trimmed_range(value: &str) -> Option<(usize, usize)> {
    let start = value
        .char_indices()
        .find_map(|(idx, ch)| (!ch.is_whitespace()).then_some(idx))?;
    let end = value
        .char_indices()
        .rfind(|(_, ch)| !ch.is_whitespace())
        .map(|(idx, ch)| idx + ch.len_utf8())?;

    (start < end).then_some((start, end))
}

fn strip_wrapping(value: &str, prefix: &str, suffix: &str) -> String {
    value
        .strip_prefix(prefix)
        .and_then(|value| value.strip_suffix(suffix))
        .unwrap_or(value)
        .to_string()
}
