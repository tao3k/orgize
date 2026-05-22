//! Source-block reference side-table projection for literate-programming tools.

use std::collections::BTreeSet;

use super::{
    AstRef, Document, ElementData, ObjectData, ParsedAnnotation, SourceBlockHeaderArgKind,
    SourceBlockHeaderArgSource, SourceBlockRecord, SourceBlockReference, SourceBlockReferenceKind,
    SourceBlockSource,
};

impl Document<ParsedAnnotation> {
    /// Projects source-block name references without executing Babel.
    ///
    /// A reference resolves when it points at a local `#+NAME` source block or
    /// a syntax-appropriate local `:noweb-ref` header argument. Babel calls and
    /// header-variable source dependencies resolve only against named source
    /// blocks. The projection is intentionally file-local; workspace-level
    /// resolution belongs in host tooling.
    pub fn source_block_references(&self) -> Vec<SourceBlockReference> {
        let records = self.source_block_records();
        let names = source_block_names(&records);
        let mut references = noweb_reference_edges(&records, &names.noweb_names);
        references.extend(header_var_reference_edges(&records, &names.source_names));
        references.extend(call_reference_edges(self, &names.source_names));
        references
    }
}

struct SourceBlockNameIndex {
    source_names: BTreeSet<String>,
    noweb_names: BTreeSet<String>,
}

fn noweb_reference_edges(
    records: &[SourceBlockRecord],
    names: &BTreeSet<String>,
) -> Vec<SourceBlockReference> {
    let mut references = Vec::new();
    for record in records {
        for target in noweb_references(&record.value) {
            references.push(source_block_reference(
                names,
                record.source.clone(),
                SourceBlockReferenceKind::Noweb,
                None,
                target,
            ));
        }
    }
    references
}

fn header_var_reference_edges(
    records: &[SourceBlockRecord],
    names: &BTreeSet<String>,
) -> Vec<SourceBlockReference> {
    let mut references = Vec::new();
    for record in records {
        for arg in &record.normalized_header_args {
            if arg.source != SourceBlockHeaderArgSource::Explicit
                || arg.kind != SourceBlockHeaderArgKind::Var
            {
                continue;
            }
            let Some(variable) = &arg.variable else {
                continue;
            };
            let Some(assignment) = variable.assignment.as_deref() else {
                continue;
            };
            let Some(reference) = header_var_reference_target(assignment, names) else {
                continue;
            };
            references.push(source_block_reference(
                names,
                record.source.clone(),
                SourceBlockReferenceKind::HeaderVar,
                Some(variable.name.clone()),
                reference,
            ));
        }
    }
    references
}

fn call_reference_edges(
    document: &Document<ParsedAnnotation>,
    names: &BTreeSet<String>,
) -> Vec<SourceBlockReference> {
    let mut references = Vec::new();
    document.visit(|node| {
        if let Some(reference) =
            element_call_reference(&node, names).or_else(|| object_call_reference(&node, names))
        {
            references.push(reference);
        }
    });
    references
}

fn element_call_reference(
    node: &AstRef<'_, ParsedAnnotation>,
    names: &BTreeSet<String>,
) -> Option<SourceBlockReference> {
    let AstRef::Element(element) = node else {
        return None;
    };
    let ElementData::BabelCall(keyword) = &element.data else {
        return None;
    };
    let target = babel_call_target(&keyword.value)?;
    Some(source_block_reference(
        names,
        SourceBlockSource::from_annotation(&keyword.ann),
        SourceBlockReferenceKind::BabelCall,
        None,
        target,
    ))
}

fn object_call_reference(
    node: &AstRef<'_, ParsedAnnotation>,
    names: &BTreeSet<String>,
) -> Option<SourceBlockReference> {
    let AstRef::Object(object) = node else {
        return None;
    };
    let ObjectData::InlineCall { name, .. } = &object.data else {
        return None;
    };
    let target = name.trim();
    (!target.is_empty()).then(|| {
        source_block_reference(
            names,
            SourceBlockSource::from_annotation(&object.ann),
            SourceBlockReferenceKind::InlineCall,
            None,
            target.to_string(),
        )
    })
}

fn source_block_reference(
    names: &BTreeSet<String>,
    source: SourceBlockSource,
    kind: SourceBlockReferenceKind,
    variable: Option<String>,
    target: String,
) -> SourceBlockReference {
    SourceBlockReference {
        source,
        kind,
        variable,
        resolved: names.contains(&target.to_ascii_lowercase()),
        target,
    }
}

