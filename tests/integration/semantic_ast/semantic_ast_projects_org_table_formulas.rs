use super::*;

#[test]
fn semantic_ast_projects_org_table_formulas() {
    let doc = Org::parse("| a |\n#+TBLFM: $1=$2\n#+tblfm: @2=$3\n").document();

    assert_clean_projection(&doc);
    let table = match &doc.children[0].data {
        ElementData::Table(table) => table,
        other => panic!("expected org table, got {other:#?}"),
    };

    assert_eq!(table.rows.len(), 1);
    assert_eq!(table.formulas.len(), 2);
    assert_eq!(table.formulas[0].key, "TBLFM");
    assert_eq!(table.formulas[0].value, " $1=$2");
    assert_eq!(table.formulas[1].key, "tblfm");
    assert_eq!(table.formulas[1].value, " @2=$3");

    let formula_keyword_count = doc.fold(0usize, |count, node| match node {
        AstRef::Keyword(keyword) if keyword.key.eq_ignore_ascii_case("TBLFM") => count + 1,
        _ => count,
    });
    assert_eq!(formula_keyword_count, 2);
}
