//! Headline-shaped metadata extraction used by section and inlinetask projection.

use crate::syntax::{SyntaxKind, SyntaxNode};

use super::{TodoKeyword, TodoState};

pub(super) fn headline_level(node: &SyntaxNode) -> usize {
    node.children_with_tokens()
        .find(|element| element.kind() == SyntaxKind::HEADLINE_STARS)
        .and_then(|element| element.as_token().map(|token| token.text().len()))
        .unwrap_or_default()
}

pub(super) fn headline_todo(node: &SyntaxNode) -> Option<TodoKeyword> {
    node.children_with_tokens()
        .find_map(|element| match element.kind() {
            SyntaxKind::HEADLINE_KEYWORD_TODO | SyntaxKind::HEADLINE_KEYWORD_DONE => {
                let token = element.as_token()?;
                Some(TodoKeyword {
                    state: if element.kind() == SyntaxKind::HEADLINE_KEYWORD_DONE {
                        TodoState::Done
                    } else {
                        TodoState::Todo
                    },
                    name: token.text().to_string(),
                })
            }
            _ => None,
        })
}

pub(super) fn headline_priority(node: &SyntaxNode) -> Option<String> {
    let raw = node
        .children()
        .find(|child| child.kind() == SyntaxKind::HEADLINE_PRIORITY)?
        .to_string();
    let value = raw.strip_prefix("[#")?.strip_suffix(']')?;
    Some(value.to_string())
}

pub(super) fn headline_raw_title(node: &SyntaxNode) -> String {
    node.children()
        .find(|child| child.kind() == SyntaxKind::HEADLINE_TITLE)
        .map(|child| child.to_string())
        .unwrap_or_default()
}

pub(super) fn headline_tags(node: &SyntaxNode) -> Vec<String> {
    let Some(tags) = node
        .children()
        .find(|child| child.kind() == SyntaxKind::HEADLINE_TAGS)
    else {
        return Vec::new();
    };

    tags.children_with_tokens()
        .filter_map(|element| {
            (element.kind() == SyntaxKind::TEXT)
                .then(|| element.as_token().map(|token| token.text().to_string()))
                .flatten()
        })
        .collect()
}
