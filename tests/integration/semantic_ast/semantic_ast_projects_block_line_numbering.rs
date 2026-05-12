use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{BlockKind, BlockLineNumberMode, ElementData},
    Org,
};

#[test]
fn semantic_ast_projects_source_and_example_block_line_numbering() {
    let doc = Org::parse(
        r#"#+begin_src rust -n 20 -r :exports code
fn main() {}
#+end_src

#+begin_src rust +n 10
println!("continued");
#+end_src

#+begin_example -n 3
,* example
#+end_example
"#,
    )
    .document();

    assert_clean_projection(&doc);

    let blocks = doc
        .children
        .iter()
        .filter_map(|element| match &element.data {
            ElementData::Block(block) => Some(block),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(blocks.len(), 3);

    assert_eq!(blocks[0].kind, BlockKind::Source);
    assert_eq!(blocks[0].language.as_deref(), Some("rust"));
    assert_eq!(blocks[0].switches.as_deref(), Some("-n 20 -r"));
    assert_eq!(blocks[0].parameters.as_deref(), Some(":exports code"));
    let first_numbering = blocks[0]
        .line_numbering
        .as_ref()
        .expect("source block line numbering");
    assert_eq!(first_numbering.mode, BlockLineNumberMode::New);
    assert_eq!(first_numbering.start, Some(20));

    assert_eq!(blocks[1].kind, BlockKind::Source);
    assert_eq!(blocks[1].switches.as_deref(), Some("+n 10"));
    let continued_numbering = blocks[1]
        .line_numbering
        .as_ref()
        .expect("continued source block line numbering");
    assert_eq!(continued_numbering.mode, BlockLineNumberMode::Continued);
    assert_eq!(continued_numbering.start, Some(10));

    assert_eq!(blocks[2].kind, BlockKind::Example);
    assert_eq!(blocks[2].switches.as_deref(), Some("-n 3"));
    let example_numbering = blocks[2]
        .line_numbering
        .as_ref()
        .expect("example block line numbering");
    assert_eq!(example_numbering.mode, BlockLineNumberMode::New);
    assert_eq!(example_numbering.start, Some(3));
}
