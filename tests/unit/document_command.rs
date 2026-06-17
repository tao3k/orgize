use crate::document::compact_query_content;

#[test]
fn query_content_compacts_document_whitespace() {
    let content = "Plugin JSON sample spec\n\njson\n---\n  trailing   words";

    assert_eq!(
        compact_query_content(content),
        "Plugin JSON sample spec\njson\n---\ntrailing words"
    );
}

#[test]
fn query_content_preserves_org_source_block_syntax() {
    let content = "* Project\n\n** Overview\n\n- item   with    spaces\n\n#+begin_src scheme\n  (display   \"x\")\n#+end_src\n";

    assert_eq!(
        compact_query_content(content),
        "* Project\n** Overview\n- item with spaces\n#+begin_src scheme\n  (display   \"x\")\n#+end_src"
    );
}
