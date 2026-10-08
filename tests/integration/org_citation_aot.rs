//! Citation Objects are Scheme-owned and source-backed in native navigation.

use orgize::{
    Org,
    ast::{ElementData, ObjectData},
};

fn malformed_citation_marker_reaches_the_owned_diagnostic() {
    let parsed = Org::parse("[cite:@ok; @].");
    let malformed = parsed
        .records()
        .iter()
        .find(|record| record.kind == "citation-malformed")
        .expect("Scheme-classified malformed citation segment");
    assert_eq!(malformed.field("text"), Some(" @"));
    assert!(
        parsed
            .document()
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "malformed citation segment")
    );
}

fn scheme_citation_header_aot_projects_style_and_variant() {
    for (head, style, variant) in [
        ("[cite:", "nil", ""),
        ("[cite/text:", "text", ""),
        ("[cite/noauthor/bare:", "noauthor", "bare"),
    ] {
        let source = format!("{head}@key]");
        let document = Org::parse(&source).document();
        let ElementData::Paragraph(objects) = &document.children[0].data else {
            panic!("citation paragraph");
        };
        let citation = objects
            .iter()
            .find_map(|object| match &object.data {
                ObjectData::Citation(citation) => Some(citation),
                _ => None,
            })
            .expect("native citation");
        assert_eq!(citation.style, style);
        assert_eq!(citation.variant, variant);
    }
}

fn public_ast_projects_scheme_citation_reference_fields() {
    let document = Org::parse("See [cite/text:see @doe2020 p. 42; cf. @roe2021].").document();
    assert!(document.diagnostics.is_empty());
    let Some(ElementData::Paragraph(objects)) = document.children.first().map(|child| &child.data)
    else {
        panic!("expected paragraph");
    };
    let citation = objects
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::Citation(citation) => Some(citation),
            _ => None,
        })
        .expect("citation object");
    assert_eq!(citation.style, "text");
    assert_eq!(citation.variant, "");
    assert_eq!(citation.references.len(), 2);
    assert_eq!(citation.references[0].id, "doe2020");
    assert_eq!(citation.references[1].id, "roe2021");
}

fn scheme_declared_citations_project_into_native_index_and_graph() {
    check_org_aot_element!("[cite:@doe2020]\n", "citation-reference", "key" => "doe2020");
    check_org_aot_element!(
        "[cite:@key\n[cite:@next]\n",
        "citation-reference",
        "key" => "next"
    );
    for invalid in [
        "[cite:no key]\n",
        "[cite/:@key]\n",
        "[cite:\\@key]\n",
        "[cite:@ ]\n",
        "[cite:[@key]]\n",
    ] {
        let document = orgize::org_aot::parse_org_aot(invalid)
            .expect("invalid citation remains lossless text");
        assert!(
            !document
                .records()
                .iter()
                .any(|record| record.kind == "citation")
        );
        assert_eq!(document.syntax().to_string(), invalid);
    }
}

fn citation_references_keep_distinct_keys_and_source_fields() {
    let source = "[cite/text:see @doe2020 p. 42; cf. @roe2021]\n";
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("Scheme-owned references compile through native navigation");
    let references: Vec<_> = document
        .records()
        .iter()
        .filter(|record| record.kind == "citation-reference")
        .collect();
    assert_eq!(references.len(), 2);
    assert_eq!(references[0].field("prefix"), Some("see "));
    assert_eq!(references[0].field("key"), Some("doe2020"));
    assert_eq!(references[0].field("suffix"), Some(" p. 42"));
    assert_eq!(references[1].field("prefix"), Some(" cf. "));
    assert_eq!(references[1].field("key"), Some("roe2021"));
    assert_eq!(document.syntax().to_string(), source);
}

fn citation_global_prefix_and_suffix_project_from_scheme() {
    let source = "[cite:see;@key;and]\n";
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("Scheme-owned citation retains global adornments");
    let citation = document
        .records()
        .iter()
        .find(|record| record.kind == "citation")
        .expect("citation Element record");
    assert_eq!(citation.field("global-prefix"), Some("see"));
    assert_eq!(citation.field("global-suffix"), Some("and"));
    assert_eq!(document.syntax().to_string(), source);
}

fn citation_affix_graph_keeps_source_ranges_and_nested_objects() {
    let source = "See [cite/text:global *prefix* ; see /also/ @doe2020 p. *42*; cf. @roe2021; global suffix].";
    let document = orgize::org_aot::parse_org_aot(source).expect("Scheme citation graph");
    let citation = document
        .records()
        .iter()
        .find(|record| record.kind == "citation")
        .expect("citation Object");
    assert_eq!(citation.field("head"), Some("[cite/text:"));
    assert_eq!(citation.field("global-prefix"), Some("global *prefix* "));
    assert_eq!(citation.field("global-suffix"), Some(" global suffix"));
    assert!(
        citation
            .child_ids
            .iter()
            .map(|&id| &document.records()[id])
            .any(|child| child.kind == "bold")
    );
}

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[
    (
        "org_citation_aot::malformed_citation_marker_reaches_the_owned_diagnostic",
        malformed_citation_marker_reaches_the_owned_diagnostic,
    ),
    (
        "org_citation_aot::scheme_citation_header_aot_projects_style_and_variant",
        scheme_citation_header_aot_projects_style_and_variant,
    ),
    (
        "org_citation_aot::public_ast_projects_scheme_citation_reference_fields",
        public_ast_projects_scheme_citation_reference_fields,
    ),
    (
        "org_citation_aot::scheme_declared_citations_project_into_native_index_and_graph",
        scheme_declared_citations_project_into_native_index_and_graph,
    ),
    (
        "org_citation_aot::citation_references_keep_distinct_keys_and_source_fields",
        citation_references_keep_distinct_keys_and_source_fields,
    ),
    (
        "org_citation_aot::citation_global_prefix_and_suffix_project_from_scheme",
        citation_global_prefix_and_suffix_project_from_scheme,
    ),
    (
        "org_citation_aot::citation_affix_graph_keeps_source_ranges_and_nested_objects",
        citation_affix_graph_keeps_source_ranges_and_nested_objects,
    ),
];
