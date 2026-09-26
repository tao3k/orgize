//! Thin Python projection of the Scheme-AOT Org parser, functions, and source edits.

use orgize::org_aot::{OrgAotDocument, parse_org_aot};
use orgize::org_aot_edit::{OrgSourceEdit, apply_org_source_edits, org_source_digest};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

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
fn parse_org(source: &str) -> PyResult<ParsedOrg> {
    parse_org_aot(source)
        .map(|document| ParsedOrg { document })
        .map_err(|error| PyValueError::new_err(format!("Org parse failed: {error:?}")))
}

#[pyfunction]
fn source_digest(source: &str) -> String {
    org_source_digest(source)
}

#[pyfunction]
fn apply_source_edits(
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
    apply_org_source_edits(source, projected_source_digest, &edits)
        .map_err(|error| PyValueError::new_err(format!("Org source edit failed: {error:?}")))
}

#[pymodule]
fn _orgize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<ParsedOrg>()?;
    module.add_class::<Element>()?;
    module.add_function(wrap_pyfunction!(parse_org, module)?)?;
    module.add_function(wrap_pyfunction!(source_digest, module)?)?;
    module.add_function(wrap_pyfunction!(apply_source_edits, module)?)?;
    Ok(())
}
