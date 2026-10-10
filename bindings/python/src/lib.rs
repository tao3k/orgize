//! Thin Python projection of the Scheme-AOT Org parser, functions, and source edits.

mod contract_ffi;

use orgize::org_aot::{OrgAotDocument, parse_org_aot};
use orgize::org_aot_edit::{OrgSourceEdit, apply_org_source_edits, org_source_digest};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// Owned Python input goes directly to the shared Rust/Scheme owner, not C ABI.
#[pyfunction]
fn _evaluate_contract(
    py: Python<'_>,
    rows: Vec<(i64, i64, String, String, String)>,
    scope_id: i64,
    kind: String,
    field_name: String,
    field_value: String,
    expectation: &str,
    expected_count: &Bound<'_, PyAny>,
) -> PyResult<(u32, bool)> {
    use orgize::c_ffi::{ContractInput, ContractRow};
    let mode = match expectation {
        "exactly" => 0,
        "at_least" => 1,
        "at_most" => 2,
        _ => return Err(PyValueError::new_err("invalid Contract expectation")),
    };
    let expected_count = expected_count
        .extract::<u32>()
        .map_err(|_| PyValueError::new_err("invalid Contract expectation"))?;
    if rows.len() > 10_000 {
        return Err(PyValueError::new_err("too many Contract rows"));
    }
    if [&kind, &field_name, &field_value]
        .into_iter()
        .any(|s| s.contains('\0'))
        || rows.iter().any(|(_, _, kind, field, value)| {
            [kind, field, value].into_iter().any(|s| s.contains('\0'))
        })
    {
        return Err(PyValueError::new_err(
            "Contract strings cannot contain NUL bytes",
        ));
    }
    let input = ContractInput {
        rows: rows
            .into_iter()
            .map(
                |(id, parent_id, kind, field_name, field_value)| ContractRow {
                    id,
                    parent_id,
                    kind,
                    field_name,
                    field_value,
                },
            )
            .collect(),
        scope_id,
        kind,
        field_name,
        field_value,
        expectation: mode,
        expected_count,
    };
    py.detach(|| orgize::c_ffi::evaluate_contract(input))
        .map(|result| (result.matched_count, result.passed))
        .map_err(pyo3::exceptions::PyRuntimeError::new_err)
}

/// Private installed-artifact qualification; does not start the native runtime.
#[pyfunction]
fn _native_runtime_identity() -> (&'static str, &'static str) {
    (
        orgize::runtime_backend().name(),
        orgize::org_aot::org_event_parser_digest(),
    )
}

/// First call requires exclusive host startup, before workers/subprocesses.
#[pyfunction]
fn initialize_native_runtime() -> PyResult<()> {
    // SAFETY: explicit Python startup API transfers this documented host
    // admission obligation to its caller; import alone never initializes.
    unsafe { orgize::initialize_native_runtime() }
        .map_err(pyo3::exceptions::PyRuntimeError::new_err)
}

#[pyclass(module = "orgizepy._orgize")]
struct ParsedOrg {
    document: OrgAotDocument,
}

#[pyclass(module = "orgizepy._orgize", get_all)]
#[derive(Clone)]
struct Element {
    id: usize,
    parent_id: Option<usize>,
    child_ids: Vec<usize>,
    kind: String,
    category: String,
    start: u32,
    end: u32,
    fields: Vec<(String, String)>,
}

#[pymethods]
impl ParsedOrg {
    #[getter]
    fn elements(&self) -> Vec<Element> {
        self.document
            .records()
            .iter()
            .map(|record| Element {
                id: record.id,
                parent_id: record.parent_id,
                child_ids: record.child_ids.clone(),
                kind: record.kind.to_owned(),
                category: record.category.to_owned(),
                start: u32::from(record.range.start()),
                end: u32::from(record.range.end()),
                fields: record
                    .fields
                    .iter()
                    .map(|field| (field.name.to_owned(), field.value.clone()))
                    .collect(),
            })
            .collect()
    }

    fn headline_todo_type(&self, record_id: usize) -> Option<String> {
        self.document
            .headline_todo_type(record_id)
            .map(str::to_owned)
    }

    fn headline_todo_keyword(&self, record_id: usize) -> Option<String> {
        self.document.headline_todo_keyword(record_id)
    }

    fn headline_content_after_todo(&self, record_id: usize) -> Option<String> {
        self.document.headline_content_after_todo(record_id)
    }
}

#[pyfunction]
fn parse_org(py: Python<'_>, source: &str) -> PyResult<ParsedOrg> {
    // Only immutable Rust text and owned native-index/Element data cross this boundary.
    // Scheme handles/roots stay on the selected native owner, not Python workers.
    py.detach(|| parse_org_aot(source))
        .map(|document| ParsedOrg { document })
        .map_err(|error| PyValueError::new_err(format!("Org parse failed: {error:?}")))
}

#[pyfunction]
fn source_digest(py: Python<'_>, source: &str) -> String {
    py.detach(|| org_source_digest(source))
}

#[pyfunction]
fn apply_source_edits(
    py: Python<'_>,
    source: &str,
    projected_source_digest: &str,
    edits: Vec<(String, usize, usize, String, String)>,
) -> PyResult<String> {
    let edits = edits
        .iter()
        .map(
            |(node_id, start_byte, end_byte, expected_old, replacement)| OrgSourceEdit {
                node_id,
                start_byte: *start_byte,
                end_byte: *end_byte,
                expected_old,
                replacement,
            },
        )
        .collect::<Vec<_>>();
    py.detach(|| apply_org_source_edits(source, projected_source_digest, &edits))
        .map_err(|error| PyValueError::new_err(format!("Org source edit failed: {error:?}")))
}

#[pymodule]
fn _orgize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(_evaluate_contract, module)?)?;
    module.add_function(wrap_pyfunction!(_native_runtime_identity, module)?)?;
    module.add_function(wrap_pyfunction!(initialize_native_runtime, module)?)?;
    module.add_class::<ParsedOrg>()?;
    module.add_class::<Element>()?;
    module.add_function(wrap_pyfunction!(parse_org, module)?)?;
    module.add_function(wrap_pyfunction!(source_digest, module)?)?;
    module.add_function(wrap_pyfunction!(apply_source_edits, module)?)?;
    Ok(())
}
