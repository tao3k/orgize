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
    let name_range = keyword.ann.babel_call_name_range?;
    let keyword_start = usize::from(keyword.ann.range.start());
    let start = usize::from(name_range.start()).checked_sub(keyword_start)?;
    let end = usize::from(name_range.end()).checked_sub(keyword_start)?;
    let target = keyword.ann.raw.get(start..end)?.to_owned();
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
    super::org_values::rows("noweb-references", &[value])
        .pop()
        .expect("native noweb row")
}
fn header_var_reference_target(assignment: &str, names: &BTreeSet<String>) -> Option<String> {
    let row = super::org_values::optional("var-target", assignment)?;
    let [target, policy]: [String; 2] = row.try_into().expect("native reference arity");
    match policy.as_str() {
        "call" => Some(target),
        "named" => names
            .contains(&super::org_values::scalar("ascii-lower", &[&target]))
            .then_some(target),
        _ => panic!("native reference policy"),
    }
}
