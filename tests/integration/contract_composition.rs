use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use orgize::ast::parse_contract_references;

const REGISTRY: &str = r#"* document-contract
:PROPERTIES:
:CONTRACT_ID: philosophy.document.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** has-title
:PROPERTIES:
:ASSERT_ID: philosophy.document.has-title
:SEVERITY: error
:END:
#+BEGIN_SRC org-contract
(assert exists (keyword :context "metadata" :summary (key "TITLE")))
#+END_SRC
* charter-contract
:PROPERTIES:
:CONTRACT_ID: philosophy.charter.en.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** has-claim
:PROPERTIES:
:ASSERT_ID: philosophy.charter.en.has-claim
:SEVERITY: error
:END:
#+BEGIN_SRC org-contract
(assert exists (headline :summary (title "Claim")))
#+END_SRC
"#;

static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

#[test]
fn parser_preserves_reference_order_and_nested_link_whitespace() {
    let references = parse_contract_references(
        "philosophy.document.v1, philosophy.charter.en.v1 [[file:contracts/shared.org][Shared contract]]",
    );

    assert_eq!(references.len(), 3);
    assert_eq!(
        references[0].contract_id.as_deref(),
        Some("philosophy.document.v1")
    );
    assert_eq!(
        references[1].contract_id.as_deref(),
        Some("philosophy.charter.en.v1")
    );
    assert_eq!(references[2].path.as_deref(), Some("contracts/shared.org"));
    assert_eq!(
        references[2].contract_id.as_deref(),
        Some("Shared contract")
    );
}

#[test]
fn trace_and_lint_execute_inline_contracts_in_declared_order() {
    let dir = fixture_dir();
    fs::write(dir.join("charter.org"), complete_charter()).unwrap();

    let trace = run(&dir, "contract", "charter.org");
    assert!(trace.status.success(), "{}", receipt(&trace));
    let trace_json: serde_json::Value = serde_json::from_slice(&trace.stdout).unwrap();
    let evaluations = trace_json["files"][0]["evaluations"].as_array().unwrap();
    assert_eq!(
        evaluations
            .iter()
            .map(|evaluation| evaluation["contractId"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["philosophy.document.v1", "philosophy.charter.en.v1"]
    );
    assert!(
        evaluations
            .iter()
            .all(|evaluation| { evaluation["assertions"][0]["status"].as_str() == Some("passed") })
    );

    let lint = run(&dir, "lint", "charter.org");
    assert!(lint.status.success(), "{}", receipt(&lint));
}

#[test]
fn a_failed_contract_does_not_short_circuit_later_contracts() {
    let dir = fixture_dir();
    fs::write(
        dir.join("incomplete.org"),
        ":PROPERTIES:\n:CONTRACT_ORG: philosophy.document.v1 philosophy.charter.en.v1\n:END:\n* Claim\nStill evaluated.\n",
    )
    .unwrap();

    let trace = run(&dir, "contract", "incomplete.org");
    assert!(trace.status.success(), "{}", receipt(&trace));
    let json: serde_json::Value = serde_json::from_slice(&trace.stdout).unwrap();
    let evaluations = json["files"][0]["evaluations"].as_array().unwrap();
    assert_eq!(evaluations[0]["assertions"][0]["status"], "failed");
    assert_eq!(evaluations[1]["assertions"][0]["status"], "passed");
}

#[test]
fn duplicate_contracts_on_one_scope_are_rejected() {
    let dir = fixture_dir();
    fs::write(
        dir.join("duplicate.org"),
        "#+TITLE: Duplicate\n:PROPERTIES:\n:CONTRACT_ORG: philosophy.document.v1 philosophy.document.v1\n:END:\n* Claim\nAmbiguous.\n",
    )
    .unwrap();

    let trace = run(&dir, "contract", "duplicate.org");
    assert!(!trace.status.success(), "duplicate binding must fail");
    assert!(
        String::from_utf8_lossy(&trace.stderr)
            .contains("duplicate CONTRACT_ORG `philosophy.document.v1`")
    );
}

fn fixture_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("orgize-cli-tests")
        .join(format!(
            "contract-composition-{}-{}",
            std::process::id(),
            NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("contracts.org"), REGISTRY).unwrap();
    dir
}

fn complete_charter() -> &'static str {
    "#+TITLE: Charter\n:PROPERTIES:\n:CONTRACT_ORG: philosophy.document.v1 philosophy.charter.en.v1\n:END:\n* Claim\nKnowledge must be grounded.\n"
}

fn run(dir: &PathBuf, command: &str, target: &str) -> std::process::Output {
    let mut args = vec![command];
    if command == "contract" {
        args.push("trace");
    }
    args.extend(["--org-contract-registry", "contracts.org", target]);
    Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

fn receipt(output: &std::process::Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status.code().unwrap_or_default(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}
