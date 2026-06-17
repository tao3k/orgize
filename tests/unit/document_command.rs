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
fn query_content_compacts_soft_wrapped_paragraphs() {
    let content = "first line\n  second   line\nthird line";

    assert_eq!(
        compact_query_content(content),
        "first line second line third line"
    );
}

#[test]
fn query_content_preserves_markdown_code_fence_syntax() {
    let content = "Intro\n\n```json\n  {  \"k\":   \"v\"  }\n```\n\nOutro";

    assert_eq!(
        compact_query_content(content),
        "Intro\n```json\n  {  \"k\":   \"v\"  }\n```\nOutro"
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
