use std::path::Path;

#[test]
fn source_block_body_is_parser_owned_and_excludes_org_delimiters() {
    let source = "\
#+begin_src typst
$ rho(B) = c + s + a $
#+end_src

#+begin_src mermaid
flowchart LR
  A -->|closed| B
#+end_src
";
    let facts = super::org_elements::index_org(Path::new("proof.org"), source);
    let blocks = facts
        .iter()
        .filter(|fact| fact.kind == "block")
        .collect::<Vec<_>>();

    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].content, "$ rho(B) = c + s + a $\n");
    assert_eq!(blocks[1].content, "flowchart LR\n  A -->|closed| B\n");
    assert!(!blocks[0].content.contains("#+begin_src"));
    assert!(!blocks[1].content.contains("#+end_src"));
    assert_eq!(
        super::command_query::project_query_content(blocks[0]),
        "$ rho(B) = c + s + a $\n"
    );
    assert_eq!(
        super::command_query::project_query_content(blocks[1]),
        "flowchart LR\n  A -->|closed| B\n"
    );
}
