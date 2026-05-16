//! Source-block side-table projection for Babel/tangle-aware consumers.

use super::{
    block_metadata::parse_block_header_args, BlockHeaderArg, BlockKind, Document, Element,
    ElementData, Keyword, ListItem, Object, ObjectData, ParsedAnnotation, SourceBlockHeaderArg,
    SourceBlockHeaderArgKind, SourceBlockHeaderArgSource, SourceBlockHeaderVar, SourceBlockRecord,
    SourceBlockRecordKind, SourceBlockResult, SourceBlockResultKind, SourceBlockSource,
    SourceBlockTangle, SourceBlockTangleMode,
};

impl Document<ParsedAnnotation> {
    /// Projects source blocks into a document-local side table.
    ///
    /// This does not execute Babel blocks or resolve tangle destinations. It
    /// preserves ordinary Org metadata so downstream indexers can decide how to
    /// store, lint, or display source-block evidence.
    pub fn source_block_records(&self) -> Vec<SourceBlockRecord> {
        let mut records = Vec::new();
        collect_source_block_records_in_elements(&self.children, &mut records);
        for section in &self.sections {
            collect_source_block_records_in_elements(&section.children, &mut records);
            collect_source_block_records_in_sections(&section.subsections, &mut records);
        }
        records
    }
}

fn collect_source_block_records_in_sections(
    sections: &[super::Section<ParsedAnnotation>],
    records: &mut Vec<SourceBlockRecord>,
) {
    for section in sections {
        collect_source_block_records_in_elements(&section.children, records);
        collect_source_block_records_in_sections(&section.subsections, records);
    }
}

fn collect_source_block_records_in_elements(
    elements: &[Element<ParsedAnnotation>],
    records: &mut Vec<SourceBlockRecord>,
) {
    for (index, element) in elements.iter().enumerate() {
        if let ElementData::Block(block) = &element.data {
            if block.kind == BlockKind::Source {
                records.push(SourceBlockRecord {
                    source: SourceBlockSource::from_annotation(&element.ann),
                    kind: SourceBlockRecordKind::Block,
                    name: affiliated_keyword_value(&element.affiliated_keywords, "NAME"),
                    language: block.language.clone(),
                    parameters: block.parameters.clone(),
                    header_args: block.header_args.clone(),
                    normalized_header_args: source_block_header_args(
                        SourceBlockRecordKind::Block,
                        &block.header_args,
                    ),
                    code_refs: block.code_refs.clone(),
                    tangle: source_block_tangle(&block.header_args),
                    result: elements
                        .get(index + 1)
                        .and_then(source_block_result_from_element),
                    value: block.value.clone(),
                });
            }
            collect_source_block_records_in_elements(&block.children, records);
        }
        collect_inline_source_records(element, records);
        collect_nested_source_block_records(element, records);
    }
}

fn collect_nested_source_block_records(
    element: &Element<ParsedAnnotation>,
    records: &mut Vec<SourceBlockRecord>,
) {
    match &element.data {
        ElementData::Drawer(drawer) => {
            collect_source_block_records_in_elements(&drawer.children, records)
        }
        ElementData::List(list) => collect_source_block_records_in_list_items(&list.items, records),
        ElementData::FootnoteDef(footnote) => {
            collect_source_block_records_in_elements(&footnote.children, records);
        }
        ElementData::Inlinetask(task) => {
            collect_source_block_records_in_elements(&task.children, records);
        }
        ElementData::Paragraph(_)
        | ElementData::Table(_)
        | ElementData::Keyword(_)
        | ElementData::BabelCall(_)
        | ElementData::Clock(_)
        | ElementData::PropertyDrawer(_)
        | ElementData::TableEl { .. }
        | ElementData::Block(_)
        | ElementData::Comment(_)
        | ElementData::FixedWidth(_)
        | ElementData::Rule
        | ElementData::LatexEnvironment(_)
        | ElementData::Unknown { .. } => {}
    }
}

fn collect_source_block_records_in_list_items(
    items: &[ListItem<ParsedAnnotation>],
    records: &mut Vec<SourceBlockRecord>,
) {
    for item in items {
        collect_source_block_records_in_elements(&item.children, records);
    }
}

fn collect_inline_source_records(
    element: &Element<ParsedAnnotation>,
    records: &mut Vec<SourceBlockRecord>,
) {
    match &element.data {
        ElementData::Paragraph(objects) => {
            collect_inline_source_records_in_objects(objects, records)
        }
        ElementData::Table(table) => {
            for row in &table.rows {
                for cell in &row.cells {
                    collect_inline_source_records_in_objects(&cell.objects, records);
                }
            }
        }
        ElementData::List(list) => {
            for item in &list.items {
                collect_inline_source_records_in_objects(&item.tag, records);
            }
        }
        ElementData::Inlinetask(task) => {
            collect_inline_source_records_in_objects(&task.title, records);
        }
        _ => {}
    }
}

