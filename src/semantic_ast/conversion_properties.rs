//! Property, planning, clock, and keyword conversion helpers.

use super::conversion::Converter;
use super::settings::{is_parsed_keyword, keyword_attributes};
use super::{Clock, Keyword, Object, OrgDuration, ParsedAnnotation, Planning, Property};
use crate::{
    syntax::{SyntaxKind, SyntaxNode},
    syntax_ast,
};
use rowan::ast::AstNode;

impl<'a> Converter<'a> {
    pub(super) fn affiliated_keywords(
        &mut self,
        node: &SyntaxNode,
    ) -> Vec<Keyword<ParsedAnnotation>> {
        node.children()
            .take_while(|child| child.kind() == SyntaxKind::AFFILIATED_KEYWORD)
            .map(|child| self.keyword(&child, true))
            .collect()
    }

    pub(super) fn keyword(
        &mut self,
        node: &SyntaxNode,
        affiliated: bool,
    ) -> Keyword<ParsedAnnotation> {
        if affiliated {
            let syntax = syntax_ast::AffiliatedKeyword::cast(node.clone()).expect("keyword node");
            let key = syntax.key().to_string();
            let value = syntax.value().map(|x| x.to_string()).unwrap_or_default();
            Keyword {
                ann: self.node_ann(node),
                key: key.clone(),
                optional: syntax.optional().map(|x| x.to_string()),
                parsed: self.keyword_parsed_objects(node, &key, &value),
                attributes: keyword_attributes(&key, &value),
                value,
            }
        } else {
            let syntax = syntax_ast::SyntaxKeyword::cast(node.clone());
            if let Some(syntax) = syntax {
                let key = syntax.key().to_string();
                let value = syntax.value().to_string();
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

    pub(super) fn keyword_parsed_objects(
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
            .map_or(node_start, |position| node_start + position);
        self.objects_from_raw(value, value_start)
    }

    pub(super) fn properties(&self, node: &SyntaxNode) -> Vec<Property<ParsedAnnotation>> {
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

    pub(super) fn planning(&self, node: &SyntaxNode) -> Planning {
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

    pub(super) fn clock(&self, node: &SyntaxNode) -> Clock {
        let syntax = syntax_ast::SyntaxClock::cast(node.clone()).expect("clock node");
        let value = node
            .children()
            .find_map(|child| self.timestamp_node(&child));

        Clock {
            value,
            duration: syntax.duration().map(|token| token.to_string()),
            parsed_duration: syntax
                .duration()
                .and_then(|token| OrgDuration::parse(token.to_string())),
            raw: node.to_string(),
        }
    }
}
