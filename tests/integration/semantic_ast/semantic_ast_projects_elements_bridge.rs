#[path = "semantic_ast_projects_elements_bridge_fixtures.rs"]
mod semantic_ast_projects_elements_bridge_fixtures;
#[path = "semantic_ast_projects_elements_bridge_indexing.rs"]
mod semantic_ast_projects_elements_bridge_indexing;
#[path = "semantic_ast_projects_elements_bridge_query_cases.rs"]
mod semantic_ast_projects_elements_bridge_query_cases;
#[path = "semantic_ast_projects_elements_bridge_resolution.rs"]
mod semantic_ast_projects_elements_bridge_resolution;

pub(super) fn cases() -> Vec<(&'static str, fn())> {
    let mut cases = Vec::new();
    for group in [
        semantic_ast_projects_elements_bridge_fixtures::NATIVE_CASES,
        semantic_ast_projects_elements_bridge_indexing::NATIVE_CASES,
        semantic_ast_projects_elements_bridge_query_cases::NATIVE_CASES,
        semantic_ast_projects_elements_bridge_resolution::NATIVE_CASES,
    ] {
        cases.extend_from_slice(group);
    }
    cases
}
