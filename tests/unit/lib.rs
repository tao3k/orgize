//! Test helpers mounted into the crate during `cargo test --lib`.

#[test]
fn explicit_startup_precedes_parallel_native_unit_cases() {
    // SAFETY: no native consumer/child is started before this fixture;
    // application workers are created below, after exclusive startup.
    unsafe { crate::initialize_native_runtime() }.expect("native unit startup");
    let mut case_count = 0;
    std::thread::scope(|scope| {
        for cases in [
            crate::contract_feature::tests::NATIVE_CASES,
            crate::document::block_body_tests::NATIVE_CASES,
            crate::document::org_elements_aot_tests::NATIVE_CASES,
            aot_affiliation::NATIVE_CASES,
            aot_projection::NATIVE_CASES,
            document_source_selection::NATIVE_CASES,
            lint_metadata::NATIVE_CASES,
            org_contract_evaluation::NATIVE_CASES,
            org_contract_reference::NATIVE_CASES,
            org_contract_source_validation::NATIVE_CASES,
            org_elements_query_expr::NATIVE_CASES,
            crate::ast::EXPRESSION_NATIVE_CASES,
            lint_syntax::NATIVE_CASES,
            crate::org_aot::batch_tests::NATIVE_CASES,
        ] {
            for &(name, case) in cases {
                case_count += 1;
                scope.spawn(move || {
                    case();
                    println!("native-unit case={name} OK");
                });
            }
        }
    });
    println!("startup-native unit-cases={case_count} complete OK");
}

#[path = "aot_affiliation.rs"]
mod aot_affiliation;
#[path = "aot_projection.rs"]
mod aot_projection;
#[path = "document_command.rs"]
mod document_command;
#[path = "document_source_selection.rs"]
mod document_source_selection;
#[path = "elements_bridge_query_json.rs"]
mod elements_bridge_query_json;
#[path = "lint_metadata.rs"]
mod lint_metadata;
#[path = "lint_syntax.rs"]
mod lint_syntax;
#[path = "org_contract_evaluation.rs"]
mod org_contract_evaluation;
#[path = "org_contract_reference.rs"]
mod org_contract_reference;
#[path = "org_contract_source_validation.rs"]
mod org_contract_source_validation;
#[path = "org_elements_query_expr.rs"]
mod org_elements_query_expr;
