use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::elements::query_project_with_config;
use super::model::{DocumentLanguage, DocumentWalkConfig};

const SCENARIO_ID: &str = "document-query-lexical-prefilter-warm-path";
const SCENARIO_ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/unit/scenarios/document_query_lexical_prefilter_warm_path"
);

#[test]
fn document_query_lexical_prefilter_warm_path_stays_inside_scenario_gate() {
    let scenario = fs::read_to_string(Path::new(SCENARIO_ROOT).join("scenario.toml"))
        .expect("read scenario manifest");
    let benchmark = fs::read_to_string(Path::new(SCENARIO_ROOT).join("benchmark.toml"))
        .expect("read benchmark manifest");
    assert!(scenario.contains(&format!("id = \"{SCENARIO_ID}\"")));
    assert!(scenario.contains("ORGIZE-AGENT-ASP-PERF-SUBCOMMAND-QUERY-LEXICAL-PREFILTER-001"));
    assert_benchmark_contract(&benchmark);
    let max_total = duration_from_manifest(&benchmark, "max_total");

    let root = temp_document_root("orgize-query-lexical-prefilter");
    for index in 0..48 {
        fs::write(
            root.join(format!("note-{index}.org")),
            format!("* Note {index}\n\nThis fixture intentionally lacks the searched token.\n"),
        )
        .expect("write org fixture");
    }

    let started_at = Instant::now();
    let facts = query_project_with_config(
        DocumentLanguage::Org,
        &root,
        &DocumentWalkConfig::default(),
        &["document_query_absent_fixture".to_string()],
        &[],
    )
    .expect("query project with lexical prefilter");
    let elapsed = started_at.elapsed();

    assert!(facts.is_empty(), "facts={facts:#?}");
    assert!(
        elapsed <= max_total,
        "query lexical prefilter exceeded max_total={max_total:?} observed={elapsed:?}"
    );

    let _ = fs::remove_dir_all(root);
}

fn assert_benchmark_contract(text: &str) {
    for expected in [
        "harness = \"libtest\"",
        "test = \"document_query_lexical_prefilter_warm_path_stays_inside_scenario_gate\"",
        "route_source = \"lexical-prefilter\"",
        "max_provider_process_count = 0",
        "fallback_reason = \"none\"",
    ] {
        assert!(text.contains(expected), "benchmark missing {expected:?}");
    }
}

fn duration_from_manifest(text: &str, field: &str) -> Duration {
    let prefix = format!("{field} = \"");
    let value = text
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix))
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or_else(|| panic!("benchmark missing duration field {field}"));
    if let Some(value) = value.strip_suffix("ms") {
        return Duration::from_millis(value.parse().expect("parse ms duration"));
    }
    if let Some(value) = value.strip_suffix("us") {
        return Duration::from_micros(value.parse().expect("parse us duration"));
    }
    panic!("unsupported benchmark duration {value:?}");
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
