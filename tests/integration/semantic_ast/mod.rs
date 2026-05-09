use orgize::{
    ast::{
        AstMut, AstRef, ElementData, LinkTarget, MarkupKind, ObjectData, ParsedAst, RepeaterKind,
        SourcePosition, TimeUnit, TodoState, WarningKind,
    },
    config::UseSubSuperscript,
    Org, ParseConfig,
};

fn assert_clean_projection(doc: &ParsedAst) {
    assert!(
        doc.diagnostics.is_empty(),
        "unexpected diagnostics: {:#?}",
        doc.diagnostics
    );

    let unknowns = doc.fold(Vec::new(), |mut unknowns, node| {
        match node {
            AstRef::Element(element) => {
                if let ElementData::Unknown { kind, .. } = &element.data {
                    unknowns.push(format!("element:{kind}"));
                }
            }
            AstRef::Object(object) => {
                if let ObjectData::Unknown { kind, .. } = &object.data {
                    unknowns.push(format!("object:{kind}"));
                }
            }
            _ => {}
        }
        unknowns
    });

    assert!(
        unknowns.is_empty(),
        "unexpected semantic unknowns: {unknowns:#?}"
    );
}

mod annotations_map_and_fold_work_across_the_tree;
mod existing_html_traversal_still_uses_the_lossless_substrate;
mod html_export_preserves_citation_raw_text;
mod semantic_annotations_handle_parser_line_endings_and_utf8_columns;
mod semantic_ast_covers_current_lossless_projection_surface;
mod semantic_ast_keeps_affiliated_keywords_out_of_paragraph_objects;
mod semantic_ast_projection_and_bare_snapshot;
mod semantic_ast_projects_citations;
mod semantic_ast_projects_clean_clock_duration;
mod semantic_ast_projects_cloze_objects_with_metadata;
mod semantic_ast_projects_footnote_definition_label_and_body;
mod semantic_ast_projects_inline_babel_and_footnote_details;
mod semantic_ast_projects_link_metadata;
mod semantic_ast_projects_object_gap_repairs;
mod semantic_ast_projects_org_table_formulas;
mod semantic_ast_projects_table_el;
mod semantic_ast_projects_timestamp_metadata;
mod semantic_citation_affixes_respect_parse_config;
mod semantic_traversal_supports_exporter_and_indexer_shapes;
mod traversal_visits_annotation_bearing_metadata_nodes;
