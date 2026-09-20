use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

use asp_rust_build_support::{
    AspRustScenarioObservation, asp_rust_scenario, measure_asp_rust_scenario,
};

static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

#[test]
fn workspace_contract_admits_exact_reciprocal_pair() {
    let fixture = WorkspaceFixture::new();
    let output = fixture.run();
    assert!(output.status.success(), "{}", receipt(&output));
    assert!(String::from_utf8_lossy(&output.stdout).contains("2 documents, 4 evaluations"));
}

#[test]
fn workspace_contract_requires_trace_target_to_be_maintained() {
    let fixture = WorkspaceFixture::new();
    let admitted = fixture.run_requiring(&fixture.root.join("cn/docs/doc.org"));
    assert!(admitted.status.success(), "{}", receipt(&admitted));

    let rejected = fixture.run_requiring(&fixture.root.join("contracts.org"));
    assert!(
        !rejected.status.success(),
        "support target must be rejected"
    );
    assert!(
        String::from_utf8_lossy(&rejected.stderr)
            .contains("must match exactly one maintained workspace route"),
        "{}",
        receipt(&rejected)
    );
}

#[test]
fn workspace_contract_rejects_unrouted_nested_org_file() {
    let fixture = WorkspaceFixture::new();
    fs::create_dir_all(fixture.root.join("cn/docs/nested")).unwrap();
    fs::write(
        fixture.root.join("cn/docs/nested/rogue.org"),
        complete_document(
            "Rogue",
            "zh-CN",
            "test.rogue",
            "../../../en/docs/doc.org",
            "test.base.v1 test.purpose.cn.v1",
        ),
    )
    .unwrap();

    fixture.assert_failure("expected exactly one workspace route, matched 0");
}

#[test]
fn workspace_contract_rejects_a_second_path_pattern_language() {
    let fixture = WorkspaceFixture::new();
    fs::write(
        fixture.root.join("policy.org"),
        POLICY.replace(":PATH: cn/docs", ":PATH: cn/docs/*.org"),
    )
    .unwrap();

    fixture.assert_failure("PATH does not accept wildcard syntax");
}

#[test]
fn workspace_contract_rejects_inexact_contract_composition() {
    let fixture = WorkspaceFixture::new();
    fs::write(
        fixture.root.join("cn/docs/doc.org"),
        complete_document(
            "CN",
            "zh-CN",
            "test.semantic",
            "../../en/docs/doc.org",
            "test.base.v1",
        ),
    )
    .unwrap();

    fixture.assert_failure("exact contract composition must be");
}

#[test]
fn workspace_contract_rejects_nonreciprocal_and_escaping_counterparts() {
    let fixture = WorkspaceFixture::new();
    fs::write(fixture.root.join("outside.org"), "#+TITLE: Outside\n").unwrap();
    fs::write(
        fixture.root.join("cn/docs/doc.org"),
        complete_document(
            "CN",
            "zh-CN",
            "test.semantic",
            "../../outside.org",
            "test.base.v1 test.purpose.cn.v1",
        ),
    )
    .unwrap();
    fixture.assert_failure("counterpart escapes `en`");

    fs::write(
        fixture.root.join("cn/docs/doc.org"),
        complete_document(
            "CN",
            "zh-CN",
            "test.semantic",
            "../../en/docs/doc.org",
            "test.base.v1 test.purpose.cn.v1",
        ),
    )
    .unwrap();
    fs::write(
        fixture.root.join("en/docs/doc.org"),
        complete_document(
            "EN",
            "en",
            "test.semantic",
            "../../cn/docs/other.org",
            "test.base.v1 test.purpose.en.v1",
        ),
    )
    .unwrap();
    fixture.assert_failure("counterpart relation is not reciprocal");
}

