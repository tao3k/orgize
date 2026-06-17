use orgize::{
    Org,
    ast::parse_contracts_from_document,
    lint::{LintOptions, lint_org_with_options},
};

#[test]
fn lint_builtin_document_metadata_runs_without_target_contract_org() {
    let report = lint_org_with_options(
        "Steer\n\n    To pick up a draggable item, press the space bar.\n",
        &LintOptions::default(),
    );
    let messages = report
        .findings
        .iter()
        .filter(|finding| finding.code == "ORG044")
        .map(|finding| finding.message.as_str())
        .collect::<Vec<_>>();

    assert_eq!(messages.len(), 2, "{}", report.to_text("steer.org"));
    assert!(
        messages
            .iter()
            .any(|message| message.contains("document is missing a #+TITLE keyword")),
        "{}",
        report.to_text("steer.org")
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("document is missing document-level properties")),
        "{}",
        report.to_text("steer.org")
    );
}

#[test]
fn lint_builtin_document_metadata_accepts_downstream_document_without_contract_org() {
    let report = lint_org_with_options(
        r#"#+TITLE: Steer
#+PROPERTY: STATUS draft

Steer instructions.
"#,
        &LintOptions::default(),
    );

    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.code != "ORG044"),
        "{}",
        report.to_text("steer.org")
    );
}

#[test]
fn lint_builtin_document_metadata_allows_document_contract_override() {
    let registry = document_contract_registry();
    let report = lint_org_with_options(
        r#"#+CONTRACT_ORG: agent.document.v1

Steer instructions.

* Steer
"#,
        &LintOptions {
            org_contract_registry: registry,
            ..LintOptions::default()
        },
    );

    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.code != "ORG044"),
        "{}",
        report.to_text("steer.org")
    );
}

fn document_contract_registry() -> orgize::ast::OrgContractRegistry {
    let document = Org::parse(document_contract_source()).document();
    parse_contracts_from_document(&document, None)
}

fn document_contract_source() -> &'static str {
    r#"* document-v1
:PROPERTIES:
:CONTRACT_ID: agent.document.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:

** has-heading
:PROPERTIES:
:ASSERT_ID: document.has-heading
:SEVERITY: warning
:END:

#+BEGIN_SRC org-contract
(assert exists
  (headline))
#+END_SRC
"#
}
