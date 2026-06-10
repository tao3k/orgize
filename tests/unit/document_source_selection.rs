use std::path::PathBuf;

use crate::document::{SourceLineRange, SourceSelector, select_source};

#[test]
fn parses_selector_without_range() {
    let selector = SourceSelector::parse("notes.org").expect("selector should parse");

    assert_eq!(selector.path, PathBuf::from("notes.org"));
    assert_eq!(selector.range, None);
}

#[test]
fn parses_selector_with_inclusive_line_range() {
    let selector = SourceSelector::parse("notes.org:2-4").expect("selector should parse");

    assert_eq!(selector.path, PathBuf::from("notes.org"));
    assert_eq!(selector.range, Some(SourceLineRange::new(2, 4)));
}

#[test]
fn normalizes_reversed_selector_range() {
    let selector = SourceSelector::parse("notes.org:4-2").expect("selector should parse");

    assert_eq!(selector.range, Some(SourceLineRange::new(4, 4)));
}

#[test]
fn selects_source_with_inclusive_range() {
    let source = "one\ntwo\nthree\nfour\n";

    assert_eq!(
        select_source(source, Some(SourceLineRange::new(2, 3))),
        "two\nthree\n"
    );
}
