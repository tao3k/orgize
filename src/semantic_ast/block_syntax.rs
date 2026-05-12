//! Lossless block syntax helpers used by semantic projection.

use rowan::NodeOrToken;

use crate::syntax::{SyntaxKind, SyntaxNode};

pub(super) struct BlockParts {
    pub(super) begin: Option<SyntaxNode>,
    pub(super) content: Option<SyntaxNode>,
}

pub(super) fn block_parts(node: &SyntaxNode) -> BlockParts {
    let mut begin = None;
    let mut content = None;

    for child in node.children() {
        match child.kind() {
            SyntaxKind::BLOCK_BEGIN => begin = Some(child),
            SyntaxKind::BLOCK_CONTENT => content = Some(child),
            _ => {}
        }

        if begin.is_some() && content.is_some() {
            break;
        }
    }

    BlockParts { begin, content }
}

pub(super) fn block_name_from_begin(begin: Option<&SyntaxNode>) -> Option<String> {
    begin.and_then(|begin| {
        begin
            .children_with_tokens()
            .filter_map(|child| child.into_token())
            .filter(|token| token.kind() == SyntaxKind::TEXT)
            .nth(1)
            .map(|token| token.text().to_string())
    })
}

pub(super) fn semantic_block_name(kind: SyntaxKind, begin: Option<&SyntaxNode>) -> Option<String> {
    match kind {
        SyntaxKind::SOURCE_BLOCK => Some("src".into()),
        SyntaxKind::EXAMPLE_BLOCK => Some("example".into()),
        SyntaxKind::EXPORT_BLOCK => Some("export".into()),
        SyntaxKind::QUOTE_BLOCK => Some("quote".into()),
        SyntaxKind::VERSE_BLOCK => Some("verse".into()),
        SyntaxKind::CENTER_BLOCK => Some("center".into()),
        SyntaxKind::COMMENT_BLOCK => Some("comment".into()),
        SyntaxKind::DYN_BLOCK => Some("dynamic".into()),
        SyntaxKind::SPECIAL_BLOCK => block_name_from_begin(begin),
        _ => None,
    }
}

pub(super) fn block_switches_from_begin(begin: Option<&SyntaxNode>) -> Option<String> {
    begin
        .into_iter()
        .flat_map(SyntaxNode::children_with_tokens)
        .filter_map(NodeOrToken::into_token)
        .find(|token| token.kind() == SyntaxKind::SRC_BLOCK_SWITCHES)
        .map(|token| token.text().to_string())
}
