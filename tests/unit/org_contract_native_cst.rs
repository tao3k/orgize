use super::{parse_query_expression_syntax, parse_query_expression_values};
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
    let registry = |source| {
        let document = crate::Org::parse(&format!(
            "* Contract\n:PROPERTIES:\n:CONTRACT_ID: native.composition\n:END:\n** Assertion\n:PROPERTIES:\n:ASSERT_ID: native.bindings\n:END:\n#+begin_src org-contract\n{source}\n#+end_src\n"
        )).document();
        crate::ast::validate_contract_source(&document, None)
    };
    let valid =
        registry("(let (($a (paragraph))) (let (($b (headline))) (assert exists (paragraph))))");
    assert!(valid.is_valid(), "{:?}", valid.diagnostics);
    assert_eq!(
        valid.registry.contracts[0].assertions[0]
            .bindings
            .iter()
            .map(|binding| binding.name.as_str())
            .collect::<Vec<_>>(),
        ["a", "b"],
    );
    let invalid =
        registry("(let (($a (paragraph))) (let ((a (headline))) (assert exists (paragraph))))");
    assert!(!invalid.is_valid());
    assert!(invalid.registry.contracts[0].assertions.is_empty());
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
