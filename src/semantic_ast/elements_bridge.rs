//! Explicit host bindings over semantic Org elements.

use std::{
    io::Write,
    process::{Command, Stdio},
};

use serde_json::{json, Value};

use super::{
    Document, Element, ElementData, Keyword, ListItem, OrgElementsExecutionPlan,
    OrgElementsHostExecutionError, OrgElementsHostExecutionOptions, OrgElementsHostExecutionOutput,
    OrgElementsHostExecutionStatus, ParsedAnnotation, Property, PythonDirective,
    PythonDirectiveKind, PythonExecutionOptions, Section, SourceBlockHeaderArg,
    SourceBlockHeaderArgKind, SourceBlockHeaderArgSource, SourceBlockHeaderVar, SourceBlockRecord,
    SourceBlockRecordKind, SourceBlockResult, SourceBlockResultKind, SourceBlockSource,
    SourceBlockTangle, SourceBlockTangleMode, SourcePosition, TodoKeyword, TodoState,
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
    /// Serializes a stable, compact Org elements payload for host consumers.
    pub fn org_elements_json(&self) -> String {
        serde_json::to_string(&self.org_elements_value())
            .expect("Org elements payload should serialize")
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

    fn org_elements_value(&self) -> Value {
        json!({
            "schemaVersion": 1,
            "metadata": self.metadata.iter().map(keyword_json).collect::<Vec<_>>(),
            "filetags": &self.filetags,
            "tagDefinitions": self
                .tag_definitions
                .iter()
                .map(|definition| json!({
                    "name": &definition.name,
                    "shortcut": &definition.shortcut,
                    "raw": &definition.raw,
                }))
                .collect::<Vec<_>>(),
            "sections": sections_json(&self.sections, Vec::new()),
            "sourceBlocks": self
                .source_block_records()
                .iter()
                .map(source_block_json)
                .collect::<Vec<_>>(),
        })
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

fn sections_json(sections: &[Section<ParsedAnnotation>], outline_path: Vec<String>) -> Vec<Value> {
    sections
        .iter()
        .map(|section| section_json(section, outline_path.clone()))
        .collect()
}

fn section_json(section: &Section<ParsedAnnotation>, mut outline_path: Vec<String>) -> Value {
    outline_path.push(section.raw_title.trim_end().to_string());
    let children = sections_json(&section.subsections, outline_path.clone());
    json!({
        "source": annotation_json(&section.ann),
        "outlinePath": &outline_path,
        "level": section.level,
        "title": section.raw_title.trim_end(),
        "todo": section.todo.as_ref().map(|todo| todo.name.as_str()),
        "todoState": section.todo.as_ref().map(todo_state),
        "tags": &section.tags,
        "effectiveTags": &section.effective_tags,
        "anchor": &section.anchor,
        "isComment": section.is_comment,
        "properties": properties_json(&section.properties),
        "effectiveProperties": properties_json(&section.effective_properties),
        "children": children,
    })
}

fn keyword_json(keyword: &Keyword<ParsedAnnotation>) -> Value {
    json!({
        "source": annotation_json(&keyword.ann),
        "key": &keyword.key,
        "optional": &keyword.optional,
        "value": &keyword.value,
        "parsedObjectCount": keyword.parsed.len(),
        "attributes": keyword
            .attributes
            .iter()
            .map(|attribute| json!({
                "key": &attribute.key,
                "value": &attribute.value,
                "raw": &attribute.raw,
            }))
            .collect::<Vec<_>>(),
    })
}

fn properties_json(properties: &[Property<ParsedAnnotation>]) -> Vec<Value> {
    properties.iter().map(property_json).collect()
}

fn property_json(property: &Property<ParsedAnnotation>) -> Value {
    json!({
        "source": annotation_json(&property.ann),
        "key": &property.key,
        "value": &property.value,
    })
}

fn source_block_json(record: &SourceBlockRecord) -> Value {
    json!({
        "source": source_block_source_json(&record.source),
        "kind": source_block_record_kind(record.kind),
        "name": &record.name,
        "language": &record.language,
        "parameters": &record.parameters,
        "headerArgs": record.header_args.iter().map(block_header_arg_json).collect::<Vec<_>>(),
        "normalizedHeaderArgs": record
            .normalized_header_args
            .iter()
            .map(source_block_header_arg_json)
            .collect::<Vec<_>>(),
        "codeRefs": record
            .code_refs
            .iter()
            .map(|code_ref| json!({
                "line": code_ref.line,
                "column": code_ref.column,
                "endColumn": code_ref.end_column,
                "name": &code_ref.name,
                "raw": &code_ref.raw,
            }))
            .collect::<Vec<_>>(),
        "tangle": record.tangle.as_ref().map(source_block_tangle_json),
        "result": record.result.as_ref().map(source_block_result_json),
        "value": &record.value,
    })
}

fn block_header_arg_json(arg: &super::BlockHeaderArg) -> Value {
    json!({
        "key": &arg.key,
        "value": &arg.value,
        "raw": &arg.raw,
    })
}

fn source_block_header_arg_json(arg: &SourceBlockHeaderArg) -> Value {
    json!({
        "key": &arg.key,
        "value": &arg.value,
        "raw": &arg.raw,
        "kind": source_block_header_arg_kind(arg.kind),
        "source": source_block_header_arg_source(arg.source),
        "tokens": &arg.tokens,
        "variable": arg.variable.as_ref().map(source_block_header_var_json),
    })
}

fn source_block_header_var_json(var: &SourceBlockHeaderVar) -> Value {
    json!({
        "name": &var.name,
        "assignment": &var.assignment,
    })
}

fn source_block_tangle_json(tangle: &SourceBlockTangle) -> Value {
    json!({
        "raw": &tangle.raw,
        "mode": source_block_tangle_mode(tangle.mode),
        "target": &tangle.target,
    })
}

fn source_block_result_json(result: &SourceBlockResult) -> Value {
    json!({
        "source": source_block_source_json(&result.source),
        "kind": source_block_result_kind(result.kind),
        "hash": &result.hash,
        "name": &result.name,
        "keywordValue": &result.keyword_value,
        "value": &result.value,
    })
}

fn annotation_json(annotation: &ParsedAnnotation) -> Value {
    json!({
        "start": source_position_json(annotation.start),
        "end": source_position_json(annotation.end),
        "rangeStart": u32::from(annotation.range.start()),
        "rangeEnd": u32::from(annotation.range.end()),
        "raw": &annotation.raw,
    })
}

fn source_block_source_json(source: &SourceBlockSource) -> Value {
    json!({
        "start": source_position_json(source.start),
        "end": source_position_json(source.end),
        "rangeStart": source.range_start,
        "rangeEnd": source.range_end,
    })
}

fn source_position_json(position: SourcePosition) -> Value {
    json!({
        "line": position.line,
        "column": position.column,
    })
}

fn todo_state(todo: &TodoKeyword) -> &'static str {
    match todo.state {
        TodoState::Todo => "todo",
        TodoState::Done => "done",
    }
}

fn source_block_record_kind(kind: SourceBlockRecordKind) -> &'static str {
    match kind {
        SourceBlockRecordKind::Block => "block",
        SourceBlockRecordKind::InlineSource => "inlineSource",
    }
}

fn source_block_header_arg_kind(kind: SourceBlockHeaderArgKind) -> &'static str {
    match kind {
        SourceBlockHeaderArgKind::Cache => "cache",
        SourceBlockHeaderArgKind::Dir => "dir",
        SourceBlockHeaderArgKind::Eval => "eval",
        SourceBlockHeaderArgKind::Exports => "exports",
        SourceBlockHeaderArgKind::Hlines => "hlines",
        SourceBlockHeaderArgKind::Noweb => "noweb",
        SourceBlockHeaderArgKind::Results => "results",
        SourceBlockHeaderArgKind::Session => "session",
        SourceBlockHeaderArgKind::Tangle => "tangle",
        SourceBlockHeaderArgKind::Var => "var",
        SourceBlockHeaderArgKind::Other => "other",
    }
}

fn source_block_header_arg_source(source: SourceBlockHeaderArgSource) -> &'static str {
    match source {
        SourceBlockHeaderArgSource::Explicit => "explicit",
        SourceBlockHeaderArgSource::Default => "default",
    }
}

fn source_block_tangle_mode(mode: SourceBlockTangleMode) -> &'static str {
    match mode {
        SourceBlockTangleMode::Yes => "yes",
        SourceBlockTangleMode::No => "no",
        SourceBlockTangleMode::File => "file",
    }
}

fn source_block_result_kind(kind: SourceBlockResultKind) -> &'static str {
    match kind {
        SourceBlockResultKind::Keyword => "keyword",
        SourceBlockResultKind::InlineMacro => "inlineMacro",
    }
}
