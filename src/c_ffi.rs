//! Explicit owned handoff for the C SDK; recognition and contracts remain Scheme-owned.

#[derive(Debug)]
pub struct ContractRow {
    pub id: i64,
    pub parent_id: i64,
    pub kind: String,
    pub field_name: String,
    pub field_value: String,
}

#[derive(Debug)]
pub struct ContractInput {
    pub rows: Vec<ContractRow>,
    pub scope_id: i64,
    pub kind: String,
    pub field_name: String,
    pub field_value: String,
    pub expectation: u32,
    pub expected_count: u32,
}

#[derive(Debug)]
pub struct ContractOutput {
    pub matched_count: u32,
    pub passed: bool,
}

/// Execute an admitted C SDK request on the same owner as native Org parsing.
/// Callers hand off owned data; ABI borrows remain live through completion.
pub fn evaluate_contract(input: ContractInput) -> Result<ContractOutput, String> {
    crate::org_aot::evaluate_native_contract(input)
}

/// Project a private native semantic plan through the existing owned handoff.
/// Scheme validates the closed operation/argument schema and owns recognition.
/// This is an interop boundary; the normal document API remains `Org::parse`.
pub fn project_native_semantic_rows(operation: u8, fields: &[&str]) -> Result<Vec<Vec<String>>, String> {
    crate::org_aot::native_semantic_rows(operation, fields)
}
