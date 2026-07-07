use std::path::Path;

use crate::document::{DocumentLanguage, document_query_lexical_prefilter_miss};

#[test]
fn document_query_lexical_prefilter_skips_absent_literal_term() {
    assert!(document_query_lexical_prefilter_miss(
        DocumentLanguage::Org,
        Path::new("plan.org"),
        "* Plan\n\nNo matching token here.\n",
        &["aspOrg".to_string()],
        &[],
    ));
}

#[test]
fn document_query_lexical_prefilter_keeps_parser_only_terms() {
    assert!(!document_query_lexical_prefilter_miss(
        DocumentLanguage::Org,
        Path::new("plan.org"),
        "* Plan\n\nNo literal heading token here.\n",
        &["heading".to_string()],
        &[],
    ));
}

#[test]
fn document_query_lexical_prefilter_keeps_markdown_source_kind_terms() {
    assert!(!document_query_lexical_prefilter_miss(
        DocumentLanguage::Markdown,
        Path::new("guide.md"),
        "# Guide\n\nBody\n",
        &["nodevalue::heading".to_string(), "codeblock".to_string()],
        &[],
    ));
}
