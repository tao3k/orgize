use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{BlockKind, ElementData},
    Org,
};

#[test]
fn semantic_ast_projects_lesser_elements_and_block_variants() {
    let doc = Org::parse(
        r#"# file comment
:LOGBOOK:
Inside drawer with *bold* text.
:END:

: fixed
: width

#+begin_verse
Verse *body*.
#+end_verse

#+begin_center
Centered /body/.
#+end_center

#+begin_comment
hidden
#+end_comment

#+begin_details
Special body.
#+end_details

#+BEGIN: clocktable :scope file

| Task | Time |
#+END:

\begin{equation}
x=1
\end{equation}
"#,
    )
    .document();

    assert_clean_projection(&doc);

    assert!(doc
        .children
        .iter()
        .any(|element| matches!(element.data, ElementData::Comment(_))));
    assert!(doc
        .children
        .iter()
        .any(|element| matches!(element.data, ElementData::FixedWidth(_))));
    assert!(doc
        .children
        .iter()
        .any(|element| matches!(element.data, ElementData::LatexEnvironment(_))));

    let drawer = doc
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::Drawer(drawer) => Some(drawer),
            _ => None,
        })
        .expect("drawer element");
    assert_eq!(drawer.name, "LOGBOOK");
    assert!(drawer
        .children
        .iter()
        .any(|element| matches!(element.data, ElementData::Paragraph(_))));

    let block_kinds = doc
        .children
        .iter()
        .filter_map(|element| match &element.data {
            ElementData::Block(block) => Some(&block.kind),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(block_kinds.contains(&&BlockKind::Verse));
    assert!(block_kinds.contains(&&BlockKind::Center));
    assert!(block_kinds.contains(&&BlockKind::Comment));
    assert!(block_kinds.contains(&&BlockKind::Dynamic));
    assert!(block_kinds
        .iter()
        .any(|kind| matches!(kind, BlockKind::Special(name) if name == "details")));
}
