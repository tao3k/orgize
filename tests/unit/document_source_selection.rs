use std::path::PathBuf;

use crate::document::{SourceLineRange, SourceSelector, select_source};

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[
    (
        "parses_selector_without_range",
        parses_selector_without_range,
    ),
    (
        "parses_selector_with_inclusive_line_range",
        parses_selector_with_inclusive_line_range,
    ),
    (
        "normalizes_reversed_selector_range",
        normalizes_reversed_selector_range,
    ),
    ("parses_structural_selector", parses_structural_selector),
    (
        "rejects_line_range_for_structural_query_selector",
        rejects_line_range_for_structural_query_selector,
    ),
    (
        "selects_source_with_inclusive_range",
        selects_source_with_inclusive_range,
    ),
];

fn parses_selector_without_range() {
    let selector = SourceSelector::parse_direct_read("notes.org").expect("selector should parse");

    assert_eq!(selector.path, PathBuf::from("notes.org"));
    assert_eq!(selector.range, None);
}

fn parses_selector_with_inclusive_line_range() {
    let selector =
        SourceSelector::parse_direct_read("notes.org:2-4").expect("selector should parse");

    assert_eq!(selector.path, PathBuf::from("notes.org"));
    assert_eq!(selector.range, Some(SourceLineRange::new(2, 4)));
}

fn normalizes_reversed_selector_range() {
    let selector =
        SourceSelector::parse_direct_read("notes.org:4-2").expect("selector should parse");

    assert_eq!(selector.range, Some(SourceLineRange::new(4, 4)));
}

fn parses_structural_selector() {
    let selector = SourceSelector::parse_structural("org://notes.org#headline/heading/document[1]")
        .expect("selector should parse");

    assert_eq!(selector.path, PathBuf::from("notes.org"));
    assert_eq!(selector.range, None);
    assert_eq!(
        selector.structural_fragment.as_deref(),
        Some("headline/heading/document[1]")
    );
}

fn rejects_line_range_for_structural_query_selector() {
    let error = SourceSelector::parse_structural("notes.org:2-4").expect_err("line selector");

    assert!(error.contains("not structural"));
}

fn selects_source_with_inclusive_range() {
    let source = "one\ntwo\nthree\nfour\n";

    assert_eq!(
        select_source(source, Some(SourceLineRange::new(2, 3))),
        "two\nthree\n"
    );
}
