//! Stable JSON projection for explicit Org element host bindings.

use serde_json::{Value, json};

use super::{
    Document, FootnoteDefinition, FootnoteEntry, Keyword, OrgDuration, ParsedAnnotation, Planning,
    Priority, Property, Section, SourceBlockBooleanHeader, SourceBlockCache, SourceBlockDirectory,
    SourceBlockDirectoryKind, SourceBlockEval, SourceBlockEvalPolicy, SourceBlockExecutionPlan,
    SourceBlockExports, SourceBlockExportsPolicy, SourceBlockHeaderArg, SourceBlockHeaderArgKind,
    SourceBlockHeaderArgSource, SourceBlockHeaderVar, SourceBlockNowebAction, SourceBlockNowebPlan,
    SourceBlockRecord, SourceBlockRecordKind, SourceBlockResult, SourceBlockResultCollection,
    SourceBlockResultFile, SourceBlockResultFormat, SourceBlockResultHandling,
    SourceBlockResultKind, SourceBlockResultOptions, SourceBlockResultValueType,
    SourceBlockSession, SourceBlockSource, SourceBlockTangle, SourceBlockTangleCommentsMode,
    SourceBlockTangleMode, SourceBlockTangleNowebMode, SourcePosition, TargetDefinition,
    TargetKind, TodoKeyword, TodoState,
};

pub(super) fn document_json(document: &Document<ParsedAnnotation>) -> String {
    serde_json::to_string(&document_value(document)).expect("Org elements payload should serialize")
}

fn document_value(document: &Document<ParsedAnnotation>) -> Value {
    json!({
        "schemaVersion": 1,
        "source": annotation_json(&document.ann),
        "metadata": document.metadata.iter().map(keyword_json).collect::<Vec<_>>(),
        "filetags": &document.filetags,
        "tagDefinitions": document
            .tag_definitions
            .iter()
            .map(|definition| json!({
                "name": &definition.name,
                "shortcut": &definition.shortcut,
                "raw": &definition.raw,
                "isGroup": definition.is_group,
                "group": definition.group.as_ref().map(|group| json!({
                    "name": &group.name,
                    "exclusive": group.exclusive,
                })),
            }))
            .collect::<Vec<_>>(),
        "targets": document.targets.iter().map(target_definition_json).collect::<Vec<_>>(),
        "footnotes": document.footnotes.iter().map(footnote_entry_json).collect::<Vec<_>>(),
        "elements": super::elements_bridge_element_json::elements_json(&document.children),
        "sections": sections_json(&document.sections, Vec::new()),
        "index": super::elements_bridge_index_json::index_json(document),
        "sourceBlocks": document
            .source_block_records()
            .iter()
            .map(source_block_json)
            .collect::<Vec<_>>(),
    })
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
        "priority": priority_json(&section.priority),
        "planning": planning_json(&section.planning),
        "titleObjects": super::elements_bridge_object_json::objects_json(&section.title),
        "properties": properties_json(&section.properties),
        "effectiveProperties": properties_json(&section.effective_properties),
        "elements": super::elements_bridge_element_json::elements_json(&section.children),
        "children": children,
    })
}

