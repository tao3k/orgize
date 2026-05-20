use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{
        SourceBlockHeaderArgKind, SourceBlockHeaderArgSource, SourceBlockRecordKind,
        SourceBlockReferenceKind, SourceBlockResultKind, SourceBlockTangleMode,
    },
};

#[test]
fn semantic_ast_projects_source_block_records_with_results_and_tangle() {
    let doc = Org::parse(
        r#"#+NAME: demo-block
#+begin_src sh :results output :tangle "scripts/run.sh"
echo hi
#+end_src

#+RESULTS:
: hi

#+begin_src emacs-lisp :tangle no
(message "skip")
#+end_src
"#,
    )
    .document();

    assert_clean_projection(&doc);
    let records = doc.source_block_records();
    assert_eq!(records.len(), 2);

    assert_eq!(records[0].kind, SourceBlockRecordKind::Block);
    assert_eq!(records[0].name.as_deref(), Some("demo-block"));
    assert_eq!(records[0].language.as_deref(), Some("sh"));
    assert_eq!(
        records[0]
            .tangle
            .as_ref()
            .and_then(|tangle| tangle.target.as_deref()),
        Some("scripts/run.sh")
    );
    assert_eq!(
        records[0].tangle.as_ref().map(|tangle| tangle.mode),
        Some(SourceBlockTangleMode::File)
    );
    assert_eq!(
        records[0]
            .result
            .as_ref()
            .map(|result| result.value.as_str()),
        Some("hi")
    );
    assert_eq!(
        records[0].result.as_ref().map(|result| result.kind),
        Some(SourceBlockResultKind::Keyword)
    );
    let results_arg = records[0]
        .normalized_header_args
        .iter()
        .find(|arg| {
            arg.kind == SourceBlockHeaderArgKind::Results
                && arg.source == SourceBlockHeaderArgSource::Explicit
        })
        .expect("explicit :results projection");
    assert_eq!(results_arg.tokens, ["output"]);

    assert_eq!(records[1].language.as_deref(), Some("emacs-lisp"));
    assert_eq!(
        records[1].tangle.as_ref().map(|tangle| tangle.mode),
        Some(SourceBlockTangleMode::No)
    );
    assert!(records[1].result.is_none());
}

#[test]
fn semantic_ast_projects_inline_source_records_with_defaults_and_results_macro() {
    let doc = Org::parse(
        r#"Value src_sh[:exports both :var x=1]{echo $x}{{{results(=1=)}}}
"#,
    )
    .document();

    assert_clean_projection(&doc);
    let records = doc.source_block_records();
    assert_eq!(records.len(), 1);

    assert_eq!(records[0].kind, SourceBlockRecordKind::InlineSource);
    assert_eq!(records[0].language.as_deref(), Some("sh"));
    assert_eq!(records[0].value, "echo $x");
    assert_eq!(
        records[0]
            .normalized_header_args
            .iter()
            .find(|arg| {
                arg.key == "exports" && arg.source == SourceBlockHeaderArgSource::Explicit
            })
            .and_then(|arg| arg.value.as_deref()),
        Some("both")
    );
    assert_eq!(
        records[0]
            .normalized_header_args
            .iter()
            .find(|arg| arg.key == "hlines")
            .map(|arg| (arg.value.as_deref(), arg.source)),
        Some((Some("yes"), SourceBlockHeaderArgSource::Default))
    );
    assert_eq!(
        records[0]
            .normalized_header_args
            .iter()
            .find(|arg| arg.kind == SourceBlockHeaderArgKind::Var)
            .and_then(|arg| arg.variable.as_ref())
            .map(|var| (var.name.as_str(), var.assignment.as_deref())),
        Some(("x", Some("1")))
    );
    assert_eq!(
        records[0].result.as_ref().map(|result| result.kind),
        Some(SourceBlockResultKind::InlineMacro)
    );
    assert_eq!(
        records[0]
            .result
            .as_ref()
            .map(|result| result.value.as_str()),
        Some("=1=")
    );
}

#[test]
fn semantic_ast_projects_literate_source_block_references() {
    let doc = Org::parse(
        r#"#+NAME: load_data
#+begin_src python :noweb-ref setup
print("load")
#+end_src

#+begin_src python
<<load_data>>
<<setup("topic")>>
<<missing>>
#+end_src

#+CALL: load_data()
Inline call_load_data() and call_missing_inline().
"#,
    )
    .document();

    assert_clean_projection(&doc);
    let references = doc.source_block_references();
    let summary = references
        .iter()
        .map(|reference| {
            (
                reference.kind,
                reference.target.as_str(),
                reference.resolved,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        summary,
        [
            (SourceBlockReferenceKind::Noweb, "load_data", true),
            (SourceBlockReferenceKind::Noweb, "setup", true),
            (SourceBlockReferenceKind::Noweb, "missing", false),
            (SourceBlockReferenceKind::BabelCall, "load_data", true),
            (SourceBlockReferenceKind::InlineCall, "load_data", true),
            (
                SourceBlockReferenceKind::InlineCall,
                "missing_inline",
                false
            ),
        ]
    );
}
