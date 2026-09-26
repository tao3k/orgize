use super::{lower_root, parse_query_expression_syntax};
use crate::ast::org_elements_query_expr::core_types::QueryExpr;

#[test]
fn generated_cst_preserves_comments_utf8_and_adjacent_strings() {
    let source = "(query π\u{a0}\"one\"\"two\") ; note\n";
    let parsed = parse_query_expression_syntax(source).expect("generated grammar accepts source");
    assert_eq!(parsed.syntax().to_string(), source);
    assert_eq!(
        parsed.kind_name(parsed.syntax().kind()),
        Some("ContractSource")
    );
    assert_eq!(
        lower_root(&parsed),
        Some(vec![QueryExpr::List(vec![
            QueryExpr::Atom("query".into()),
            QueryExpr::Atom("π".into()),
            QueryExpr::String("one".into()),
            QueryExpr::String("two".into()),
        ])])
    );
}

#[test]
fn generated_cst_rejects_unbalanced_and_unclosed_expressions() {
    for source in [")", "(query", "\"unclosed"] {
        assert!(parse_query_expression_syntax(source).is_none(), "{source}");
    }
}
