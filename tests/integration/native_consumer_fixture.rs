use super::{
    agent_cli, capture_cli, contract_composition, contract_evaluation, contract_registry,
    contract_workspace, contract_workspace_receipt, contract_workspace_reciprocal,
    document_git_scope, eval_cli, export_cli, fmt_cli, fmt_links, fmt_table,
    harness_report_consumer, lint_attachments, lint_babel, lint_builtin_contracts, lint_contract,
    lint_crypt, lint_file_links, lint_fix_cli, lint_fmt, lint_lifecycle, lint_progress,
    lint_property_schema, lint_table_formulas, lint_task_blockers, named_source_block_template,
    org_aot_edit, org_case_insensitive_aot, org_citation_aot, org_customer_contract,
    org_dynamic_block, org_element_query, org_event_aot_contract, org_event_aot_parity,
    org_export_snippet_aot, org_footnote_aot, org_headline_aot, org_headline_function_aot,
    org_inline_code_aot, org_inline_object_aot, org_inlinetask_aot, org_list_aot, org_list_fields,
    org_named_drawer, org_named_elements, org_parser_aot, org_public_aot_boundary, org_script_aot,
    org_timestamp_aot, parse, scenario_benchmark, sdd, semantic_ast, source_block_document,
    task_cli,
};

// All cases share one explicit startup window, before native callers or children.
#[test]
fn explicit_startup_precedes_parallel_consumer_cases() {
    // SAFETY: this is the sole non-ignored test in the aggregate; workers and
    // host children below are created only after startup completes.
    unsafe { orgize::initialize_native_runtime() }.expect("native consumer startup");
    let mut cases: Vec<(&str, fn())> = Vec::new();
    for group in [
        agent_cli::NATIVE_CASES,
        capture_cli::NATIVE_CASES,
        contract_composition::NATIVE_CASES,
        contract_evaluation::NATIVE_CASES,
        contract_registry::NATIVE_CASES,
        contract_workspace::NATIVE_CASES,
        contract_workspace_receipt::NATIVE_CASES,
        contract_workspace_reciprocal::NATIVE_CASES,
        document_git_scope::NATIVE_CASES,
        eval_cli::NATIVE_CASES,
        fmt_cli::NATIVE_CASES,
        fmt_links::NATIVE_CASES,
        fmt_table::NATIVE_CASES,
        harness_report_consumer::NATIVE_CASES,
        lint_attachments::NATIVE_CASES,
        lint_babel::NATIVE_CASES,
        lint_builtin_contracts::NATIVE_CASES,
        lint_contract::NATIVE_CASES,
        lint_crypt::NATIVE_CASES,
        lint_file_links::NATIVE_CASES,
        lint_fix_cli::NATIVE_CASES,
        lint_fmt::NATIVE_CASES,
        lint_lifecycle::NATIVE_CASES,
        lint_progress::NATIVE_CASES,
        lint_property_schema::NATIVE_CASES,
        lint_table_formulas::NATIVE_CASES,
        lint_task_blockers::NATIVE_CASES,
        named_source_block_template::NATIVE_CASES,
        org_aot_edit::NATIVE_CASES,
        org_case_insensitive_aot::NATIVE_CASES,
        org_citation_aot::NATIVE_CASES,
        org_customer_contract::NATIVE_CASES,
        org_dynamic_block::NATIVE_CASES,
        org_element_query::NATIVE_CASES,
        org_event_aot_contract::NATIVE_CASES,
        org_event_aot_parity::NATIVE_CASES,
        org_export_snippet_aot::NATIVE_CASES,
        org_footnote_aot::NATIVE_CASES,
        org_headline_aot::NATIVE_CASES,
        org_headline_function_aot::NATIVE_CASES,
        org_inline_code_aot::NATIVE_CASES,
        org_inline_object_aot::NATIVE_CASES,
        org_inlinetask_aot::NATIVE_CASES,
        org_list_aot::NATIVE_CASES,
        org_list_fields::NATIVE_CASES,
        org_named_drawer::NATIVE_CASES,
        org_named_elements::NATIVE_CASES,
        org_parser_aot::NATIVE_CASES,
        org_public_aot_boundary::NATIVE_CASES,
        org_script_aot::NATIVE_CASES,
        org_timestamp_aot::NATIVE_CASES,
        parse::NATIVE_CASES,
        scenario_benchmark::NATIVE_CASES,
        sdd::NATIVE_CASES,
        source_block_document::NATIVE_CASES,
        task_cli::NATIVE_CASES,
    ] {
        cases.extend_from_slice(group);
    }
    cases.extend(semantic_ast::native_cases());
    cases.extend(export_cli::native_cases());
    cases.extend(contract_workspace::additional_native_cases());
    let catalog_count = cases.len();
    let names: std::collections::HashSet<_> = cases.iter().map(|&(name, _)| name).collect();
    assert_eq!(names.len(), catalog_count, "duplicate consumer case names");
    if !cfg!(any(
        feature = "md",
        feature = "syntax-org-fc",
        feature = "datafusion-sql"
    )) {
        let original: Vec<String> = serde_json::from_str(include_str!(
            "../support/native_consumer_default_cases.json"
        ))
        .expect("original default consumer test catalog");
        let original: std::collections::HashSet<_> = original.iter().map(String::as_str).collect();
        assert_eq!(
            names, original,
            "consumer migration must not lose or invent cases"
        );
    }
    if let Ok(selected) = std::env::var("ORGIZE_NATIVE_CONSUMER_CASE") {
        if !selected.is_empty() {
            cases.retain(|&(name, _)| name == selected);
            assert_eq!(
                cases.len(),
                1,
                "unknown or duplicate consumer case: {selected}"
            );
        }
    }
    let count = cases.len();
    let callers = std::thread::available_parallelism().map_or(1, |count| count.get());
    let chunk_size = count.div_ceil(callers);
    let failures = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for chunk in cases.chunks(chunk_size) {
            let failures = &failures;
            scope.spawn(move || {
                for &(name, case) in chunk {
                    println!("native-consumer case={name} START");
                    if std::panic::catch_unwind(case).is_ok() {
                        println!("native-consumer case={name} OK");
                    } else {
                        println!("native-consumer case={name} FAILED");
                        failures.lock().expect("consumer failures").push(name);
                    }
                }
            });
        }
    });
    let failures = failures.into_inner().expect("consumer failures");
    println!(
        "startup-native consumer-cases={count} callers={callers} completed={count} failed={}",
        failures.len()
    );
    assert!(failures.is_empty(), "failed consumer cases: {failures:?}");
    println!("startup-native consumer-cases={count} callers={callers} complete OK");
}