#[test]
fn workspace_contract_rejects_third_semantic_projection() {
    let fixture = WorkspaceFixture::new();
    fs::write(
        fixture.root.join("cn/docs/duplicate.org"),
        complete_document(
            "CN duplicate",
            "zh-CN",
            "test.semantic",
            "../../en/docs/duplicate.org",
            "test.base.v1 test.purpose.cn.v1",
        ),
    )
    .unwrap();
    fs::write(
        fixture.root.join("en/docs/duplicate.org"),
        complete_document(
            "EN duplicate",
            "en",
            "test.semantic",
            "../../cn/docs/duplicate.org",
            "test.base.v1 test.purpose.en.v1",
        ),
    )
    .unwrap();

    fixture.assert_failure("must identify exactly two documents; found 4");
}

#[test]
fn workspace_contract_rejects_mismatched_paired_node_identities() {
    let fixture = WorkspaceFixture::new();
    let policy = POLICY.replace(
        ":PAIR_GROUP: test.cn-en",
        ":PAIR_GROUP: test.cn-en\n:PAIR_NODE_ID_PROPERTY: PRINCIPLE_ID",
    );
    fs::write(fixture.root.join("policy.org"), policy).unwrap();
    fs::write(
        fixture.root.join("cn/docs/doc.org"),
        complete_document(
            "CN",
            "zh-CN",
            "test.semantic",
            "../../en/docs/doc.org",
            "test.base.v1 test.purpose.cn.v1",
        )
        .replace(
            "* Required\n",
            "* Required\n:PROPERTIES:\n:PRINCIPLE_ID: P-001\n:END:\n",
        ),
    )
    .unwrap();
    fs::write(
        fixture.root.join("en/docs/doc.org"),
        complete_document(
            "EN",
            "en",
            "test.semantic",
            "../../cn/docs/doc.org",
            "test.base.v1 test.purpose.en.v1",
        )
        .replace(
            "* Required\n",
            "* Required\n:PROPERTIES:\n:PRINCIPLE_ID: P-002\n:END:\n",
        ),
    )
    .unwrap();

    fixture.assert_failure("must have identical unique paired node identities in PRINCIPLE_ID");
}

#[test]
fn workspace_contract_rejects_mismatched_paired_node_metadata_from_contract_expression() {
    let fixture = WorkspaceFixture::new();
    let pair_contract = r#"
#+begin_src org-contract
(assert pair-node-properties-equal
  (identity "PRINCIPLE_ID")
  (properties "PRINCIPLE_STATUS" "REVISION"))
#+end_src"#;
    let policy = POLICY
        .replace(
            ":LANGUAGE_VALUE: zh-CN\n:END:",
            &format!(":LANGUAGE_VALUE: zh-CN\n:END:\n{pair_contract}"),
        )
        .replace(
            ":LANGUAGE_VALUE: en\n:END:",
            &format!(":LANGUAGE_VALUE: en\n:END:\n{pair_contract}"),
        );
    fs::write(fixture.root.join("policy.org"), policy).unwrap();
    for (directory, language, counterpart, status) in [
        ("cn", "zh-CN", "../../en/docs/doc.org", "proposed"),
        ("en", "en", "../../cn/docs/doc.org", "proposed"),
    ] {
        fs::write(
            fixture.root.join(format!("{directory}/docs/doc.org")),
            complete_document(
                directory,
                language,
                "test.semantic",
                counterpart,
                &format!("test.base.v1 test.purpose.{directory}.v1"),
            )
            .replace(
                "* Required\n",
                &format!("* Required\n:PROPERTIES:\n:PRINCIPLE_ID: P-001\n:PRINCIPLE_STATUS: {status}\n:REVISION: 1\n:END:\n"),
            ),
        )
        .unwrap();
    }

    let admitted = fixture.run();
    assert!(admitted.status.success(), "{}", receipt(&admitted));
    fs::write(
        fixture.root.join("en/docs/doc.org"),
        complete_document(
            "en",
            "en",
            "test.semantic",
            "../../cn/docs/doc.org",
            "test.base.v1 test.purpose.en.v1",
        )
        .replace(
            "* Required\n",
            "* Required\n:PROPERTIES:\n:PRINCIPLE_ID: P-001\n:PRINCIPLE_STATUS: accepted\n:REVISION: 1\n:END:\n",
        ),
    )
    .unwrap();

    fixture.assert_failure("paired node `P-001` must have equal PRINCIPLE_STATUS metadata");
}

