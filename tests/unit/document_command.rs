use crate::document::compact_query_content;

#[test]
fn query_content_compacts_document_whitespace() {
    let content = "Plugin JSON sample spec\n\njson\n---\n  trailing   words";

    assert_eq!(
        compact_query_content(content),
        "Plugin JSON sample spec json --- trailing words"
    );
}
