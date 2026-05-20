use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{
        SourceBlockDirectoryKind, SourceBlockEvalPolicy, SourceBlockExportsPolicy,
        SourceBlockHeaderArgKind, SourceBlockHeaderArgSource, SourceBlockNowebAction,
        SourceBlockRecordKind, SourceBlockReferenceKind, SourceBlockResultCollection,
        SourceBlockResultFormat, SourceBlockResultHandling, SourceBlockResultKind,
        SourceBlockResultValueType, SourceBlockTangleCommentsMode, SourceBlockTangleMode,
        SourceBlockTangleNowebMode,
    },
};

#[test]
fn semantic_ast_projects_source_block_records_with_results_and_tangle() {
    let doc = Org::parse(
        r#"#+NAME: demo-block
#+begin_src sh :results output :tangle "scripts/run.sh" :mkdirp yes :comments both :shebang #!/usr/bin/env sh :noweb tangle
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
    let tangle = records[0].tangle.as_ref().expect("tangle metadata");
    assert!(tangle.mkdirp.enabled);
    assert_eq!(tangle.comments.mode, SourceBlockTangleCommentsMode::Both);
    assert_eq!(tangle.shebang.as_deref(), Some("#!/usr/bin/env sh"));
    assert_eq!(tangle.noweb.mode, SourceBlockTangleNowebMode::Expand);
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
    assert_eq!(
        records[0].result_options.value_type,
        SourceBlockResultValueType::Output
    );

    assert_eq!(records[1].language.as_deref(), Some("emacs-lisp"));
    assert_eq!(
        records[1].tangle.as_ref().map(|tangle| tangle.mode),
        Some(SourceBlockTangleMode::No)
    );
    assert!(records[1].result.is_none());
}

#[test]
fn semantic_ast_projects_source_block_result_options() {
    let doc = Org::parse(
        r#"#+PROPERTY: header-args :results file html replace :file "default.html"
#+NAME: report
#+begin_src python :results list append output :file "reports/out.json" :file-desc "Report output" :file-ext json :file-mode 0644 :output-dir public
print("report")
#+end_src

#+begin_src sh :results drawer silent unknown-mode
echo hidden
#+end_src
"#,
    )
    .document();

    assert_clean_projection(&doc);
    let records = doc.source_block_records();
    assert_eq!(records.len(), 2);

    let report = &records[0].result_options;
    assert_eq!(report.collection, Some(SourceBlockResultCollection::List));
    assert_eq!(report.format, Some(SourceBlockResultFormat::Html));
    assert_eq!(report.handling, SourceBlockResultHandling::Append);
    assert_eq!(report.value_type, SourceBlockResultValueType::Output);
    let file = report.file.as_ref().expect("file result target");
    assert_eq!(file.target, "reports/out.json");
    assert_eq!(file.description.as_deref(), Some("Report output"));
    assert_eq!(file.extension.as_deref(), Some("json"));
    assert_eq!(
        file.file_mode.as_ref().map(|mode| mode.raw.as_str()),
        Some("0644")
    );
    assert_eq!(file.output_dir.as_deref(), Some("public"));

    let hidden = &records[1].result_options;
    assert_eq!(hidden.format, Some(SourceBlockResultFormat::Drawer));
    assert_eq!(hidden.handling, SourceBlockResultHandling::Silent);
    assert_eq!(hidden.value_type, SourceBlockResultValueType::Value);
    assert_eq!(hidden.unknown, ["unknown-mode"]);
}

#[test]
fn semantic_ast_projects_source_block_execution_plan() {
    let doc = Org::parse(
        r#"#+PROPERTY: header-args :eval query :cache yes :session shared :dir ./workspace :noweb no-export
#+PROPERTY: header-args:python :exports results :hlines yes
#+begin_src python :eval never-export :exports both :cache no :session none :dir attach :noweb strip-export
print("x")
#+end_src

Inline src_sh[:exports none :eval no :noweb eval]{echo hi}
"#,
    )
    .document();

    assert_clean_projection(&doc);
    let records = doc.source_block_records();
    assert_eq!(records.len(), 2);

    let block = records
        .iter()
        .find(|record| record.kind == SourceBlockRecordKind::Block)
        .expect("source block");
    assert_eq!(
        block.execution.eval.policy,
        SourceBlockEvalPolicy::NeverExport
    );
    assert_eq!(
        block.execution.eval.source,
        SourceBlockHeaderArgSource::Explicit
    );
    assert_eq!(
        block.execution.exports.policy,
        SourceBlockExportsPolicy::Both
    );
    assert!(!block.execution.cache.enabled);
    assert!(!block.execution.session.active);
    assert_eq!(
        block
            .execution
            .directory
            .as_ref()
            .map(|directory| (directory.target.as_str(), directory.kind)),
        Some(("attach", SourceBlockDirectoryKind::Attachment))
    );
    assert!(block.execution.hlines.enabled);
    assert_eq!(block.execution.noweb.eval, SourceBlockNowebAction::Expand);
    assert_eq!(block.execution.noweb.export, SourceBlockNowebAction::Strip);
    assert_eq!(block.execution.noweb.tangle, SourceBlockNowebAction::Expand);

    let inline = records
        .iter()
        .find(|record| record.kind == SourceBlockRecordKind::InlineSource)
        .expect("inline source block");
    assert_eq!(inline.execution.eval.policy, SourceBlockEvalPolicy::No);
    assert_eq!(
        inline.execution.exports.policy,
        SourceBlockExportsPolicy::None
    );
    assert!(inline.execution.cache.enabled);
    assert_eq!(inline.execution.session.name.as_deref(), Some("shared"));
    assert_eq!(
        inline
            .execution
            .directory
            .as_ref()
            .map(|directory| (directory.target.as_str(), directory.kind)),
        Some(("./workspace", SourceBlockDirectoryKind::Path))
    );
    assert!(inline.execution.hlines.enabled);
    assert_eq!(inline.execution.noweb.eval, SourceBlockNowebAction::Expand);
    assert_eq!(
        inline.execution.noweb.export,
        SourceBlockNowebAction::Disabled
    );
    assert_eq!(
        inline.execution.noweb.tangle,
        SourceBlockNowebAction::Disabled
    );
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
fn semantic_ast_projects_source_block_records_property_header_args() {
    let doc = Org::parse(
        r#"#+PROPERTY: header-args :results output :exports results :var dataset=data_block
#+PROPERTY: header-args:python :session py :tangle "python.py"
#+NAME: data_block
#+begin_src emacs-lisp
(list 1 2)
#+end_src

#+NAME: root
#+HEADER: :tangle "affiliated.py"
#+begin_src python :exports both :tangle "root.py"
print(dataset)
#+end_src

Inline src_python[:results value]{print("inline")}

* Local
:PROPERTIES:
:header-args: :cache yes :results drawer
:header-args:python: :session local
:END:
#+begin_src python
print("local")
#+end_src
"#,
    )
    .document();

    assert_clean_projection(&doc);
    let records = doc.source_block_records();
    assert_eq!(records.len(), 4);

    let root = records
        .iter()
        .find(|record| record.name.as_deref() == Some("root"))
        .expect("named root source block");
    assert_eq!(
        root.header_args
            .iter()
            .map(|arg| arg.raw.as_str())
            .collect::<Vec<_>>(),
        [
            ":results output",
            ":exports results",
            ":var dataset=data_block",
            ":session py",
            r#":tangle "python.py""#,
            r#":tangle "affiliated.py""#,
            ":exports both",
            r#":tangle "root.py""#,
        ]
    );
    assert_eq!(
        root.tangle
            .as_ref()
            .and_then(|tangle| tangle.target.as_deref()),
        Some("root.py")
    );
    assert_eq!(
        root.normalized_header_args
            .iter()
            .find(|arg| arg.key == "session")
            .and_then(|arg| arg.value.as_deref()),
        Some("py")
    );
    assert_eq!(
        root.normalized_header_args
            .iter()
            .find(|arg| arg.key == "exports")
            .and_then(|arg| arg.value.as_deref()),
        Some("both")
    );
    assert!(root.normalized_header_args.iter().any(|arg| {
        arg.kind == SourceBlockHeaderArgKind::Var
            && arg.variable.as_ref().is_some_and(|var| {
                var.name == "dataset" && var.assignment.as_deref() == Some("data_block")
            })
    }));

    let inline = records
        .iter()
        .find(|record| record.kind == SourceBlockRecordKind::InlineSource)
        .expect("inline source block");
    assert_eq!(inline.language.as_deref(), Some("python"));
    assert_eq!(
        inline
            .normalized_header_args
            .iter()
            .find(|arg| arg.key == "session")
            .and_then(|arg| arg.value.as_deref()),
        Some("py")
    );
    assert!(inline.normalized_header_args.iter().any(|arg| {
        arg.key == "results"
            && arg.source == SourceBlockHeaderArgSource::Explicit
            && arg.value.as_deref() == Some("value")
    }));

    let local = records
        .iter()
        .find(|record| record.value == "print(\"local\")\n")
        .expect("subtree source block");
    assert_eq!(
        local
            .header_args
            .iter()
            .map(|arg| arg.raw.as_str())
            .collect::<Vec<_>>(),
        [":cache yes", ":results drawer", ":session local"]
    );
    assert!(local.normalized_header_args.iter().all(|arg| {
        arg.variable
            .as_ref()
            .is_none_or(|var| var.name.as_str() != "dataset")
    }));
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
