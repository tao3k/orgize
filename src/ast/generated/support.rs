//! Shared helpers for generated typed syntax wrappers.

use crate::syntax::{SyntaxKind, SyntaxNode};
use rowan::ast::AstNode;

use super::AffiliatedKeyword;

pub(crate) fn affiliated_keyword(
    node: &SyntaxNode,
    filter: impl Fn(&str) -> bool,
) -> Option<AffiliatedKeyword> {
    node.children()
        .take_while(|node| node.kind() == SyntaxKind::AFFILIATED_KEYWORD)
        .filter_map(AffiliatedKeyword::cast)
        .find(|keyword| {
            let key = keyword.key();
            filter(key.as_ref()) || filter(&key.to_ascii_uppercase())
        })
}
