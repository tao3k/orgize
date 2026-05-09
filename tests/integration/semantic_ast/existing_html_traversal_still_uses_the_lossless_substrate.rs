use super::*;

#[test]
fn existing_html_traversal_still_uses_the_lossless_substrate() {
    let html = Org::parse(
        r#"* title
paragraph with [[https://example.com][link]]

- one
- two

#+begin_quote
quoted
#+end_quote
"#,
    )
    .to_html();

    insta::with_settings!({snapshot_path => "../../snapshots", prepend_module_to_snapshot => false}, {
        insta::assert_snapshot!("semantic_ast__semantic_ast_html_compatibility", html);
    });
}