#[test]
fn workspace_contract_rejects_mismatched_paired_document_metadata_from_contract_expression() {
    let fixture = WorkspaceFixture::new();
    let pair_contract = r#"
#+begin_src org-contract
(assert pair-document-properties-equal
  (properties "PRINCIPLE_REF"))
#+end_src"#;
    let policy = POLICY
        .replace(
            ":LANGUAGE_VALUE: zh-CN\n:END:",
            &format!(":LANGUAGE_VALUE: zh-CN\n:END:\n{pair_contract}"),
        )
        .replace(
            ":LANGUAGE_VALUE: en\n:END:",
            &format!(":LANGUAGE_VALUE: en\n:END:\n{pair_contract}"),
        );
    fs::write(fixture.root.join("policy.org"), policy).unwrap();
    for (directory, language, counterpart) in [
        ("cn", "zh-CN", "../../en/docs/doc.org"),
        ("en", "en", "../../cn/docs/doc.org"),
    ] {
        fs::write(
            fixture.root.join(format!("{directory}/docs/doc.org")),
            complete_document(
                directory,
                language,
                "test.semantic",
                counterpart,
                &format!("test.base.v1 test.purpose.{directory}.v1"),
            )
            .replace(":LANGUAGE:", ":PRINCIPLE_REF: P-001\n:LANGUAGE:"),
        )
        .unwrap();
    }

    let admitted = fixture.run();
    assert!(admitted.status.success(), "{}", receipt(&admitted));
    let en_path = fixture.root.join("en/docs/doc.org");
    fs::write(
        &en_path,
        fs::read_to_string(&en_path)
            .unwrap()
            .replace(":PRINCIPLE_REF: P-001", ":PRINCIPLE_REF: P-002"),
    )
    .unwrap();

    fixture.assert_failure("must have equal document property PRINCIPLE_REF");
}

#[test]
fn workspace_contract_resolves_document_property_values_to_node_identities() {
    let fixture = WorkspaceFixture::new();
    fixture.install_reference_policy();
    fixture.add_reference_fixture("P-001", "none");

    let admitted = fixture.run();
    assert!(admitted.status.success(), "{}", receipt(&admitted));

    let cn_path = fixture.root.join("cn/docs/doc.org");
    fs::write(
        &cn_path,
        fs::read_to_string(&cn_path)
            .unwrap()
            .replace(":PRINCIPLE_REF: P-001", ":PRINCIPLE_REF: P-404"),
    )
    .unwrap();
    fixture.assert_failure(
        "document property PRINCIPLE_REF reference `P-404` does not resolve to node identity PRINCIPLE_ID",
    );
}

#[test]
fn workspace_contract_resolves_node_property_values_and_allows_declared_sentinels() {
    let fixture = WorkspaceFixture::new();
    fixture.install_reference_policy();
    fixture.add_reference_fixture("P-001", "none");

    let admitted = fixture.run();
    assert!(admitted.status.success(), "{}", receipt(&admitted));

    let en_path = fixture.root.join("en/docs/doc.org");
    fs::write(
        &en_path,
        fs::read_to_string(&en_path)
            .unwrap()
            .replace(":REFINES: none", ":REFINES: P-404"),
    )
    .unwrap();
    fixture.assert_failure(
        "node property REFINES reference `P-404` does not resolve to node identity PRINCIPLE_ID",
    );
}

#[test]
fn workspace_contract_requires_reciprocal_node_references() {
    let fixture = WorkspaceFixture::new();
    fixture.install_reference_policy();
    fixture.add_reciprocal_fixture();

    let admitted = fixture.run();
    assert!(admitted.status.success(), "{}", receipt(&admitted));

    for language in ["cn", "en"] {
        let path = fixture.root.join(language).join("docs/doc.org");
        fs::write(
            &path,
            fs::read_to_string(&path)
                .unwrap()
                .replace(":SUPERSEDED_BY: P-002", ":SUPERSEDED_BY: none"),
        )
        .unwrap();
    }
    fixture.assert_failure(
        "node `P-002` property SUPERSEDES reference `P-001` must be reciprocated by target property SUPERSEDED_BY",
    );
}

