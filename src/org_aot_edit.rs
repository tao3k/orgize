//! Source-bound edits over the Scheme-AOT Org Element graph.

use std::collections::BTreeSet;

use sha2::{Digest, Sha256};

use crate::org_aot::{OrgAotDocument, OrgAotError, parse_org_aot};

/// One owner-reviewed replacement within a source-backed Org node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrgSourceEdit<'a> {
    pub node_id: &'a str,
    pub start_byte: usize,
    pub end_byte: usize,
    pub expected_old: &'a str,
    pub replacement: &'a str,
}

/// A source or identity conflict. No candidate text is returned on failure.
#[derive(Debug)]
pub enum OrgSourceEditError {
    StaleSource,
    EmptyEdits,
    InvalidRange,
    Overlap,
    DuplicateNodeId,
    MissingNodeId,
    AmbiguousNodeId,
    WrongNode,
    ChangedContent,
    Parse(OrgAotError),
}

/// SHA-256 over the exact UTF-8 Org source, including whitespace and line endings.
#[must_use]
pub fn org_source_digest(source: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(source.as_bytes()))
}

/// Return candidate source only if every projected node anchor is still valid.
///
/// This is source-owned transformation, not authorization or file I/O. The
/// caller must review and recheck the source before persisting the candidate.
///
/// # Errors
///
/// Rejects stale source, malformed or overlapping ranges, absent/ambiguous
/// node identities, wrong-node anchors, changed content, and AOT parse errors.
pub fn apply_org_source_edits(
    source: &str,
    projected_source_digest: &str,
    edits: &[OrgSourceEdit<'_>],
) -> Result<String, OrgSourceEditError> {
    if org_source_digest(source) != projected_source_digest {
        return Err(OrgSourceEditError::StaleSource);
    }
    if edits.is_empty() {
        return Err(OrgSourceEditError::EmptyEdits);
    }
    let document = parse_org_aot(source).map_err(OrgSourceEditError::Parse)?;
    let mut ordered = edits.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|edit| (edit.start_byte, edit.end_byte));
    let mut seen_ids = BTreeSet::new();
    let mut prior_end = 0;
    for edit in &ordered {
        if !seen_ids.insert(edit.node_id) {
            return Err(OrgSourceEditError::DuplicateNodeId);
        }
        if edit.start_byte >= edit.end_byte
            || edit.end_byte > source.len()
            || !source.is_char_boundary(edit.start_byte)
            || !source.is_char_boundary(edit.end_byte)
        {
            return Err(OrgSourceEditError::InvalidRange);
        }
        if edit.start_byte < prior_end {
            return Err(OrgSourceEditError::Overlap);
        }
        if source.get(edit.start_byte..edit.end_byte) != Some(edit.expected_old) {
            return Err(OrgSourceEditError::ChangedContent);
        }
        validate_node_anchor(&document, edit)?;
        prior_end = edit.end_byte;
    }
    let mut candidate = source.to_owned();
    for edit in ordered.into_iter().rev() {
        candidate.replace_range(edit.start_byte..edit.end_byte, edit.replacement);
    }
    let revised = parse_org_aot(&candidate).map_err(OrgSourceEditError::Parse)?;
    for edit in edits {
        owner_for_id(&revised, edit.node_id)?;
    }
    Ok(candidate)
}

fn validate_node_anchor(
    document: &OrgAotDocument,
    edit: &OrgSourceEdit<'_>,
) -> Result<(), OrgSourceEditError> {
    let records = document.records();
    let owner = owner_for_id(document, edit.node_id)?;
    let span_owner = records
        .iter()
        .filter(|record| {
            let start = usize::from(record.range.start());
            let end = usize::from(record.range.end());
            start <= edit.start_byte && edit.end_byte <= end
        })
        .min_by_key(|record| usize::from(record.range.len()))
        .and_then(|record| nearest_headline(records, record.id));
    if span_owner != Some(owner) {
        return Err(OrgSourceEditError::WrongNode);
    }
    Ok(())
}

fn owner_for_id(document: &OrgAotDocument, node_id: &str) -> Result<usize, OrgSourceEditError> {
    let records = document.records();
    let owners = records
        .iter()
        .filter(|property| {
            property.kind == "node-property"
                && property
                    .field("key")
                    .is_some_and(|key| key.eq_ignore_ascii_case("ID"))
                && property.field("value") == Some(node_id)
        })
        .filter_map(|property| nearest_headline(records, property.id))
        .collect::<Vec<_>>();
    match owners.as_slice() {
        [] => Err(OrgSourceEditError::MissingNodeId),
        [owner] => Ok(*owner),
        _ => Err(OrgSourceEditError::AmbiguousNodeId),
    }
}

fn nearest_headline(records: &[gerbil_parser_rowan::GraphRecord], id: usize) -> Option<usize> {
    let mut cursor = Some(id);
    while let Some(current) = cursor {
        let record = records.get(current)?;
        if record.kind == "headline" {
            return Some(current);
        }
        cursor = record.parent_id;
    }
    None
}
