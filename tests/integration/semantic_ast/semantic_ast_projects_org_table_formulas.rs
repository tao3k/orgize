use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{AstRef, ElementData, TableFormulaReferenceKind},
    Org,
};

#[test]
fn semantic_ast_projects_org_table_formulas() {
    let doc = Org::parse(
        "| a | b |\n#+TBLFM: $2=vsum(@2..@4);%.1f::$1=remote(other,$2)\n#+tblfm: @2=$3\n",
    )
    .document();

    assert_clean_projection(&doc);
    let table = match &doc.children[0].data {
        ElementData::Table(table) => table,
        other => panic!("expected org table, got {other:#?}"),
    };

    assert_eq!(table.rows.len(), 1);
    assert_eq!(table.formulas.len(), 2);
    assert_eq!(table.formulas[0].key, "TBLFM");
    assert_eq!(
        table.formulas[0].value,
        " $2=vsum(@2..@4);%.1f::$1=remote(other,$2)"
    );
    assert_eq!(table.formulas[1].key, "tblfm");
    assert_eq!(table.formulas[1].value, " @2=$3");
    assert_eq!(table.parsed_formulas.len(), 2);
    assert_eq!(table.parsed_formulas[0].assignments.len(), 2);
    assert_eq!(table.parsed_formulas[0].assignments[0].lhs, "$2");
    assert_eq!(table.parsed_formulas[0].assignments[0].rhs, "vsum(@2..@4)");
    assert_eq!(table.parsed_formulas[0].assignments[0].flags, ["%.1f"]);
    assert_eq!(
        table.parsed_formulas[0].assignments[0].references[0].kind,
        TableFormulaReferenceKind::Field
    );
    assert_eq!(
        table.parsed_formulas[0].assignments[1].references[1].kind,
        TableFormulaReferenceKind::Remote
    );

    let formula_keyword_count = doc.fold(0usize, |count, node| match node {
        AstRef::Keyword(keyword) if keyword.key.eq_ignore_ascii_case("TBLFM") => count + 1,
        _ => count,
    });
    assert_eq!(formula_keyword_count, 2);
}
