use orgize::{
    Org,
    ast::{
        BabelEvalOutput, BabelEvalPlanError, BabelEvalResultPatchKind, SourceBlockEvalPolicy,
        SourceBlockResultHandling,
    },
};

#[test]
fn semantic_ast_projects_named_babel_eval_plan_without_running_code() {
    let doc = Org::parse(
        r#"#+NAME: verify
#+BEGIN_SRC bash :results output replace :eval query
echo ok
#+END_SRC
"#,
    )
    .document();

    let plan = doc.babel_eval_plan("verify").expect("eval plan");
    assert_eq!(plan.name, "verify");
    assert_eq!(plan.record.language.as_deref(), Some("bash"));
    assert_eq!(plan.record.value.trim(), "echo ok");
    assert_eq!(
        plan.record.execution.eval.policy,
        SourceBlockEvalPolicy::Query
    );
    assert_eq!(
        plan.record.result_options.handling,
        SourceBlockResultHandling::Replace
    );
    assert!(plan.record.result.is_none());
}

#[test]
fn semantic_ast_projects_babel_eval_patch_inserts_results() {
    let source = r#"#+NAME: verify
#+BEGIN_SRC bash :results output replace
echo ok
#+END_SRC
"#;
    let doc = Org::parse(source).document();
    let plan = doc.babel_eval_plan("verify").expect("eval plan");
    let patch = plan.result_patch(
        source,
        &BabelEvalOutput {
            stdout: "ok\n".to_string(),
            stderr: String::new(),
            exit_code: Some(0),
        },
    );

    assert_eq!(patch.kind, BabelEvalResultPatchKind::Insert);
    assert_eq!(patch.replacement, "\n#+RESULTS: verify\n: ok\n".to_string());
    assert_eq!(
        patch.apply_to(source),
        r#"#+NAME: verify
#+BEGIN_SRC bash :results output replace
echo ok
#+END_SRC

#+RESULTS: verify
: ok
"#
    );
}

#[test]
fn semantic_ast_projects_babel_eval_patch_replaces_existing_results() {
    let source = r#"#+NAME: verify
#+BEGIN_SRC bash :results output replace
echo old
#+END_SRC

#+RESULTS:
: old
"#;
    let doc = Org::parse(source).document();
    let plan = doc.babel_eval_plan("verify").expect("eval plan");
    let patch = plan.result_patch(
        source,
        &BabelEvalOutput {
            stdout: "new\n".to_string(),
            stderr: String::new(),
            exit_code: Some(0),
        },
    );

    assert_eq!(patch.kind, BabelEvalResultPatchKind::Replace);
    assert_eq!(
        patch.apply_to(source),
        r#"#+NAME: verify
#+BEGIN_SRC bash :results output replace
echo old
#+END_SRC

#+RESULTS: verify
: new
"#
    );
}

#[test]
fn semantic_ast_projects_babel_eval_reports_ambiguous_names() {
    let doc = Org::parse(
        r#"#+NAME: verify
#+BEGIN_SRC bash
echo one
#+END_SRC

#+NAME: verify
#+BEGIN_SRC bash
echo two
#+END_SRC
"#,
    )
    .document();

    assert_eq!(
        doc.babel_eval_plan("verify"),
        Err(BabelEvalPlanError::Ambiguous {
            name: "verify".to_string(),
            matches: 2,
        })
    );
}