fn source_block_names(records: &[SourceBlockRecord]) -> SourceBlockNameIndex {
    let mut source_names = BTreeSet::new();
    let mut noweb_names = BTreeSet::new();
    for record in records {
        if let Some(name) = record
            .name
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            let name = name.to_ascii_lowercase();
            source_names.insert(name.clone());
            noweb_names.insert(name);
        }
        for name in source_block_noweb_ref_names(record) {
            noweb_names.insert(name.to_ascii_lowercase());
        }
    }
    SourceBlockNameIndex {
        source_names,
        noweb_names,
    }
}

fn source_block_noweb_ref_names(record: &SourceBlockRecord) -> Vec<&str> {
    record
        .normalized_header_args
        .iter()
        .filter_map(|arg| {
            if arg.source == SourceBlockHeaderArgSource::Explicit
                && arg.key.eq_ignore_ascii_case("noweb-ref")
            {
                arg.value
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
            } else {
                None
            }
        })
        .collect()
}

fn noweb_references(value: &str) -> Vec<String> {
    let mut references = Vec::new();
    let mut rest = value;
    while let Some(start) = rest.find("<<") {
        rest = &rest[start + 2..];
        let Some(end) = rest.find(">>") else {
            break;
        };
        let raw = rest[..end].trim();
        if let Some(reference) = noweb_reference_name(raw) {
            references.push(reference.to_string());
        }
        rest = &rest[end + 2..];
    }
    references
}

fn noweb_reference_name(raw: &str) -> Option<&str> {
    let target = raw
        .split_once('(')
        .map(|(name, _)| name)
        .unwrap_or(raw)
        .trim();
    (!target.is_empty() && !target.contains(char::is_whitespace)).then_some(target)
}

fn babel_call_target(value: &str) -> Option<String> {
    let value = strip_babel_call_prefix(value.trim())
        .unwrap_or_else(|| value.trim())
        .trim_start();
    let target = value
        .split(|ch: char| ch == '(' || ch == '[' || ch.is_whitespace())
        .next()
        .unwrap_or_default()
        .trim();
    (!target.is_empty()).then(|| target.to_string())
}

fn strip_babel_call_prefix(value: &str) -> Option<&str> {
    let prefix_len = "#+call:".len();
    value
        .get(..prefix_len)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("#+call:"))
        .then(|| &value[prefix_len..])
}

fn header_var_reference_target(assignment: &str, names: &BTreeSet<String>) -> Option<String> {
    let trimmed = assignment.trim();
    if trimmed.is_empty() || is_literal_header_var_assignment(trimmed) {
        return None;
    }

    if let Some(target) = header_var_call_target(trimmed) {
        return Some(target.to_string());
    }

    let target = trimmed
        .split_once('[')
        .map(|(name, _)| name)
        .unwrap_or(trimmed)
        .trim();
    if target.is_empty() || target.contains(char::is_whitespace) || target.contains(':') {
        return None;
    }

    names
        .contains(&target.to_ascii_lowercase())
        .then(|| target.to_string())
}

fn header_var_call_target(assignment: &str) -> Option<&str> {
    let open = assignment.find('(')?;
    if !assignment.ends_with(')') {
        return None;
    }
    let target = assignment[..open]
        .split_once('[')
        .map(|(name, _)| name)
        .unwrap_or(&assignment[..open])
        .trim();
    (!target.is_empty()
        && !target.contains(char::is_whitespace)
        && !target.contains(':')
        && balanced_header_var_call(assignment))
    .then_some(target)
}

fn balanced_header_var_call(assignment: &str) -> bool {
    let mut depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote = None;
    let mut escaped = false;

    for ch in assignment.chars() {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if quote == Some(ch) {
            quote = None;
        } else if quote.is_none() && matches!(ch, '"' | '\'') {
            quote = Some(ch);
        } else if quote.is_none() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    let Some(next_depth) = depth.checked_sub(1) else {
                        return false;
                    };
                    depth = next_depth;
                }
                '[' => bracket_depth += 1,
                ']' => {
                    let Some(next_depth) = bracket_depth.checked_sub(1) else {
                        return false;
                    };
                    bracket_depth = next_depth;
                }
                _ => {}
            }
        }
    }

    depth == 0 && bracket_depth == 0 && quote.is_none() && !escaped
}

fn is_literal_header_var_assignment(value: &str) -> bool {
    value.starts_with('"')
        || value.starts_with('\'')
        || value.starts_with('[')
        || value.starts_with('(')
        || value.starts_with('{')
        || value.starts_with('*')
        || value.eq_ignore_ascii_case("nil")
        || value.eq_ignore_ascii_case("t")
        || value.eq_ignore_ascii_case("true")
        || value.eq_ignore_ascii_case("false")
        || value.parse::<f64>().is_ok()
}
