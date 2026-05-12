//! Document-local target collection for semantic internal link resolution.

use std::{borrow::Cow, collections::HashMap};

use rowan::{ast::AstNode, NodeOrToken};

use crate::{
    syntax::{SyntaxKind, SyntaxNode, SyntaxToken},
    syntax_ast,
};

use super::block_metadata::parse_block_code_refs;
use super::{ParsedAnnotation, TargetDefinition, TargetKind};

#[derive(Clone, Default)]
pub(super) struct TargetIndex {
    pub(super) definitions: Vec<TargetDefinition<ParsedAnnotation>>,
    keys: HashMap<String, Vec<usize>>,
    radio_targets: Vec<String>,
}

impl TargetIndex {
    fn push(&mut self, definition: TargetDefinition<ParsedAnnotation>) {
        if definition.key.is_empty() {
            return;
        }

        if definition.kind == TargetKind::RadioTarget {
            self.radio_targets.push(definition.value.clone());
        }

        let index = self.definitions.len();
        self.keys
            .entry(definition.key.clone())
            .or_default()
            .push(index);
        self.definitions.push(definition);
    }

    pub(super) fn radio_targets(&self) -> Vec<String> {
        let mut targets = self.radio_targets.clone();
        targets.sort_by(|left, right| right.len().cmp(&left.len()).then_with(|| left.cmp(right)));
        targets.dedup();
        targets
    }

    pub(super) fn resolve<'a>(&self, path: &'a str) -> TargetLookup<'a> {
        let key = normalized_target_key(path);
        let Some(indices) = self.keys.get(key.as_ref()) else {
            return TargetLookup::Missing { key };
        };

        if indices.len() == 1 {
            TargetLookup::Found {
                key: key.into_owned(),
            }
        } else {
            TargetLookup::Ambiguous {
                key: key.into_owned(),
                count: indices.len(),
            }
        }
    }
}

pub(super) enum TargetLookup<'a> {
    Found { key: String },
    Ambiguous { key: String, count: usize },
    Missing { key: Cow<'a, str> },
}

pub(super) fn collect_target_node(
    node: &SyntaxNode,
    index: &mut TargetIndex,
    node_ann: &impl Fn(&SyntaxNode) -> ParsedAnnotation,
    token_ann: &impl Fn(&SyntaxToken) -> ParsedAnnotation,
) {
    match node.kind() {
        SyntaxKind::HEADLINE => collect_headline_targets(node, index, node_ann, token_ann),
        SyntaxKind::TARGET => {
            let raw = node.to_string();
            let value = strip_wrapping(&raw, "<<", ">>");
            index.push(TargetDefinition {
                ann: node_ann(node),
                kind: TargetKind::Target,
                key: value.clone(),
                value,
                raw,
            });
        }
        SyntaxKind::RADIO_TARGET => {
            let raw = node.to_string();
            let value = strip_wrapping(&raw, "<<<", ">>>");
            index.push(TargetDefinition {
                ann: node_ann(node),
                kind: TargetKind::RadioTarget,
                key: value.clone(),
                value,
                raw,
            });
        }
        SyntaxKind::FN_DEF => {
            if let Some(label) = footnote_definition_label(node) {
                index.push(TargetDefinition {
                    ann: node_ann(node),
                    kind: TargetKind::FootnoteDefinition,
                    key: format!("fn:{label}"),
                    value: label,
                    raw: node.to_string(),
                });
            }
        }
        SyntaxKind::SOURCE_BLOCK | SyntaxKind::EXAMPLE_BLOCK => {
            collect_code_ref_targets(node, index, node_ann);
        }
        _ => {}
    }
}

pub(super) fn is_strict_internal_link_path(path: &str) -> bool {
    path.starts_with('*') || path.starts_with("fn:") || path.starts_with("coderef:")
}

