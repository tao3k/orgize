use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{
        SourceBlockTangleCommentsMode, SourceBlockTangleNowebMode, SourceTangleOptions,
        SourceTangleSkipReason, TableFormulaReferenceKind,
    },
};

const SOURCE: &str = include_str!("../../fixtures/semantic_ast/tangle-and-table-formulas.org");

#[test]
fn semantic_ast_projects_safe_tangle_plan_and_table_formula_records() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let tangle = doc.source_tangle_plan(&SourceTangleOptions::with_default_stem("notebook"));
    assert_eq!(tangle.files.len(), 2);
    assert!(tangle.files.iter().any(|file| {
        file.target == "src/lib.rs"
            && file.blocks.len() == 1
            && file.blocks[0].name.as_deref() == Some("explicit-rust")
            && file.blocks[0].tangle.mkdirp.enabled
            && file.blocks[0].tangle.comments.mode == SourceBlockTangleCommentsMode::Both
            && file.blocks[0].tangle.shebang.as_deref() == Some("#!/usr/bin/env rust-script")
            && file.blocks[0].tangle.noweb.mode == SourceBlockTangleNowebMode::Expand
    }));
    assert!(
        tangle
            .files
            .iter()
            .any(|file| file.target == "notebook.py" && file.blocks.len() == 1)
    );
    assert!(
        tangle
            .skipped
            .iter()
            .any(|skip| skip.reason == SourceTangleSkipReason::Disabled)
    );
    assert!(
        tangle
            .skipped
            .iter()
            .any(|skip| skip.reason == SourceTangleSkipReason::InlineSource)
    );

    let formula_records = doc.table_formula_records();
    assert_eq!(formula_records.len(), 1);
    assert_eq!(formula_records[0].row_count, 3);
    assert_eq!(formula_records[0].column_count, 3);
    assert!(
        formula_records[0].formulas[0].assignments[0]
            .references
            .iter()
            .any(|reference| reference.kind == TableFormulaReferenceKind::Field)
    );
    assert!(
        formula_records[0].formulas[0].assignments[1]
            .references
            .iter()
            .any(|reference| reference.kind == TableFormulaReferenceKind::Remote)
    );

    insta::assert_debug_snapshot!("semantic_ast__semantic_tangle_plan", tangle);
    insta::assert_debug_snapshot!(
        "semantic_ast__semantic_table_formula_records",
        formula_records
    );
}
