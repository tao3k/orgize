//! Explicit host bindings over semantic Org elements.

use std::{
    io::Write,
    process::{Command, Stdio},
};

use super::{
    Document, Element, ElementData, Keyword, ListItem, OrgElementSelector,
    OrgElementsExecutionPlan, OrgElementsHostExecutionError, OrgElementsHostExecutionOptions,
    OrgElementsHostExecutionOutput, OrgElementsHostExecutionStatus, OrgElementsIndexQuery,
    OrgElementsIndexRecord, ParsedAnnotation, PythonDirective, PythonDirectiveKind,
    PythonExecutionOptions, Section,
};

impl<A: Clone> Document<A> {
    /// Projects explicit host directives without running them.
    ///
    /// `#+PYTHON:` is treated as inline code and `#+PYTHON_FILE:` as a host
    /// script path. Execution is always a separate opt-in host call.
    pub fn org_elements_execution_plan(&self) -> OrgElementsExecutionPlan<A> {
        let mut python_directives = Vec::new();
        collect_python_directives_in_elements(&self.children, &mut python_directives);
        for section in &self.sections {
            collect_python_directives_in_section(section, &mut python_directives);
        }
        OrgElementsExecutionPlan { python_directives }
    }
}

impl Document<ParsedAnnotation> {
    /// Builds a typed, flat `org-element-map`-style index for host consumers.
    pub fn org_elements_index(&self) -> Vec<OrgElementsIndexRecord<ParsedAnnotation>> {
        super::elements_bridge_index::index_records(self)
    }

    /// Builds a typed flat index filtered by an `OrgElementsIndexQuery`.
    pub fn query_org_elements_index(
        &self,
        query: &OrgElementsIndexQuery,
    ) -> Vec<OrgElementsIndexRecord<ParsedAnnotation>> {
        if query.limit == Some(0) {
            return Vec::new();
        }
        let mut records = Vec::new();
        for record in self.org_elements_index() {
            if query.matches(&record) {
                records.push(record);
                if query.limit.is_some_and(|limit| records.len() >= limit) {
                    break;
                }
            }
        }
        records
    }

    /// Selects element index records using an Org-mode-style element selector.
    pub fn select_org_elements(
        &self,
        selector: &OrgElementSelector,
    ) -> Vec<OrgElementsIndexRecord<ParsedAnnotation>> {
        self.query_org_elements_index(&selector.to_index_query())
    }

    /// Serializes only the flat Org elements index, without the full tree.
    pub fn org_elements_index_json(&self) -> String {
        serde_json::to_string(&super::elements_bridge_index_json::index_json_from_records(
            &self.org_elements_index(),
        ))
        .expect("Org elements index JSON serialization should not fail")
    }

    /// Serializes a filtered flat Org elements index.
    pub fn org_elements_index_query_json(&self, query: &OrgElementsIndexQuery) -> String {
        serde_json::to_string(&super::elements_bridge_index_json::index_json_from_records(
            &self.query_org_elements_index(query),
        ))
        .expect("Org elements index query JSON serialization should not fail")
    }

    /// Serializes a stable, compact Org elements payload for host consumers.
    pub fn org_elements_json(&self) -> String {
        super::elements_bridge_json::document_json(self)
    }

    /// Runs a host tool with `org_elements_json()` on stdin.
    ///
    /// The parser never calls this automatically; callers must choose to run it.
    pub fn execute_org_elements(
        &self,
        options: &OrgElementsHostExecutionOptions,
    ) -> Result<OrgElementsHostExecutionOutput, OrgElementsHostExecutionError> {
        let mut command = Command::new(&options.command);
        command.args(&options.args);

        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(OrgElementsHostExecutionError::Spawn)?;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(self.org_elements_json().as_bytes())
                .map_err(OrgElementsHostExecutionError::Stdin)?;
        }
        let output = child
            .wait_with_output()
            .map_err(OrgElementsHostExecutionError::Wait)?;
        Ok(OrgElementsHostExecutionOutput {
            status: OrgElementsHostExecutionStatus {
                success: output.status.success(),
                code: output.status.code(),
            },
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }

    /// Runs Python with `org_elements_json()` on stdin.
    pub fn execute_python(
        &self,
        options: &PythonExecutionOptions,
    ) -> Result<OrgElementsHostExecutionOutput, OrgElementsHostExecutionError> {
        self.execute_org_elements(&options.to_host_options())
    }
}

fn collect_python_directives_in_section<A: Clone>(
    section: &Section<A>,
    directives: &mut Vec<PythonDirective<A>>,
) {
    collect_python_directives_in_elements(&section.children, directives);
    for subsection in &section.subsections {
        collect_python_directives_in_section(subsection, directives);
    }
}

fn collect_python_directives_in_elements<A: Clone>(
    elements: &[Element<A>],
    directives: &mut Vec<PythonDirective<A>>,
) {
    for element in elements {
        for keyword in &element.affiliated_keywords {
            collect_python_directive(keyword, directives);
        }
        match &element.data {
            ElementData::Keyword(keyword) | ElementData::BabelCall(keyword) => {
                collect_python_directive(keyword, directives);
            }
            ElementData::Drawer(drawer) => {
                collect_python_directives_in_elements(&drawer.children, directives);
            }
            ElementData::List(list) => {
                for item in &list.items {
                    collect_python_directives_in_list_item(item, directives);
                }
            }
            ElementData::Block(block) => {
                collect_python_directives_in_elements(&block.children, directives);
            }
            ElementData::FootnoteDef(footnote) => {
                collect_python_directives_in_elements(&footnote.children, directives);
            }
            ElementData::Inlinetask(task) => {
                collect_python_directives_in_elements(&task.children, directives);
            }
            ElementData::Paragraph(_)
            | ElementData::Clock(_)
            | ElementData::PropertyDrawer(_)
            | ElementData::Table(_)
            | ElementData::TableEl { .. }
            | ElementData::Comment(_)
            | ElementData::FixedWidth(_)
            | ElementData::Rule
            | ElementData::LatexEnvironment(_)
            | ElementData::Unknown { .. } => {}
        }
    }
}

fn collect_python_directives_in_list_item<A: Clone>(
    item: &ListItem<A>,
    directives: &mut Vec<PythonDirective<A>>,
) {
    collect_python_directives_in_elements(&item.children, directives);
}

fn collect_python_directive<A: Clone>(
    keyword: &Keyword<A>,
    directives: &mut Vec<PythonDirective<A>>,
) {
    let kind = match keyword.key.to_ascii_uppercase().as_str() {
        "PYTHON" => PythonDirectiveKind::Inline,
        "PYTHON_FILE" | "PYTHON-FILE" => PythonDirectiveKind::File,
        _ => return,
    };
    directives.push(PythonDirective {
        ann: keyword.ann.clone(),
        kind,
        value: keyword.value.trim().to_string(),
        raw: keyword.value.clone(),
    });
}
