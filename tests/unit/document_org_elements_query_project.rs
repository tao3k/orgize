use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use super::elements::{filter_elements_by_query, query_project_with_config};
use super::model::{DocumentLanguage, DocumentWalkConfig};

const SCENARIO_ID: &str = "document-query-org-elements-ast";
const SCENARIO_ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/unit/scenarios/document_query_org_elements_ast"
);

#[test]
fn document_query_org_elements_ast_stays_inside_scenario_gate() {
    let scenario = fs::read_to_string(Path::new(SCENARIO_ROOT).join("scenario.toml"))
        .expect("read scenario manifest");
    let benchmark = fs::read_to_string(Path::new(SCENARIO_ROOT).join("benchmark.toml"))
        .expect("read benchmark manifest");
    assert!(scenario.contains(&format!("id = \"{SCENARIO_ID}\"")));
    assert!(scenario.contains("ORGIZE-AGENT-ASP-PERF-SUBCOMMAND-QUERY-ORG-ELEMENTS-001"));
    let max_total = duration_from_manifest(&benchmark, "max_total");

    let root = temp_document_root("orgize-query-org-elements");
    for index in 0..48 {
        fs::write(
            root.join(format!("note-{index}.org")),
            format!(
                "* Note {index}\n:PROPERTIES:\n:REVISION: {}\n:END:\nParser-owned body.\n",
                index + 1
            ),
        )
        .expect("write Org fixture");
    }

    let started_at = Instant::now();
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
    let elapsed = started_at.elapsed();
    assert!(matches.is_empty(), "matches={matches:#?}");
    assert!(
        elapsed <= max_total,
        "Org AST query exceeded max_total={max_total:?} observed={elapsed:?}"
    );
    fs::remove_dir_all(root).expect("remove query fixture");
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