pub(super) fn keyword_json(keyword: &Keyword<ParsedAnnotation>) -> Value {
    json!({
        "source": annotation_json(&keyword.ann),
        "key": &keyword.key,
        "optional": &keyword.optional,
        "value": &keyword.value,
        "parsedObjectCount": keyword.parsed.len(),
        "parsed": super::elements_bridge_object_json::objects_json(&keyword.parsed),
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

pub(super) fn properties_json(properties: &[Property<ParsedAnnotation>]) -> Vec<Value> {
    properties.iter().map(property_json).collect()
}

fn property_json(property: &Property<ParsedAnnotation>) -> Value {
    json!({
        "source": annotation_json(&property.ann),
        "key": &property.key,
        "value": &property.value,
        "duration": property.duration.as_ref().map(duration_json),
    })
}

fn target_definition_json(target: &TargetDefinition<ParsedAnnotation>) -> Value {
    json!({
        "source": annotation_json(&target.ann),
        "kind": target_kind(target.kind),
        "key": &target.key,
        "value": &target.value,
        "raw": &target.raw,
        "alias": super::elements_bridge_object_json::objects_json(&target.alias),
    })
}

fn footnote_entry_json(footnote: &FootnoteEntry<ParsedAnnotation>) -> Value {
    let (kind, body) = match &footnote.definition {
        FootnoteDefinition::Standalone(elements) => (
            "standalone",
            json!(super::elements_bridge_element_json::elements_json(elements)),
        ),
        FootnoteDefinition::Inline(objects) => (
            "inline",
            json!(super::elements_bridge_object_json::objects_json(objects)),
        ),
    };
    json!({
        "source": annotation_json(&footnote.ann),
        "label": &footnote.label,
        "definitionKind": kind,
        "body": body,
    })
}

pub(super) fn planning_json(planning: &Planning) -> Value {
    json!({
        "deadline": planning.deadline.as_ref().map(super::elements_bridge_object_json::timestamp_json),
        "scheduled": planning.scheduled.as_ref().map(super::elements_bridge_object_json::timestamp_json),
        "closed": planning.closed.as_ref().map(super::elements_bridge_object_json::timestamp_json),
    })
}

pub(super) fn priority_json(priority: &Priority) -> Value {
    json!({
        "raw": priority.raw_cookie(),
        "effective": priority.effective_text(),
        "isDefault": priority.is_default(),
        "orgScore": priority.org_priority_score(),
    })
}

pub(super) fn duration_json(duration: &OrgDuration) -> Value {
    json!({
        "raw": &duration.raw,
        "totalSeconds": duration.total_seconds,
        "totalMinutes": duration.total_minutes(),
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
        "resultOptions": source_block_result_options_json(&record.result_options),
        "execution": source_block_execution_json(&record.execution),
        "result": record.result.as_ref().map(source_block_result_json),
        "value": &record.value,
    })
}

pub(super) fn block_header_arg_json(arg: &super::BlockHeaderArg) -> Value {
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
        "mkdirp": {
            "raw": &tangle.mkdirp.raw,
            "enabled": tangle.mkdirp.enabled,
        },
        "comments": {
            "raw": &tangle.comments.raw,
            "mode": source_block_tangle_comments_mode(tangle.comments.mode),
        },
        "shebang": &tangle.shebang,
        "noweb": {
            "raw": &tangle.noweb.raw,
            "mode": source_block_tangle_noweb_mode(tangle.noweb.mode),
        },
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

fn source_block_result_options_json(options: &SourceBlockResultOptions) -> Value {
    json!({
        "raw": &options.raw,
        "source": source_block_header_arg_source(options.source),
        "tokens": &options.tokens,
        "collection": options.collection.map(source_block_result_collection),
        "format": options.format.map(source_block_result_format),
        "handling": source_block_result_handling(options.handling),
        "valueType": source_block_result_value_type(options.value_type),
        "unknown": &options.unknown,
        "file": options.file.as_ref().map(source_block_result_file_json),
    })
}

fn source_block_result_file_json(file: &SourceBlockResultFile) -> Value {
    json!({
        "target": &file.target,
        "description": &file.description,
        "extension": &file.extension,
        "fileMode": file.file_mode.as_ref().map(|mode| mode.raw.as_str()),
        "outputDir": &file.output_dir,
    })
}

fn source_block_execution_json(execution: &SourceBlockExecutionPlan) -> Value {
    json!({
        "eval": source_block_eval_json(&execution.eval),
        "exports": source_block_exports_json(&execution.exports),
        "cache": source_block_cache_json(&execution.cache),
        "session": source_block_session_json(&execution.session),
        "directory": execution.directory.as_ref().map(source_block_directory_json),
        "hlines": source_block_boolean_header_json(&execution.hlines),
        "noweb": source_block_noweb_json(&execution.noweb),
    })
}

fn source_block_eval_json(eval: &SourceBlockEval) -> Value {
    json!({
        "raw": &eval.raw,
        "source": source_block_header_arg_source(eval.source),
        "policy": source_block_eval_policy(eval.policy),
    })
}

fn source_block_exports_json(exports: &SourceBlockExports) -> Value {
    json!({
        "raw": &exports.raw,
        "source": source_block_header_arg_source(exports.source),
        "policy": source_block_exports_policy(exports.policy),
    })
}

fn source_block_cache_json(cache: &SourceBlockCache) -> Value {
    json!({
        "raw": &cache.raw,
        "source": source_block_header_arg_source(cache.source),
        "enabled": cache.enabled,
    })
}

fn source_block_session_json(session: &SourceBlockSession) -> Value {
    json!({
        "raw": &session.raw,
        "source": source_block_header_arg_source(session.source),
        "name": &session.name,
        "active": session.active,
    })
}

fn source_block_directory_json(directory: &SourceBlockDirectory) -> Value {
    json!({
        "raw": &directory.raw,
        "source": source_block_header_arg_source(directory.source),
        "target": &directory.target,
        "kind": source_block_directory_kind(directory.kind),
    })
}

fn source_block_boolean_header_json(header: &SourceBlockBooleanHeader) -> Value {
    json!({
        "raw": &header.raw,
        "source": source_block_header_arg_source(header.source),
        "enabled": header.enabled,
    })
}

fn source_block_noweb_json(noweb: &SourceBlockNowebPlan) -> Value {
    json!({
        "raw": &noweb.raw,
        "source": source_block_header_arg_source(noweb.source),
        "tokens": &noweb.tokens,
        "eval": source_block_noweb_action(noweb.eval),
        "export": source_block_noweb_action(noweb.export),
        "tangle": source_block_noweb_action(noweb.tangle),
    })
}

pub(super) fn annotation_json(annotation: &ParsedAnnotation) -> Value {
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

pub(super) fn todo_state(todo: &TodoKeyword) -> &'static str {
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
        SourceBlockHeaderArgKind::File => "file",
        SourceBlockHeaderArgKind::FileDesc => "fileDesc",
        SourceBlockHeaderArgKind::FileExt => "fileExt",
        SourceBlockHeaderArgKind::FileMode => "fileMode",
        SourceBlockHeaderArgKind::Hlines => "hlines",
        SourceBlockHeaderArgKind::Noweb => "noweb",
        SourceBlockHeaderArgKind::OutputDir => "outputDir",
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

fn source_block_tangle_comments_mode(mode: SourceBlockTangleCommentsMode) -> &'static str {
    match mode {
        SourceBlockTangleCommentsMode::No => "no",
        SourceBlockTangleCommentsMode::Link => "link",
        SourceBlockTangleCommentsMode::Yes => "yes",
        SourceBlockTangleCommentsMode::Org => "org",
        SourceBlockTangleCommentsMode::Both => "both",
        SourceBlockTangleCommentsMode::Noweb => "noweb",
        SourceBlockTangleCommentsMode::Other => "other",
    }
}

fn source_block_tangle_noweb_mode(mode: SourceBlockTangleNowebMode) -> &'static str {
    match mode {
        SourceBlockTangleNowebMode::Disabled => "disabled",
        SourceBlockTangleNowebMode::Expand => "expand",
        SourceBlockTangleNowebMode::Strip => "strip",
    }
}

fn source_block_result_kind(kind: SourceBlockResultKind) -> &'static str {
    match kind {
        SourceBlockResultKind::Keyword => "keyword",
        SourceBlockResultKind::InlineMacro => "inlineMacro",
    }
}

fn source_block_result_collection(collection: SourceBlockResultCollection) -> &'static str {
    match collection {
        SourceBlockResultCollection::File => "file",
        SourceBlockResultCollection::List => "list",
        SourceBlockResultCollection::Vector => "vector",
        SourceBlockResultCollection::Table => "table",
        SourceBlockResultCollection::Scalar => "scalar",
        SourceBlockResultCollection::Verbatim => "verbatim",
    }
}

fn source_block_result_format(format: SourceBlockResultFormat) -> &'static str {
    match format {
        SourceBlockResultFormat::Raw => "raw",
        SourceBlockResultFormat::Html => "html",
        SourceBlockResultFormat::Latex => "latex",
        SourceBlockResultFormat::Org => "org",
        SourceBlockResultFormat::Code => "code",
        SourceBlockResultFormat::Pp => "pp",
        SourceBlockResultFormat::Drawer => "drawer",
        SourceBlockResultFormat::Link => "link",
        SourceBlockResultFormat::Graphics => "graphics",
    }
}

