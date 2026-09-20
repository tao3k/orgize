use std::{fs, path::PathBuf};

use asp_rust_build_support::{
    AspRustScenarioObservation, asp_rust_scenario, measure_asp_rust_scenario,
};

use super::elements::{
    filter_elements_by_query, query_project_with_config, should_index_sequentially,
};
use super::model::{DocumentLanguage, DocumentWalkConfig};

#[test]
fn document_query_org_elements_ast_stays_inside_scenario_gate() {
    let scenario_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/unit/scenarios/document_query_org_elements_ast");
    let benchmark = asp_rust::validate_rust_scenario_benchmark(&scenario_root)
        .expect("validate Org element query scenario benchmark");
    assert_eq!(
        benchmark.status,
        asp_rust::RustScenarioBenchmarkStatus::Pass,
        "{:?}",
        benchmark.violations
    );
    let scenario = asp_rust_scenario! {
        name: "document-query-org-elements-ast",
        package: "orgize",
        description: "Forty-eight Org documents parse to one parser-owned element projection and answer a no-hit query",
        fixture_root: "tests/unit/scenarios/document_query_org_elements_ast",
        tags: ["org-elements", "query", "performance"],
        commands: [
            { label: "focused", argv: ["cargo", "test", "document_query_org_elements_ast_stays_inside_scenario_gate"] }
        ],
        benchmark: {
            harness: "libtest",
            test: "document_query_org_elements_ast_stays_inside_scenario_gate",
            snapshot: "org_elements_ast_query",
            target_total: "5ms",
            max_total: "45ms",
            regression_budget: "40ms",
            memory_budget_bytes: 8_388_608,
            target_rationale: "Forty-eight Org documents must parse to one parser-owned element projection and answer a no-hit query inside the strict gate.",
            warmup_iterations: 0,
            measure_iterations: 3,
            metrics: [
                { name: "document_count", unit: "count", kind: Exact, target: 48 },
                { name: "provider_process_count", unit: "count", kind: Exact, target: 0 }
            ]
        }
    };

    let root = temp_document_root("orgize-query-org-elements");
    let mut paths = Vec::new();
    for index in 0..48 {
        let path = root.join(format!("note-{index}.org"));
        fs::write(
            &path,
            format!(
                "* Note {index}\n:PROPERTIES:\n:REVISION: {}\n:END:\nParser-owned body.\n",
                index + 1
            ),
        )
        .expect("write Org fixture");
        paths.push(path);
    }
    assert!(should_index_sequentially(DocumentLanguage::Org, &paths));

    let measurement = measure_asp_rust_scenario(&scenario, || {
        let facts = query_project_with_config(
            DocumentLanguage::Org,
            &root,
            &DocumentWalkConfig::default(),
            &["document_query_absent_fixture".to_string()],
            &[],
        )
        .expect("index parser-owned Org elements");
        let matches = filter_elements_by_query(
            facts,
            &["document_query_absent_fixture".to_string()],
            &[],
            &[],
        );
        assert!(matches.is_empty(), "matches={matches:#?}");
        AspRustScenarioObservation::default()
            .with_metric("document_count", 48)
            .with_metric("provider_process_count", 0)
    })
    .expect("measure parser-owned Org element query through ASP Rust");

    assert!(
        measurement.total_max <= benchmark.benchmark.max_total.as_duration(),
        "Org AST query exceeded max_total={:?}: p50={:?}, p95={:?}, max={:?}",
        benchmark.benchmark.max_total.as_duration(),
        measurement.total_p50,
        measurement.observed_total,
        measurement.total_max,
    );
    fs::remove_dir_all(root).expect("remove query fixture");
}

#[test]
fn document_query_parallelizes_sizable_org_and_markdown_batches() {
    let root = temp_document_root("orgize-query-large-elements");
    let payload = "x".repeat(4097);
    let mut paths = Vec::new();
    for index in 0..16 {
        let path = root.join(format!("note-{index}.org"));
        fs::write(&path, &payload).expect("write large document fixture");
        paths.push(path);
    }

    assert!(!should_index_sequentially(DocumentLanguage::Org, &paths));
    assert!(!should_index_sequentially(
        DocumentLanguage::Markdown,
        &paths
    ));
    fs::remove_dir_all(root).expect("remove large query fixture");
}

fn temp_document_root(prefix: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    ));
    fs::create_dir_all(&root).expect("create temp document root");
    root
}
