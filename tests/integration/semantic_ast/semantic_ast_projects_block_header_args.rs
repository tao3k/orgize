use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{BlockKind, BlockLineNumberMode, ElementData},
};

#[test]
fn semantic_ast_projects_source_block_header_args_and_indentation_switch() {
    let doc = Org::parse(
        r#"#+begin_src emacs-lisp -i -n 5 :exports both :results output drawer :var x=1 :tangle "docs/demo.org" :noweb-ref setup
(message "hi")
#+end_src

#+begin_src sh :dir "/tmp/:not-key" :eval no-export
echo ok
#+end_src
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
    assert_eq!(blocks.len(), 2);

    assert_eq!(blocks[0].kind, BlockKind::Source);
    assert_eq!(blocks[0].language.as_deref(), Some("emacs-lisp"));
    assert_eq!(blocks[0].switches.as_deref(), Some("-i -n 5"));
    assert!(blocks[0].preserve_indentation);
    let numbering = blocks[0]
        .line_numbering
        .as_ref()
        .expect("source block line numbering");
    assert_eq!(numbering.mode, BlockLineNumberMode::New);
    assert_eq!(numbering.start, Some(5));
    assert_eq!(
        blocks[0].parameters.as_deref(),
        Some(
            r#":exports both :results output drawer :var x=1 :tangle "docs/demo.org" :noweb-ref setup"#
        )
    );
    assert_eq!(blocks[0].header_args.len(), 5);
    assert_eq!(blocks[0].header_args[0].key, "exports");
    assert_eq!(blocks[0].header_args[0].value.as_deref(), Some("both"));
    assert_eq!(blocks[0].header_args[0].raw, ":exports both");
    assert_eq!(blocks[0].header_args[1].key, "results");
    assert_eq!(
        blocks[0].header_args[1].value.as_deref(),
        Some("output drawer")
    );
    assert_eq!(blocks[0].header_args[2].key, "var");
    assert_eq!(blocks[0].header_args[2].value.as_deref(), Some("x=1"));
    assert_eq!(blocks[0].header_args[3].key, "tangle");
    assert_eq!(
        blocks[0].header_args[3].value.as_deref(),
        Some(r#""docs/demo.org""#)
    );
    assert_eq!(blocks[0].header_args[4].key, "noweb-ref");
    assert_eq!(blocks[0].header_args[4].value.as_deref(), Some("setup"));

    assert_eq!(blocks[1].kind, BlockKind::Source);
    assert_eq!(blocks[1].switches, None);
    assert!(!blocks[1].preserve_indentation);
    assert_eq!(
        blocks[1].parameters.as_deref(),
        Some(r#":dir "/tmp/:not-key" :eval no-export"#)
    );
    assert_eq!(blocks[1].header_args.len(), 2);
    assert_eq!(blocks[1].header_args[0].key, "dir");
    assert_eq!(
        blocks[1].header_args[0].value.as_deref(),
        Some(r#""/tmp/:not-key""#)
    );
    assert_eq!(blocks[1].header_args[1].key, "eval");
    assert_eq!(blocks[1].header_args[1].value.as_deref(), Some("no-export"));
}
