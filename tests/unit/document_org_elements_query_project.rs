use std::{fs, path::PathBuf};

use asp_rust_build_support::{
    AspRustScenarioObservation, asp_rust_scenario, measure_asp_rust_scenario,
};

use super::elements::{
    filter_elements_by_query, query_project_with_config, should_index_sequentially,
};
use super::model::{DocumentLanguage, DocumentWalkConfig};

#[test]
#[ignore = "run as a focused release-mode performance scenario"]
fn document_query_org_elements_aot_stays_inside_scenario_gate() {
    run_document_query_org_elements_scenario(false, true);
}

#[test]
#[ignore = "same-program native single-document control for bounded batch A/B"]
fn document_query_org_elements_aot_individual_control() {
    run_document_query_org_elements_scenario(false, false);
}

#[cfg(feature = "runtime-profile")]
#[test]
#[ignore = "focused release-mode diagnostic; retains the existing strict scenario gate"]
fn document_query_org_elements_aot_profiles_existing_scenario_gate() {
    run_document_query_org_elements_scenario(true, true);
}

fn run_document_query_org_elements_scenario(profile: bool, batched: bool) {
    // SAFETY: this ignored scenario is run alone, before its indexing workers.
    unsafe { crate::initialize_native_runtime() }.expect("native query scenario startup");
    let scenario_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/unit/scenarios/document_query_org_elements_aot");
    let benchmark = asp_rust::validate_rust_scenario_benchmark(&scenario_root)
        .expect("validate Org element query scenario benchmark");
    assert_eq!(
        benchmark.status,
        asp_rust::RustScenarioBenchmarkStatus::Pass,
        "{:?}",
        benchmark.violations
    );
    let scenario = asp_rust_scenario! {
        name: "document-query-org-elements-aot",
        package: "orgize",
        description: "Forty-eight TAGS-aware Org documents parse to one parser-owned element projection and answer a no-hit query",
        fixture_root: "tests/unit/scenarios/document_query_org_elements_aot",
        tags: ["org-elements", "query", "performance"],
        commands: [
            { label: "focused-release", argv: ["cargo", "test", "--release", "--lib", "document_query_org_elements_aot_stays_inside_scenario_gate", "--", "--ignored"] }
        ],
        benchmark: {
            harness: "libtest",
            test: "document_query_org_elements_aot_stays_inside_scenario_gate",
            snapshot: "org_elements_aot_query",
            target_total: "5ms",
            max_total: "45ms",
            regression_budget: "40ms",
            memory_budget_bytes: 8_388_608,
            target_rationale: "Forty-eight TAGS-aware Org documents must parse to one parser-owned element projection and answer a no-hit query inside the strict gate.",
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
                "#+TAGS: {{ @work(w) @home(h) }} [ GTD : Control Persp ]\n* Note {index}\n:PROPERTIES:\n:REVISION: {}\n:END:\nParser-owned body.\n",
                index + 1
            ),
        )
        .expect("write Org fixture");
        paths.push(path);
    }
    assert!(should_index_sequentially(
        DocumentLanguage::Org,
        paths.len(),
        48 * 96,
    ));

    let observe = || {
        let facts = if batched {
            query_project_with_config(
                DocumentLanguage::Org,
                &root,
                &DocumentWalkConfig::default(),
                &["document_query_absent_fixture".to_string()],
                &[],
            )
            .expect("index parser-owned Org elements")
        } else {
            // Same walking, loading, parser and projection, with one owner
            // handoff per document. No alternate or legacy parser is involved.
            let mut files = Vec::new();
            super::elements::collect_document_paths(
                DocumentLanguage::Org,
                &root,
                &DocumentWalkConfig::default(),
                &mut files,
            )
            .unwrap();
            files.sort();
            files.dedup();
            super::elements::load_sources(&files)
                .unwrap()
                .iter()
                .flat_map(|source| {
                    super::org_elements::index_org(&source.path, &source.source).unwrap()
                })
                .collect()
        };
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
    };
    #[cfg(feature = "runtime-profile")]
    let mut stage_samples = Vec::new();
    let measurement = measure_asp_rust_scenario(&scenario, || {
        #[cfg(feature = "runtime-profile")]
        if profile {
            let (observation, stages) = crate::runtime_profile::measure(|| {
                crate::runtime_profile::stage("query.total_inclusive", observe)
            });
            stage_samples.push(stages);
            return observation;
        }
        let _ = profile;
        observe()
    })
    .expect("measure parser-owned Org element query through ASP Rust");
    eprintln!(
        "QUERY-CONTROL batched={batched} profile={profile} backend={} program={} p50={:?} p95={:?} max={:?}",
        crate::runtime_backend().name(),
        crate::org_aot::org_event_parser_digest(),
        measurement.total_p50,
        measurement.observed_total,
        measurement.total_max,
    );

    #[cfg(feature = "runtime-profile")]
    if profile {
        eprintln!(
            "QUERY-IDENTITY backend={} program={}",
            crate::runtime_backend().name(),
            crate::org_aot::org_event_parser_digest(),
        );
        // Raw totals from the existing request-local collector, not a second sampler.
        // Inclusive stages overlap; they must not be added together.
        for (sample, stages) in stage_samples.iter().enumerate() {
            for required in [
                "query.total_inclusive",
                "native.owner_admission",
                "native.owner_service_inclusive",
                "native.result_copy",
                "native.completion_handoff",
                "native.scheme_fold",
                "native.tape_encode",
                "native.scheme_thread_cpu",
                "rowan.build",
                "graph.project",
            ] {
                assert!(stages.contains_key(required), "missing stage {required}");
            }
            eprintln!("QUERY-PROFILE sample={sample} documents=48 stages_ns={stages:?}");
        }
        assert_eq!(stage_samples.len(), 3);
        eprintln!(
            "QUERY-ASP p50={:?} p95={:?} max={:?}",
            measurement.total_p50, measurement.observed_total, measurement.total_max,
        );
    }
    assert!(
        measurement.total_max <= benchmark.benchmark.max_total.as_duration(),
        "Org AOT query exceeded max_total={:?}: p50={:?}, p95={:?}, max={:?}",
        benchmark.benchmark.max_total.as_duration(),
        measurement.total_p50,
        measurement.observed_total,
        measurement.total_max,
    );
    fs::remove_dir_all(root).expect("remove query fixture");
}

#[test]
fn document_query_parallelizes_sizable_org_and_markdown_batches() {
    assert!(!should_index_sequentially(
        DocumentLanguage::Org,
        16,
        64 * 1024 + 1,
    ));
    assert!(!should_index_sequentially(
        DocumentLanguage::Markdown,
        16,
        1,
    ));
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
