//! The public Org facade must be the Scheme-generated parse product.

use orgize::{Org, ParseConfig, org_aot::OrgAotDocument};

fn public_parse_returns_the_scheme_aot_document() {
    let source = "* TODO One\nBody\n";
    let document: OrgAotDocument = Org::parse(source);
    assert_eq!(document.to_org(), source);
    assert_eq!(document.receipt().language, "org-mode");
    assert_eq!(
        document.receipt().parser_digest,
        Some(orgize::org_aot::org_event_parser_digest())
    );
    assert_eq!(document.document().sections[0].level, 1);
}

fn configured_public_parse_keeps_the_same_aot_boundary() {
    let config = ParseConfig {
        todo_keywords: (vec!["TASK".to_owned()], Vec::new()),
        ..ParseConfig::default()
    };
    let document: OrgAotDocument = config.parse("* TASK One\n");
    let headline = document
        .records()
        .iter()
        .find(|record| record.kind == "headline")
        .expect("Scheme headline record");
    assert_eq!(document.headline_todo_type(headline.id), Some("todo"));
    assert_eq!(
        document.headline_todo_keyword(headline.id).as_deref(),
        Some("TASK")
    );
    assert_eq!(
        document.receipt().parser_digest,
        Some(orgize::org_aot::org_event_parser_digest())
    );
}

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[
    (
        "org_public_aot_boundary::public_parse_returns_the_scheme_aot_document",
        public_parse_returns_the_scheme_aot_document,
    ),
    (
        "org_public_aot_boundary::configured_public_parse_keeps_the_same_aot_boundary",
        configured_public_parse_keeps_the_same_aot_boundary,
    ),
];
