//! Source-block reference side-table projection for literate-programming tools.

use std::collections::BTreeSet;

use super::{
    AstRef, Document, ElementData, ObjectData, ParsedAnnotation, SourceBlockHeaderArgSource,
    SourceBlockRecord, SourceBlockReference, SourceBlockReferenceKind, SourceBlockSource,
};

impl Document<ParsedAnnotation> {
    /// Projects source-block name references without executing Babel.
    ///
    /// A reference resolves when it points at a local `#+NAME` source block or
    /// an explicit `:noweb-ref` header argument. The projection is intentionally
    /// file-local; workspace-level resolution belongs in host tooling.
    pub fn source_block_references(&self) -> Vec<SourceBlockReference> {
        let records = self.source_block_records();
        let names = source_block_names(&records);
        let mut references = noweb_reference_edges(&records, &names);
        references.extend(call_reference_edges(self, &names));
        references
    }
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
                target,
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
            target.to_string(),
        )
    })
}

fn source_block_reference(
    names: &BTreeSet<String>,
    source: SourceBlockSource,
    kind: SourceBlockReferenceKind,
    target: String,
) -> SourceBlockReference {
    SourceBlockReference {
        source,
        kind,
        resolved: names.contains(&target.to_ascii_lowercase()),
        target,
    }
}

fn source_block_names(records: &[SourceBlockRecord]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for record in records {
        for name in source_block_declared_names(record) {
            names.insert(name.to_ascii_lowercase());
        }
    }
    names
}

fn source_block_declared_names(record: &SourceBlockRecord) -> Vec<&str> {
    record
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .into_iter()
        .chain(record.normalized_header_args.iter().filter_map(|arg| {
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
        }))
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
    let value = value
        .trim()
        .strip_prefix("#+CALL:")
        .or_else(|| value.trim().strip_prefix("#+call:"))
        .unwrap_or_else(|| value.trim())
        .trim_start();
    let target = value
        .split(|ch: char| ch == '(' || ch == '[' || ch.is_whitespace())
        .next()
        .unwrap_or_default()
        .trim();
    (!target.is_empty()).then(|| target.to_string())
}
