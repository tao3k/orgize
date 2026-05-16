use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rust_lang_project_harness::{
    build_rust_verification_analysis_profile_with_config,
    build_rust_verification_report_entry_advice_with_receipt, default_rust_harness_config,
    plan_rust_project_verification_with_config, render_rust_verification_report_write_receipt_json,
    write_rust_verification_reports_with_options, RustOwnerResponsibility,
    RustVerificationProfileHint, RustVerificationReportBundle, RustVerificationReportEntryAction,
    RustVerificationReportEntryArtifact, RustVerificationReportOptions,
    RustVerificationReportWriteConfig, RustVerificationReportWriteReceipt,
    RustVerificationSkillBinding, RustVerificationTaskKind,
};

#[test]
fn git_locked_harness_report_exposes_actionable_performance_gaps() {
    let temp = TempProject::new("orgize-harness-report-consumer");
    write_sample_project(temp.path());

    let config = default_rust_harness_config()
        .with_verification_profile_hint(RustVerificationProfileHint::new(
            "src/api.rs",
            [RustOwnerResponsibility::LatencySensitive],
        ))
        .with_verification_skill_binding(
            RustVerificationTaskKind::Performance,
            RustVerificationSkillBinding::new("rust-verification-performance")
                .with_adapter("criterion"),
        );
    let plan = plan_rust_project_verification_with_config(temp.path(), &config).expect("plan");
    let profile = build_rust_verification_analysis_profile_with_config(temp.path(), &config)
        .expect("analysis profile");
    let options = RustVerificationReportOptions::default()
        .with_analysis_profile_artifact()
        .with_selection_advice_sidecar();
    let source_dir = temp.path().join("resources/verification/reports");
    let cache_dir = temp.path().join(".cache/agent/verification/sample");
    let receipt = write_rust_verification_reports_with_options(
        &plan,
        &config,
        &RustVerificationReportWriteConfig::new(temp.path(), &source_dir, &cache_dir),
        &options,
    )
    .expect("write reports");

    let runtime_manifest = fs::read_to_string(cache_dir.join("verification_report_manifest.json"))
        .expect("runtime manifest");
    let manifest: RustVerificationReportBundle =
        serde_json::from_str(&runtime_manifest).expect("decode runtime manifest");
    let receipt_json =
        render_rust_verification_report_write_receipt_json(&receipt).expect("receipt json");
    let receipt: RustVerificationReportWriteReceipt =
        serde_json::from_str(&receipt_json).expect("decode receipt");
    let entry = build_rust_verification_report_entry_advice_with_receipt(
        &manifest,
        Some(&profile),
        Some(&receipt),
    );
    let first_artifact: RustVerificationReportEntryArtifact =
        entry.first_artifact.expect("first artifact path");
    let performance_report =
        fs::read_to_string(&first_artifact.path).expect("selected performance report");
    let performance_json: serde_json::Value =
        serde_json::from_str(&performance_report).expect("performance report json");
    let records = performance_json["records"]
        .as_array()
        .expect("performance records");
    let required_evidence = records[0]["required_evidence_keys"]
        .as_array()
        .expect("required evidence");

    assert_eq!(
        entry.action,
        RustVerificationReportEntryAction::LoadSelectionAdviceSidecar
    );
    assert_eq!(first_artifact.key, "performance_index_json");
    assert_eq!(
        receipt.artifact_path("performance_index_json"),
        Some(&source_dir.join("performance_index.json"))
    );
    assert_eq!(
        receipt.sidecar_path("selection_advice_json"),
        Some(&cache_dir.join("selection_advice.json"))
    );
    assert_eq!(records[0]["state"], "pending");
    assert!(required_evidence
        .iter()
        .any(|key| key == "benchmark_command"));
    assert!(required_evidence.iter().any(|key| key == "baseline"));
    assert!(required_evidence
        .iter()
        .any(|key| key == "regression_threshold"));
}

#[test]
fn verification_report_docs_keep_dense_parser_backlog_consumable() {
    let closeout = include_str!("../../docs/20_parser/20.02_parser_v2_performance_closeout.org");
    let consumption =
        include_str!("../../docs/90_operations/90.02_verification_report_consumption.org");
    let dense_surfaces = [
        "Org::macro_expansions/dense-macro-expansions.org",
        "Org::document/dense-target-projection/many-targets-and-radio-links.org",
        "Org::document/dense-annotation-projection/many-annotated-ascii-objects.org",
        "Org::document/dense-semantic-radio-projection/many-parsed-object-radio-links.org",
        "Org::document/dense-m15-side-tables/many-m15-settings-links-footnotes.org",
        "Document::project_for_export/dense-m15/many-m15-settings-links-footnotes.org",
        "Document::agenda_entries/dense-agenda/many-agenda-planning-timestamps.org",
        "Document::include-datetree-agenda-extras/dense/include-expansion-plan.org",
        "Document::include-datetree-agenda-extras/dense/datetree-entries.org",
        "Document::include-datetree-agenda-extras/dense/agenda-inactive-diary.org",
    ];

    assert!(consumption.contains(
        "| Surface | Benchmark command | Baseline evidence | Regression threshold | Next action |"
    ));
    for key in ["benchmark_command", "baseline", "regression_threshold"] {
        assert!(
            consumption.contains(key),
            "missing required evidence key {key}"
        );
    }
    for surface in dense_surfaces {
        assert!(
            closeout.contains(surface),
            "closeout is missing dense surface {surface}"
        );
        assert!(
            consumption.contains(surface),
            "consumption backlog is missing dense surface {surface}"
        );
    }
    assert!(consumption.contains("dense-agenda -- --sample-size 10"));
    assert!(consumption.contains("include-datetree -- --sample-size 10"));
    assert!(consumption.contains("pending | Calibrate"));
}

struct TempProject {
    path: PathBuf,
}

impl TempProject {
    fn new(prefix: &str) -> Self {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{}-{suffix}", std::process::id()));
        fs::create_dir_all(&path).expect("create temp project");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn write_sample_project(root: &Path) {
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"report-consumer-sample\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write manifest");
    fs::create_dir(root.join("src")).expect("create src");
    fs::write(root.join("src/lib.rs"), "//! Test crate.\nmod api;\n").expect("write lib");
    fs::write(
        root.join("src/api.rs"),
        "//! API owner.\npub fn handle_request() {}\n",
    )
    .expect("write api");
}