fn collect_inline_source_records_in_objects(
    objects: &[Object<ParsedAnnotation>],
    records: &mut Vec<SourceBlockRecord>,
) {
    for (index, object) in objects.iter().enumerate() {
        match &object.data {
            ObjectData::InlineSrc {
                language,
                parameters,
                value,
                ..
            } => {
                let header_args = parse_block_header_args(parameters.as_deref());
                records.push(SourceBlockRecord {
                    source: SourceBlockSource::from_annotation(&object.ann),
                    kind: SourceBlockRecordKind::InlineSource,
                    name: None,
                    language: Some(language.clone()),
                    parameters: parameters.clone(),
                    normalized_header_args: source_block_header_args(
                        SourceBlockRecordKind::InlineSource,
                        &header_args,
                    ),
                    header_args: header_args.clone(),
                    code_refs: Vec::new(),
                    tangle: source_block_tangle(&header_args),
                    result: objects.get(index + 1).and_then(inline_result_from_object),
                    value: value.clone(),
                });
            }
            ObjectData::Markup { children, .. } => {
                collect_inline_source_records_in_objects(children, records);
            }
            ObjectData::FootnoteRef { definition, .. } => {
                collect_inline_source_records_in_objects(definition, records);
            }
            ObjectData::Citation(citation) => {
                collect_inline_source_records_in_objects(&citation.prefix, records);
                collect_inline_source_records_in_objects(&citation.suffix, records);
                for reference in &citation.references {
                    collect_inline_source_records_in_objects(&reference.prefix, records);
                    collect_inline_source_records_in_objects(&reference.suffix, records);
                }
            }
            ObjectData::Link(link) => {
                collect_inline_source_records_in_objects(&link.description, records);
            }
            ObjectData::Cloze { text, .. } => {
                collect_inline_source_records_in_objects(text, records);
            }
            ObjectData::Plain(_)
            | ObjectData::LineBreak
            | ObjectData::Code(_)
            | ObjectData::Verbatim(_)
            | ObjectData::Timestamp(_)
            | ObjectData::Entity(_)
            | ObjectData::LatexFragment(_)
            | ObjectData::ExportSnippet { .. }
            | ObjectData::InlineCall { .. }
            | ObjectData::Target(_)
            | ObjectData::RadioTarget(_)
            | ObjectData::Macro { .. }
            | ObjectData::StatisticCookie(_)
            | ObjectData::Unknown { .. } => {}
        }
    }
}

