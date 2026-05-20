use crate::semantic_ast::support::assert_clean_projection;
use orgize::{Org, ast::TodoState};

#[test]
fn semantic_ast_applies_file_todo_keyword_declarations_before_headline_projection() {
    let org = Org::parse(
        r#"#+TODO: NEXT(n) WAIT(w@/!) | DONE (d) CANCELED(c)
#+SEQ_TODO: REVIEW(r) | MERGED(m)
#+typ_todo: BUG(b) | FIXED(f)
#+begin_example
#+TODO: BAD | WORSE
#+end_example
* NEXT Prepare
* WAIT Blocked
* DONE Finished
* CANCELED Dropped
* REVIEW Pull request
* MERGED Pull request
* BUG Regression
* FIXED Regression
* TODO Default keyword is just title text
"#,
    );
    let doc = org.document();

    assert_clean_projection(&doc);
    let todo_keywords = org
        .config()
        .todo_keywords
        .0
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let done_keywords = org
        .config()
        .todo_keywords
        .1
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(todo_keywords, ["NEXT", "WAIT", "REVIEW", "BUG"]);
    assert_eq!(done_keywords, ["DONE", "CANCELED", "MERGED", "FIXED"]);
    assert_eq!(doc.children.len(), 4);
    assert!(format!("{:?}", doc.children).contains("BAD | WORSE"));
    assert_eq!(doc.sections.len(), 9);

    let states = doc
        .sections
        .iter()
        .map(|section| {
            section
                .todo
                .as_ref()
                .map(|todo| (todo.name.as_str(), todo.state))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        states,
        [
            Some(("NEXT", TodoState::Todo)),
            Some(("WAIT", TodoState::Todo)),
            Some(("DONE", TodoState::Done)),
            Some(("CANCELED", TodoState::Done)),
            Some(("REVIEW", TodoState::Todo)),
            Some(("MERGED", TodoState::Done)),
            Some(("BUG", TodoState::Todo)),
            Some(("FIXED", TodoState::Done)),
            None,
        ]
    );
    assert_eq!(
        doc.sections[8].raw_title,
        "TODO Default keyword is just title text"
    );
}
