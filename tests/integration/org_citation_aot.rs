//! Citation Objects are Scheme-owned and source-backed in Rowan.

use orgize::{
    Org,
    ast::{ElementData, ObjectData},
};

include!(concat!(env!("OUT_DIR"), "/citation_style.rs"));
include!(concat!(env!("OUT_DIR"), "/citation_variant.rs"));

#[test]
fn scheme_citation_header_aot_projects_style_and_variant() {
    macro_rules! check_citation_header_aot {
        ($($head:expr => $style:expr, $variant:expr),+ $(,)?) => {
            $(
                assert_eq!(citation_style($head), $style);
                assert_eq!(citation_variant($head), $variant);
            )+
        };
    }
    check_citation_header_aot!(
        "[cite:" => "nil", "",
        "[cite/text:" => "text", "",
        "[cite/noauthor/bare:" => "noauthor", "bare",
    );
}

#[test]
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

#[test]
fn scheme_declared_citations_project_into_rowan_and_graph() {
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

#[test]
fn citation_references_keep_distinct_keys_and_source_fields() {
    let source = "[cite/text:see @doe2020 p. 42; cf. @roe2021]\n";
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("Scheme-owned references compile through Rowan");
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

#[test]
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
