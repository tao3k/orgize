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
fn semantic_ast_projects_source_block_records_affiliated_header_args() {
    let doc = Org::parse(
        r#"#+HEADER: :var data=dataset :results output
#+HEADERS: :tangle "scripts/report.py" :noweb-ref report
#+NAME: report
#+begin_src python :results drawer :exports both
print(data)
#+end_src
"#,
    )
    .document();

    assert_clean_projection(&doc);
    let records = doc.source_block_records();
    assert_eq!(records.len(), 1);

    let record = &records[0];
    assert_eq!(record.name.as_deref(), Some("report"));
    assert_eq!(record.language.as_deref(), Some("python"));
    assert_eq!(record.header_args.len(), 6);
    assert_eq!(record.header_args[0].raw, ":var data=dataset");
    assert_eq!(record.header_args[1].raw, ":results output");
    assert_eq!(record.header_args[2].raw, r#":tangle "scripts/report.py""#);
    assert_eq!(record.header_args[3].raw, ":noweb-ref report");
    assert_eq!(record.header_args[4].raw, ":results drawer");
    assert_eq!(record.header_args[5].raw, ":exports both");
    assert_eq!(
        record
            .tangle
            .as_ref()
            .and_then(|tangle| tangle.target.as_deref()),
        Some("scripts/report.py")
    );
    assert!(record.normalized_header_args.iter().any(|arg| {
        arg.key == "var"
            && arg.variable.as_ref().is_some_and(|var| {
                var.name == "data" && var.assignment.as_deref() == Some("dataset")
            })
    }));
    assert_eq!(
        record
            .normalized_header_args
            .iter()
            .find(|arg| arg.key == "exports")
            .and_then(|arg| arg.value.as_deref()),
        Some("both")
    );
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

#+begin_src python :var rows=load_data :var scoped=load_data(limit=1) :var literal=42 :var quoted="load_data" :var missing=missing_call()
print(rows)
#+end_src

#+CALL: load_data()
#+CALL: setup()
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
            (SourceBlockReferenceKind::HeaderVar, "load_data", true),
            (SourceBlockReferenceKind::HeaderVar, "load_data", true),
            (SourceBlockReferenceKind::HeaderVar, "missing_call", false),
            (SourceBlockReferenceKind::BabelCall, "load_data", true),
            (SourceBlockReferenceKind::BabelCall, "setup", false),
            (SourceBlockReferenceKind::InlineCall, "load_data", true),
            (
                SourceBlockReferenceKind::InlineCall,
                "missing_inline",
                false
            ),
        ]
    );
}