fn source_block_result_handling(handling: SourceBlockResultHandling) -> &'static str {
    match handling {
        SourceBlockResultHandling::Replace => "replace",
        SourceBlockResultHandling::Silent => "silent",
        SourceBlockResultHandling::None => "none",
        SourceBlockResultHandling::Discard => "discard",
        SourceBlockResultHandling::Append => "append",
        SourceBlockResultHandling::Prepend => "prepend",
    }
}

fn source_block_result_value_type(value_type: SourceBlockResultValueType) -> &'static str {
    match value_type {
        SourceBlockResultValueType::Value => "value",
        SourceBlockResultValueType::Output => "output",
    }
}

fn source_block_eval_policy(policy: SourceBlockEvalPolicy) -> &'static str {
    match policy {
        SourceBlockEvalPolicy::Yes => "yes",
        SourceBlockEvalPolicy::No => "no",
        SourceBlockEvalPolicy::NoExport => "noExport",
        SourceBlockEvalPolicy::StripExport => "stripExport",
        SourceBlockEvalPolicy::NeverExport => "neverExport",
        SourceBlockEvalPolicy::Eval => "eval",
        SourceBlockEvalPolicy::Never => "never",
        SourceBlockEvalPolicy::Query => "query",
        SourceBlockEvalPolicy::Other => "other",
    }
}

fn source_block_exports_policy(policy: SourceBlockExportsPolicy) -> &'static str {
    match policy {
        SourceBlockExportsPolicy::Code => "code",
        SourceBlockExportsPolicy::Results => "results",
        SourceBlockExportsPolicy::Both => "both",
        SourceBlockExportsPolicy::None => "none",
        SourceBlockExportsPolicy::Other => "other",
    }
}

fn source_block_directory_kind(kind: SourceBlockDirectoryKind) -> &'static str {
    match kind {
        SourceBlockDirectoryKind::Path => "path",
        SourceBlockDirectoryKind::Attachment => "attachment",
    }
}

fn source_block_noweb_action(action: SourceBlockNowebAction) -> &'static str {
    match action {
        SourceBlockNowebAction::Disabled => "disabled",
        SourceBlockNowebAction::Expand => "expand",
        SourceBlockNowebAction::Strip => "strip",
    }
}

fn target_kind(kind: TargetKind) -> &'static str {
    match kind {
        TargetKind::Headline => "headline",
        TargetKind::CustomId => "customId",
        TargetKind::Id => "id",
        TargetKind::Target => "target",
        TargetKind::RadioTarget => "radioTarget",
        TargetKind::FootnoteDefinition => "footnoteDefinition",
        TargetKind::CodeRef => "codeRef",
    }
}
