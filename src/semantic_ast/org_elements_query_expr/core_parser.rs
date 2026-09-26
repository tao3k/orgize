//! Lower the Org-owned AOT expression grammar into query values.

use super::core_types::QueryExpr;
use gerbil_parser_rowan::{Parse, SyntaxNode, SyntaxToken};

#[path = "../../../languages/org/v1/modules/org-contract/generated/parser.rs"]
#[rustfmt::skip]
mod grammar;

pub(super) fn parse_query_expression_syntax(value: &str) -> Option<Parse> {
    gerbil_parser_rowan::parse(&grammar::LANGUAGE, value).ok()
}

pub(super) fn lower_root(parsed: &Parse) -> Option<Vec<QueryExpr>> {
    let root = parsed.syntax();
    (parsed.kind_name(root.kind()) == Some("ContractSource"))
        .then(|| lower_children(parsed, &root))?
}

fn lower_children(parsed: &Parse, node: &SyntaxNode) -> Option<Vec<QueryExpr>> {
    node.children()
        .map(|child| lower_expr(parsed, &child))
        .collect()
}

fn lower_expr(parsed: &Parse, node: &SyntaxNode) -> Option<QueryExpr> {
    match parsed.kind_name(node.kind())? {
        "ContractList" => lower_children(parsed, node).map(QueryExpr::List),
        "ContractAtom" => token_of_kind(parsed, node, "ContractAtomToken")
            .map(|token| QueryExpr::Atom(token.text().to_string())),
        "ContractString" => token_of_kind(parsed, node, "ContractStringToken")
            .and_then(|token| unquote_query_string(token.text()))
            .map(QueryExpr::String),
        _ => None,
    }
}

fn token_of_kind(parsed: &Parse, node: &SyntaxNode, kind: &str) -> Option<SyntaxToken> {
    node.children_with_tokens()
        .filter_map(|child| child.into_token())
        .find(|token| parsed.kind_name(token.kind()) == Some(kind))
}

pub(super) fn unquote_query_string(raw: &str) -> Option<String> {
    let body = raw.strip_prefix('"')?.strip_suffix('"')?;
    let mut value = String::new();
    let mut chars = body.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            let escaped = chars.next()?;
            value.push(match escaped {
                'n' => '\n',
                't' => '\t',
                '"' => '"',
                '\\' => '\\',
                other => other,
            });
        } else {
            value.push(ch);
        }
    }
    Some(value)
}

#[cfg(test)]
#[path = "../../../tests/unit/org_contract_generated_cst.rs"]
mod tests;