#[test]
fn workspace_contract_rejects_an_invalid_route_contract_expression() {
    let fixture = WorkspaceFixture::new();
    let policy = POLICY.replace(
        ":LANGUAGE_VALUE: zh-CN\n:END:",
        ":LANGUAGE_VALUE: zh-CN\n:END:\n#+begin_src org-contract\n(assert pair-node-properties-equal (identity \"PRINCIPLE_ID\"))\n#+end_src",
    );
    fs::write(fixture.root.join("policy.org"), policy).unwrap();

    fixture.assert_failure("contains an unsupported org-contract expression");
}

#[test]
fn workspace_contract_scale_scenario_stays_in_budget() {
    let scenario_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/unit/scenarios/contract_workspace/workspace_admission_scale");
    let benchmark = asp_rust::validate_rust_scenario_benchmark(&scenario_root)
        .expect("validate workspace admission scale scenario benchmark");
    assert_eq!(
        benchmark.status,
        asp_rust::RustScenarioBenchmarkStatus::Pass,
        "{:?}",
        benchmark.violations
    );

    let fixture = WorkspaceFixture::new();
    for index in 1..128 {
        fixture.write_pair(index);
    }
    let scenario = asp_rust_scenario! {
        name: "workspace-admission-scale",
        package: "orgize",
        description: "A pure Org AST policy admits 256 bilingual documents through one exact-path workspace scan",
        fixture_root: "tests/unit/scenarios/contract_workspace/workspace_admission_scale",
        tags: ["org-contract", "workspace", "performance"],
        commands: [
            { label: "focused", argv: ["cargo", "test", "workspace_contract_scale_scenario_stays_in_budget"] }
        ],
        benchmark: {
            harness: "libtest",
            test: "contract_workspace::workspace_contract_scale_scenario_stays_in_budget",
            snapshot: "workspace_admission_scale",
            target_total: "200ms",
            max_total: "400ms",
            regression_budget: "80ms",
            memory_budget_bytes: 33_554_432,
            target_rationale: "A 256-document bilingual repository must remain below the ASP Rust sub-500ms hard ceiling.",
            warmup_iterations: 1,
            measure_iterations: 3,
            metrics: [
                { name: "document_count", unit: "count", kind: Exact, target: 256 },
                { name: "contract_evaluation_count", unit: "count", kind: Exact, target: 512 },
                { name: "provider_process_count", unit: "count", kind: Exact, target: 0 }
            ]
        }
    };
    let mut last_output = None;
    let measurement = measure_asp_rust_scenario(&scenario, || {
        let output = fixture.run();
        assert!(output.status.success(), "{}", receipt(&output));
        last_output = Some(output);
        AspRustScenarioObservation::default()
            .with_metric("document_count", 256)
            .with_metric("contract_evaluation_count", 512)
            .with_metric("provider_process_count", 0)
    })
    .expect("measure workspace admission through the ASP Rust Scenario macro");

    assert!(
        measurement.observed_total < benchmark.benchmark.max_total.as_duration(),
        "workspace admission exceeded {}ms gate for 256 documents: {:?}",
        benchmark.benchmark.max_total.as_duration().as_millis(),
        measurement.observed_total,
    );
    let output = last_output.expect("Scenario macro records the last workspace receipt");
    assert!(String::from_utf8_lossy(&output.stdout).contains("256 documents, 512 evaluations"));
}

struct WorkspaceFixture {
    root: PathBuf,
}

