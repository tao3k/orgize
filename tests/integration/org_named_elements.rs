//! Scheme-owned named Org Elements projected through Rowan.

use orgize::org_aot::parse_org_aot;

#[test]
fn org_scheme_named_special_block_projects_name_and_recursive_body() {
    let source = "#+BEGIN_NOTE\ntext\n#+END_note\n";
    let document = parse_org_aot(source).expect("Scheme named block reaches Rowan");
    assert_eq!(document.syntax().to_string(), source);
    let special = document
        .records()
        .iter()
        .find(|record| record.kind == "special-block")
        .expect("named block projects as a special-block Element");
    assert_eq!(special.field("name"), Some("NOTE"));
    assert!(
        document
            .records()
            .iter()
            .any(|record| { record.kind == "paragraph" && record.parent_id == Some(special.id) })
    );

    let mismatched = parse_org_aot("#+BEGIN_NOTE\ntext\n#+END_OTHER\n")
        .expect("unclosed named block recovers as text");
    assert!(
        mismatched
            .records()
            .iter()
            .all(|record| record.kind != "special-block")
    );

    let nested_source = "#+BEGIN_OUTER\n#+BEGIN_INNER\nx\n#+END_inner\n#+END_outer\n";
    let nested = parse_org_aot(nested_source).expect("nested named blocks reach Rowan");
    assert_eq!(nested.syntax().to_string(), nested_source);
    let blocks: Vec<_> = nested
        .records()
        .iter()
        .filter(|record| record.kind == "special-block")
        .collect();
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].field("name"), Some("OUTER"));
    assert_eq!(blocks[1].field("name"), Some("INNER"));
    assert_eq!(blocks[1].parent_id, Some(blocks[0].id));
}

#[test]
fn org_scheme_latex_environment_preserves_named_opaque_body() {
    let source = "\\begin{align*}\nx\n\\end{align*}\n";
    let document = parse_org_aot(source).expect("Scheme LaTeX environment reaches Rowan");
    assert_eq!(document.syntax().to_string(), source);
    let environment = document
        .records()
        .iter()
        .find(|record| record.kind == "latex-environment")
        .expect("LaTeX environment projects as an Element");
    assert_eq!(environment.field("name"), Some("align*"));
    assert_eq!(environment.field("body"), Some("\nx\n"));

    let same_line =
        parse_org_aot("\\begin{a}\\end{a}").expect("same-line LaTeX environment reaches Rowan");
    assert_eq!(same_line.syntax().to_string(), "\\begin{a}\\end{a}");
    assert!(
        same_line.records().iter().any(|record| {
            record.kind == "latex-environment" && record.field("name") == Some("a")
        })
    );

    let mismatch = parse_org_aot("\\begin{a}\nbody\n\\end{b}\n")
        .expect("mismatched environment remains lossless text");
    assert!(
        mismatch
            .records()
            .iter()
            .all(|record| record.kind != "latex-environment")
    );
}

#[test]
fn org_scheme_named_children_stop_at_their_parent_boundary() {
    for source in [
        "#+BEGIN_OUTER\n#+BEGIN_INNER\n#+END_outer\n#+END_inner\n",
        "#+BEGIN_NOTE\n\\begin{a}\n#+END_NOTE\n\\end{a}\n",
    ] {
        let document = parse_org_aot(source).expect("named parent boundary reaches Rowan");
        assert_eq!(document.syntax().to_string(), source);
        assert_eq!(
            document
                .records()
                .iter()
                .filter(|record| record.kind == "special-block")
                .count(),
            1
        );
        assert!(
            document
                .records()
                .iter()
                .all(|record| record.kind != "latex-environment")
        );
    }
}