fn affiliated_keyword_value(keywords: &[Keyword<ParsedAnnotation>], key: &str) -> Option<String> {
    keywords
        .iter()
        .find(|keyword| keyword.key.eq_ignore_ascii_case(key))
        .map(|keyword| keyword.value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn source_block_result_from_element(
    element: &Element<ParsedAnnotation>,
) -> Option<SourceBlockResult> {
    let keyword = element
        .affiliated_keywords
        .iter()
        .find(|keyword| keyword.key.eq_ignore_ascii_case("RESULTS"))?;
    Some(SourceBlockResult {
        source: SourceBlockSource::from_annotation(&element.ann),
        kind: SourceBlockResultKind::Keyword,
        hash: keyword
            .optional
            .clone()
            .filter(|value| !value.trim().is_empty()),
        name: (!keyword.value.trim().is_empty()).then(|| keyword.value.trim().to_string()),
        keyword_value: keyword.value.trim().to_string(),
        value: result_value(element),
    })
}

fn inline_result_from_object(object: &Object<ParsedAnnotation>) -> Option<SourceBlockResult> {
    let ObjectData::Macro { name, arguments } = &object.data else {
        return None;
    };
    if !name.eq_ignore_ascii_case("results") {
        return None;
    }
    let keyword_value = arguments.join(",");
    Some(SourceBlockResult {
        source: SourceBlockSource::from_annotation(&object.ann),
        kind: SourceBlockResultKind::InlineMacro,
        hash: None,
        name: None,
        value: keyword_value.clone(),
        keyword_value,
    })
}

fn result_value(element: &Element<ParsedAnnotation>) -> String {
    match &element.data {
        ElementData::FixedWidth(fixed) => fixed.value.trim_end().to_string(),
        ElementData::Block(block) => block.value.trim_end().to_string(),
        _ => strip_affiliated_result_prefix(element).trim().to_string(),
    }
}

fn strip_affiliated_result_prefix(element: &Element<ParsedAnnotation>) -> String {
    let mut value = element.ann.raw.as_str();
    for keyword in &element.affiliated_keywords {
        if keyword.key.eq_ignore_ascii_case("RESULTS") && value.starts_with(&keyword.ann.raw) {
            value = &value[keyword.ann.raw.len()..];
        }
    }
    value.to_string()
}

fn source_block_tangle(header_args: &[BlockHeaderArg]) -> Option<SourceBlockTangle> {
    let arg = header_args
        .iter()
        .find(|arg| arg.key.eq_ignore_ascii_case("tangle"))?;
    let raw_value = arg.value.clone().unwrap_or_else(|| "yes".to_string());
    let normalized = unquote_header_value(raw_value.trim());
    let mode = if normalized.eq_ignore_ascii_case("no") {
        SourceBlockTangleMode::No
    } else if normalized.eq_ignore_ascii_case("yes") {
        SourceBlockTangleMode::Yes
    } else {
        SourceBlockTangleMode::File
    };
    let target = (mode == SourceBlockTangleMode::File).then_some(normalized);
    Some(SourceBlockTangle {
        raw: arg.raw.clone(),
        mode,
        target,
    })
}

fn source_block_header_args(
    kind: SourceBlockRecordKind,
    header_args: &[BlockHeaderArg],
) -> Vec<SourceBlockHeaderArg> {
    let mut normalized = default_source_block_header_args(kind);
    for arg in header_args {
        let projected = source_block_header_arg(arg, SourceBlockHeaderArgSource::Explicit);
        if matches!(
            projected.kind,
            SourceBlockHeaderArgKind::Var | SourceBlockHeaderArgKind::Results
        ) {
            normalized.push(projected);
        } else if let Some(existing) = normalized
            .iter_mut()
            .find(|existing| existing.key.eq_ignore_ascii_case(&projected.key))
        {
            *existing = projected;
        } else {
            normalized.push(projected);
        }
    }
    normalized
}

fn default_source_block_header_args(kind: SourceBlockRecordKind) -> Vec<SourceBlockHeaderArg> {
    let defaults = match kind {
        SourceBlockRecordKind::Block => [
            ("session", "none"),
            ("results", "replace"),
            ("exports", "code"),
            ("cache", "no"),
            ("noweb", "no"),
            ("hlines", "no"),
            ("tangle", "no"),
        ],
        SourceBlockRecordKind::InlineSource => [
            ("session", "none"),
            ("results", "replace"),
            ("exports", "results"),
            ("cache", "no"),
            ("noweb", "no"),
            ("hlines", "yes"),
            ("tangle", "no"),
        ],
    };
    defaults
        .into_iter()
        .map(|(key, value)| {
            let raw = format!(":{key} {value}");
            let arg = BlockHeaderArg {
                key: key.to_string(),
                value: Some(value.to_string()),
                raw,
            };
            source_block_header_arg(&arg, SourceBlockHeaderArgSource::Default)
        })
        .collect()
}

fn source_block_header_arg(
    arg: &BlockHeaderArg,
    source: SourceBlockHeaderArgSource,
) -> SourceBlockHeaderArg {
    let kind = source_block_header_arg_kind(&arg.key);
    let tokens = arg
        .value
        .as_deref()
        .map(split_header_value)
        .unwrap_or_default();
    SourceBlockHeaderArg {
        key: arg.key.clone(),
        value: arg.value.clone(),
        raw: arg.raw.clone(),
        kind,
        source,
        variable: (kind == SourceBlockHeaderArgKind::Var)
            .then_some(arg.value.as_deref())
            .flatten()
            .map(source_block_header_var),
        tokens,
    }
}

fn source_block_header_arg_kind(key: &str) -> SourceBlockHeaderArgKind {
    match key.to_ascii_lowercase().as_str() {
        "cache" => SourceBlockHeaderArgKind::Cache,
        "dir" => SourceBlockHeaderArgKind::Dir,
        "eval" => SourceBlockHeaderArgKind::Eval,
        "exports" => SourceBlockHeaderArgKind::Exports,
        "hlines" => SourceBlockHeaderArgKind::Hlines,
        "noweb" => SourceBlockHeaderArgKind::Noweb,
        "results" => SourceBlockHeaderArgKind::Results,
        "session" => SourceBlockHeaderArgKind::Session,
        "tangle" => SourceBlockHeaderArgKind::Tangle,
        "var" => SourceBlockHeaderArgKind::Var,
        _ => SourceBlockHeaderArgKind::Other,
    }
}

fn source_block_header_var(value: &str) -> SourceBlockHeaderVar {
    let trimmed = value.trim();
    if let Some((name, assignment)) = trimmed.split_once('=') {
        SourceBlockHeaderVar {
            name: name.trim().to_string(),
            assignment: Some(assignment.trim().to_string()),
        }
    } else {
        SourceBlockHeaderVar {
            name: trimmed.to_string(),
            assignment: None,
        }
    }
}

fn split_header_value(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;

    for ch in value.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if quote == Some(ch) {
            quote = None;
        } else if quote.is_none() && matches!(ch, '"' | '\'') {
            quote = Some(ch);
        } else if quote.is_none() && ch.is_whitespace() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }

    if escaped {
        current.push('\\');
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn unquote_header_value(value: &str) -> String {
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}