fn collect_headline_targets(
    node: &SyntaxNode,
    index: &mut TargetIndex,
    node_ann: &impl Fn(&SyntaxNode) -> ParsedAnnotation,
    token_ann: &impl Fn(&SyntaxToken) -> ParsedAnnotation,
) {
    let legacy = syntax_ast::Headline::cast(node.clone()).expect("headline node");
    let title = legacy.title_raw().trim().to_string();
    if !title.is_empty() {
        index.push(TargetDefinition {
            ann: node_ann(node),
            kind: TargetKind::Headline,
            key: title.clone(),
            value: title.clone(),
            raw: title,
        });
    }

    if let Some(drawer) = legacy.properties() {
        for (key, value) in drawer.iter() {
            if key.eq_ignore_ascii_case("CUSTOM_ID") {
                let ann = token_ann(value.syntax());
                let value = value.to_string();
                if !value.is_empty() {
                    index.push(TargetDefinition {
                        ann,
                        kind: TargetKind::CustomId,
                        key: format!("#{value}"),
                        value: value.clone(),
                        raw: value,
                    });
                }
            } else if key.eq_ignore_ascii_case("ID") {
                let ann = token_ann(value.syntax());
                let value = value.to_string();
                if !value.is_empty() {
                    index.push(TargetDefinition {
                        ann,
                        kind: TargetKind::Id,
                        key: format!("id:{value}"),
                        value: value.clone(),
                        raw: value,
                    });
                }
            }
        }
    }
}

fn collect_code_ref_targets(
    node: &SyntaxNode,
    index: &mut TargetIndex,
    node_ann: &impl Fn(&SyntaxNode) -> ParsedAnnotation,
) {
    let switches = block_switches(node);
    let value = if let Some(source) = syntax_ast::SourceBlock::cast(node.clone()) {
        source.value()
    } else {
        node.children()
            .find(|child| child.kind() == SyntaxKind::BLOCK_CONTENT)
            .map(|child| child.to_string())
            .unwrap_or_default()
    };

    for code_ref in parse_block_code_refs(&value, switches.as_deref()) {
        index.push(TargetDefinition {
            ann: node_ann(node),
            kind: TargetKind::CodeRef,
            key: format!("coderef:{}", code_ref.name),
            value: code_ref.name,
            raw: code_ref.raw,
        });
    }
}

fn block_switches(node: &SyntaxNode) -> Option<String> {
    node.children()
        .find(|child| child.kind() == SyntaxKind::BLOCK_BEGIN)
        .into_iter()
        .flat_map(|begin| begin.children_with_tokens())
        .filter_map(NodeOrToken::into_token)
        .find(|token| token.kind() == SyntaxKind::SRC_BLOCK_SWITCHES)
        .map(|token| token.text().to_string())
}

fn normalized_target_key(path: &str) -> Cow<'_, str> {
    if let Some(id_path) = path.strip_prefix("id:") {
        return match id_path.split_once("::") {
            Some((id, _)) => Cow::Owned(format!("id:{id}")),
            None => Cow::Borrowed(path),
        };
    }

    path.strip_prefix('*')
        .map(str::trim)
        .map(Cow::Borrowed)
        .unwrap_or_else(|| Cow::Borrowed(path))
}

fn footnote_definition_label(node: &SyntaxNode) -> Option<String> {
    let mut saw_fn_prefix = false;
    let mut saw_label_colon = false;
    let mut label = String::new();

    for element in node.children_with_tokens() {
        match element.kind() {
            SyntaxKind::TEXT if !saw_fn_prefix => {
                saw_fn_prefix = true;
            }
            SyntaxKind::COLON if saw_fn_prefix && !saw_label_colon => {
                saw_label_colon = true;
            }
            SyntaxKind::R_BRACKET if saw_label_colon => break,
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

    (!label.is_empty()).then_some(label)
}

fn strip_wrapping(value: &str, prefix: &str, suffix: &str) -> String {
    value
        .strip_prefix(prefix)
        .and_then(|value| value.strip_suffix(suffix))
        .unwrap_or(value)
        .to_string()
}
