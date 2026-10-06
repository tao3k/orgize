use super::{
    parse_contract_expression_values, parse_query_expression_syntax, parse_query_expression_values,
};
use crate::ast::org_elements_query_expr::core_types::QueryExpr;

pub(crate) const NATIVE_CASES: &[(&str, fn())] = &[
    (
        "native_contract_composition_normalizes_bindings",
        native_contract_composition_normalizes_bindings,
    ),
    (
        "native_cst_preserves_comments_utf8_and_adjacent_strings",
        native_cst_preserves_comments_utf8_and_adjacent_strings,
    ),
    (
        "native_cst_rejects_unbalanced_and_unclosed_expressions",
        native_cst_rejects_unbalanced_and_unclosed_expressions,
    ),
];

fn native_contract_composition_normalizes_bindings() {
    assert_eq!(
        parse_contract_expression_values(
            "(let (($a (paragraph))) (let (($b (headline))) (assert exists (paragraph))))"
        ),
        parse_query_expression_values(
            "(let ((a (paragraph)) (b (headline)))) (assert exists (paragraph))"
        )
    );
    assert!(
        parse_contract_expression_values(
            "(let (($a (paragraph))) (let ((a (headline))) (assert exists (paragraph))))"
        )
        .is_none()
    );
}

fn native_cst_preserves_comments_utf8_and_adjacent_strings() {
    let source = "(query π\u{a0}\"one\"\"two\") ; note\n";
    let parsed = parse_query_expression_syntax(source).expect("native grammar accepts source");
    assert_eq!(parsed.syntax().to_string(), source);
    assert_eq!(
        parsed.kind_name(parsed.syntax().kind()),
        Some("ContractSource")
    );
    assert_eq!(
        parse_query_expression_values(source),
        Some(vec![QueryExpr::List(vec![
            QueryExpr::Atom("query".into()),
            QueryExpr::Atom("π".into()),
            QueryExpr::String("one".into()),
            QueryExpr::String("two".into()),
        ])])
    );
}

fn native_cst_rejects_unbalanced_and_unclosed_expressions() {
    for source in [")", "(query", "\"unclosed"] {
        assert!(parse_query_expression_syntax(source).is_none(), "{source}");
        assert!(parse_query_expression_values(source).is_none(), "{source}");
    }
    assert_eq!(
        parse_query_expression_values("(() \"π\\n\\t\\\"\\\\\\q\")"),
        Some(vec![QueryExpr::List(vec![
            QueryExpr::List(vec![]),
            QueryExpr::String("π\n\t\"\\q".into())
        ])])
    );
}
