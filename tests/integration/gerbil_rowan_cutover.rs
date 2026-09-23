//! Behavioral admission cases for replacing the handwritten Org parser.
//!
//! These assertions deliberately exercise the public Rowan tree.  The Scheme
//! grammar and generated engine must satisfy them before the old parser can be
//! removed; a source-byte roundtrip alone would also admit an all-TEXT tree.

use orgize::rowan::ast::AstNode;
use orgize::{Org, SyntaxKind};

fn assert_kinds(source: &str, expected: &[SyntaxKind]) {
    let parsed = Org::parse(source);
    let document = parsed.syntax_document();
    let root = document.syntax();
    assert_eq!(root.to_string(), source);
    assert_eq!(root.kind(), SyntaxKind::DOCUMENT);

    let actual: Vec<_> = root.descendants().map(|node| node.kind()).collect();
    for kind in expected {
        assert!(actual.contains(kind), "missing {kind:?} in {actual:?}");
    }
}

#[test]
fn nested_headlines_and_section_boundaries() {
    assert_kinds(
        "preamble\n* Parent\nbody\n** Child\nmore\n* Sibling\n",
        &[
            SyntaxKind::SECTION,
            SyntaxKind::PARAGRAPH,
            SyntaxKind::HEADLINE,
            SyntaxKind::HEADLINE_TITLE,
        ],
    );
}

#[test]
#[ignore = "current section splitter treats a headline inside SRC as a real headline; required for Scheme cutover"]
fn block_contents_do_not_start_headlines() {
    let source = "#+begin_src text\n* not a headline\n#+end_src\n* Real\n";
    assert_kinds(source, &[SyntaxKind::SOURCE_BLOCK, SyntaxKind::HEADLINE]);
    let parsed = Org::parse(source);
    let document = parsed.syntax_document();
    assert_eq!(
        document
            .syntax()
            .descendants()
            .filter(|node| node.kind() == SyntaxKind::HEADLINE)
            .count(),
        1,
    );
}

#[test]
fn affiliated_keyword_and_nested_objects() {
    assert_kinds(
        "#+NAME: example\n#+CAPTION: A *bold* caption\n| a | b |\n\n* Link [[https://example.org][*bold*]]\n",
        &[
            SyntaxKind::AFFILIATED_KEYWORD,
            SyntaxKind::ORG_TABLE,
            SyntaxKind::HEADLINE,
            SyntaxKind::LINK,
            SyntaxKind::BOLD,
        ],
    );
}

#[test]
fn malformed_constructs_remain_lossless() {
    assert_kinds(
        "* Open\r\n#+begin_src rust\r\nlet x = 1;\r\n",
        &[SyntaxKind::HEADLINE],
    );
}