impl WorkspaceFixture {
    fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("orgize-cli-tests")
            .join(format!(
                "contract-workspace-{}-{}",
                std::process::id(),
                NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
            ));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(root.join("cn/docs")).unwrap();
        fs::create_dir_all(root.join("en/docs")).unwrap();
        fs::write(root.join("policy.org"), POLICY).unwrap();
        fs::write(root.join("contracts.org"), REGISTRY).unwrap();
        fs::write(
            root.join("cn/docs/doc.org"),
            complete_document(
                "CN",
                "zh-CN",
                "test.semantic",
                "../../en/docs/doc.org",
                "test.base.v1 test.purpose.cn.v1",
            ),
        )
        .unwrap();
        fs::write(
            root.join("en/docs/doc.org"),
            complete_document(
                "EN",
                "en",
                "test.semantic",
                "../../cn/docs/doc.org",
                "test.base.v1 test.purpose.en.v1",
            ),
        )
        .unwrap();
        Self { root }
    }

    fn run(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_orgize"))
            .args([
                "contract",
                "workspace",
                "--root",
                path(&self.root),
                "--policy",
                path(&self.root.join("policy.org")),
                "--org-contract-registry",
                path(&self.root.join("contracts.org")),
            ])
            .output()
            .unwrap()
    }

    fn run_requiring(&self, target: &Path) -> Output {
        Command::new(env!("CARGO_BIN_EXE_orgize"))
            .args([
                "contract",
                "workspace",
                "--root",
                path(&self.root),
                "--policy",
                path(&self.root.join("policy.org")),
                "--org-contract-registry",
                path(&self.root.join("contracts.org")),
                "--require-maintained",
                path(target),
            ])
            .output()
            .unwrap()
    }

    fn write_pair(&self, index: usize) {
        let file_name = format!("doc-{index:03}.org");
        let semantic_id = format!("test.semantic.{index:03}");
        fs::write(
            self.root.join("cn/docs").join(&file_name),
            complete_document(
                "CN",
                "zh-CN",
                &semantic_id,
                &format!("../../en/docs/{file_name}"),
                "test.base.v1 test.purpose.cn.v1",
            ),
        )
        .unwrap();
        fs::write(
            self.root.join("en/docs").join(&file_name),
            complete_document(
                "EN",
                "en",
                &semantic_id,
                &format!("../../cn/docs/{file_name}"),
                "test.base.v1 test.purpose.en.v1",
            ),
        )
        .unwrap();
    }

    fn assert_failure(&self, expected: &str) {
        let output = self.run();
        assert!(!output.status.success(), "workspace admission must fail");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected),
            "expected `{expected}`\n{}",
            receipt(&output)
        );
    }

    fn install_reference_policy(&self) {
        let document_rule = r#"#+begin_src org-contract
(assert document-property-values-resolve-node-identities
  (property "PRINCIPLE_REF")
  (identity "PRINCIPLE_ID"))
#+end_src"#;
        let node_rule = r#"#+begin_src org-contract
(assert node-property-values-resolve-node-identities
  (property "REFINES")
  (identity "PRINCIPLE_ID")
  (allow "none"))
#+end_src"#;
        let reciprocal_rule = r#"#+begin_src org-contract
(assert node-property-values-have-reciprocal-node-property
  (property "SUPERSEDES")
  (identity "PRINCIPLE_ID")
  (reciprocal "SUPERSEDED_BY")
  (allow "none"))
#+end_src"#;
        let policy = POLICY
            .replace(
                ":LANGUAGE_VALUE: zh-CN\n:END:",
                &format!(
                    ":LANGUAGE_VALUE: zh-CN\n:END:\n{document_rule}\n{node_rule}\n{reciprocal_rule}"
                ),
            )
            .replace(
                ":LANGUAGE_VALUE: en\n:END:",
                &format!(
                    ":LANGUAGE_VALUE: en\n:END:\n{document_rule}\n{node_rule}\n{reciprocal_rule}"
                ),
            );
        fs::write(self.root.join("policy.org"), policy).unwrap();
    }

    fn add_reference_fixture(&self, principle_ref: &str, refines: &str) {
        for language in ["cn", "en"] {
            let path = self.root.join(language).join("docs/doc.org");
            let source = fs::read_to_string(&path).unwrap();
            let source = source
                .replace(
                    ":LANGUAGE:",
                    &format!(":PRINCIPLE_REF: {principle_ref}\n:LANGUAGE:"),
                )
                .replace(
                    "* Required\n",
                    &format!(
                        "* Required\n:PROPERTIES:\n:PRINCIPLE_ID: P-001\n:REFINES: {refines}\n:END:\n"
                    ),
                );
            fs::write(path, source).unwrap();
        }
    }

    fn add_reciprocal_fixture(&self) {
        for language in ["cn", "en"] {
            let path = self.root.join(language).join("docs/doc.org");
            let source = fs::read_to_string(&path).unwrap().replace(
                "* Required\n",
                "* Required\n:PROPERTIES:\n:PRINCIPLE_ID: P-001\n:SUPERSEDES: none\n:SUPERSEDED_BY: P-002\n:END:\n* Other\n:PROPERTIES:\n:PRINCIPLE_ID: P-002\n:SUPERSEDES: P-001\n:SUPERSEDED_BY: none\n:END:\n",
            );
            fs::write(path, source).unwrap();
        }
    }
}

