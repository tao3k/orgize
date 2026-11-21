use super::plan_ledger_records_from_paths;
use crate::document::memory_projection::OrgMemorySearchOptions;

#[test]
fn empty_ledger_sources_do_not_require_native_startup() {
    let records = plan_ledger_records_from_paths(&[], &OrgMemorySearchOptions::plan_ledgers())
        .expect("empty source list");
    assert!(records.is_empty());
}

#[test]
fn ledger_reader_failure_disconnects_and_joins_without_native_startup() {
    let missing = std::env::temp_dir().join(format!(
        "orgize-plan-ledger-missing-source-{}",
        std::process::id()
    ));
    assert!(!missing.exists());
    let paths = vec![missing.clone(); 2 * crate::org_aot::BATCH_MAX_DOCUMENTS];
    let error = plan_ledger_records_from_paths(&paths, &OrgMemorySearchOptions::plan_ledgers())
        .expect_err("reader must propagate the source error and finish");
    assert!(error.contains(&missing.display().to_string()), "{error}");
}
