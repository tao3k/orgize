//! Org AST traversal and high-level semantic element conversion.

use super::conversion::Converter;
use super::headline_metadata::{
    headline_level, headline_priority, headline_raw_title, headline_tags, headline_todo,
};
use super::preprocessing::{include_directive, macro_definition};
use super::prescan::{SemanticPrescan, collect_document_keyword};
use super::targets::collect_target_node;
use super::{
    Diagnostic, DiagnosticKind, Element, ElementData, Inlinetask, InlinetaskEnd, ParsedAnnotation,
    Priority, Section, TodoKeyword, TodoState, UnsupportedSyntaxKind,
};
use crate::{
    syntax::{SyntaxKind, SyntaxNode},
    syntax_ast,
};
use rowan::ast::AstNode;

impl<'a> Converter<'a> {
    pub(super) fn semantic_prescan(&mut self, root: &SyntaxNode) -> SemanticPrescan {
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

    pub(super) fn section(&mut self, node: &SyntaxNode) -> Section<ParsedAnnotation> {
        let syntax = syntax_ast::Headline::cast(node.clone()).expect("headline node");
        let properties = syntax
            .properties()
            .map(|drawer| self.properties(&drawer.syntax))
            .unwrap_or_default();
        let anchor = properties
            .iter()
            .find(|property| property.key.eq_ignore_ascii_case("CUSTOM_ID"))
            .map(|property| property.value.clone());
        let planning = syntax
            .planning()
            .map(|planning| self.planning(&planning.syntax))
            .unwrap_or_default();
        let body_section = syntax.section();
        let body_ann = body_section
            .as_ref()
            .map(|section| self.node_ann(&section.syntax));
        let children = body_section
            .as_ref()
            .map(|section| self.elements_from_container(&section.syntax))
            .unwrap_or_default();
        let subsections = node
            .children()
            .filter(|child| child.kind() == SyntaxKind::HEADLINE)
            .map(|child| self.section(&child))
            .collect();
        let todo = syntax.todo_keyword().map(|name| TodoKeyword {
            state: match syntax.todo_type() {
                Some(syntax_ast::TodoType::Done) => TodoState::Done,
                _ => TodoState::Todo,
            },
            name: name.to_string(),
        });
        let title = syntax.title().collect::<Vec<_>>();

        Section {
            ann: self.node_ann(node),
            body_ann,
            level: syntax.level(),
            properties,
            effective_properties: Vec::new(),
            archive: Default::default(),
            attachment: Default::default(),
            todo,
            is_comment: syntax.is_commented(),
            priority: Priority::from_cookie(syntax.priority().map(|x| x.to_string())),
            title: self.objects_from_elements(title),
            raw_title: syntax.title_raw(),
            anchor,
            tags: syntax.tags().map(|x| x.to_string()).collect(),
            effective_tags: syntax.tags().map(|x| x.to_string()).collect(),
            planning,
            children,
            subsections,
        }
    }

    pub(super) fn inlinetask(&mut self, node: &SyntaxNode) -> Inlinetask<ParsedAnnotation> {
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

    pub(super) fn elements_from_container(
        &mut self,
        node: &SyntaxNode,
    ) -> Vec<Element<ParsedAnnotation>> {
        node.children()
            .filter_map(|child| self.element(&child))
            .collect()
    }

    pub(super) fn element(&mut self, node: &SyntaxNode) -> Option<Element<ParsedAnnotation>> {
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
            SyntaxKind::DIARY_SEXP => ElementData::DiarySexp(node.to_string()),
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
}