fn path(path: &Path) -> &str {
    path.to_str().unwrap()
}

fn complete_document(
    title: &str,
    language: &str,
    semantic_id: &str,
    counterpart: &str,
    contracts: &str,
) -> String {
    format!(
        "#+TITLE: {title}\n:PROPERTIES:\n:CONTRACT_ORG: {contracts}\n:SEMANTIC_ID: {semantic_id}\n:COUNTERPART: {counterpart}\n:LANGUAGE: {language}\n:END:\n* Required\n"
    )
}

fn receipt(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status.code().unwrap_or_default(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

const POLICY: &str = r#"#+TITLE: Workspace policy
:PROPERTIES:
:WORKSPACE_CONTRACT_ID: test.workspace.v1
:IGNORE_DIRS: .git target
:END:
* Contract registry
:PROPERTIES:
:PATH: contracts.org
:PATH_KIND: file
:ROLE: support
:END:
* Policy
:PROPERTIES:
:PATH: policy.org
:PATH_KIND: file
:ROLE: support
:END:
* Outside fixture
:PROPERTIES:
:PATH: outside.org
:PATH_KIND: file
:ROLE: support
:END:
* Chinese documents
:PROPERTIES:
:PATH: cn/docs
:PATH_KIND: directory
:ROLE: maintained
:CONTRACT_ORG_EXACT: test.base.v1 test.purpose.cn.v1
:PAIR_GROUP: test.cn-en
:LANGUAGE_ROOT: cn
:COUNTERPART_ROOT: en
:LANGUAGE_VALUE: zh-CN
:END:
* English documents
:PROPERTIES:
:PATH: en/docs
:PATH_KIND: directory
:ROLE: maintained
:CONTRACT_ORG_EXACT: test.base.v1 test.purpose.en.v1
:PAIR_GROUP: test.cn-en
:LANGUAGE_ROOT: en
:COUNTERPART_ROOT: cn
:LANGUAGE_VALUE: en
:END:
"#;

const REGISTRY: &str = r#"* Base
:PROPERTIES:
:CONTRACT_ID: test.base.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** Title
:PROPERTIES:
:ASSERT_ID: test.base.has-title
:SEVERITY: error
:END:
#+begin_src org-contract
(assert exists (keyword :context "metadata" :summary (key "TITLE")))
#+end_src
* Chinese purpose
:PROPERTIES:
:CONTRACT_ID: test.purpose.cn.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** Required heading
:PROPERTIES:
:ASSERT_ID: test.purpose.cn.has-required
:SEVERITY: error
:END:
#+begin_src org-contract
(assert exists (headline :summary (title "Required")))
#+end_src
* English purpose
:PROPERTIES:
:CONTRACT_ID: test.purpose.en.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** Required heading
:PROPERTIES:
:ASSERT_ID: test.purpose.en.has-required
:SEVERITY: error
:END:
#+begin_src org-contract
(assert exists (headline :summary (title "Required")))
#+end_src
"#;
