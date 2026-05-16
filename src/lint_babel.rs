//! Babel/source-block lint checks.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{
    AstRef, ElementData, ParsedAst, SourceBlockHeaderArgKind, SourceBlockHeaderArgSource,
    SourceBlockRecord, SourceBlockTangleMode,
};

use super::lint_model::{
    location_for_offsets, location_for_range, LintFinding, LintLocation, LintSeverity,
};

pub(crate) fn babel_findings(document: &ParsedAst, source: &str) -> Vec<LintFinding> {
    let records = document.source_block_records();
    let mut findings = Vec::new();
    findings.extend(duplicate_source_block_name_findings(&records, source));
    findings.extend(eval_header_findings(&records, source));
    findings.extend(tangle_target_findings(&records, source));
    findings.extend(missing_noweb_target_findings(&records, source));
    findings.extend(missing_babel_call_target_findings(
        document, &records, source,
    ));
    findings
}

fn duplicate_source_block_name_findings(
    records: &[SourceBlockRecord],
    source: &str,
) -> Vec<LintFinding> {
    let mut by_name = BTreeMap::<String, Vec<&SourceBlockRecord>>::new();
    for record in records {
        let Some(name) = record
            .name
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
        else {
            continue;
        };
        by_name
            .entry(name.to_ascii_lowercase())
            .or_default()
            .push(record);
    }

    by_name
        .into_iter()
        .filter_map(|(name, records)| {
            (records.len() > 1).then(|| LintFinding {
                code: "ORG020",
                severity: LintSeverity::Warning,
                message: format!(
                    "source block name `{name}` is defined {} times",
                    records.len()
                ),
                location: location_for_source_record(source, records[1]),
            })
        })
        .collect()
}

fn eval_header_findings(records: &[SourceBlockRecord], source: &str) -> Vec<LintFinding> {
    records
        .iter()
        .flat_map(|record| {
            record.normalized_header_args.iter().filter_map(|arg| {
                if arg.source != SourceBlockHeaderArgSource::Explicit
                    || arg.kind != SourceBlockHeaderArgKind::Eval
                {
                    return None;
                }
                let value = arg.value.as_deref().unwrap_or("yes").trim();
                if matches!(value.to_ascii_lowercase().as_str(), "yes" | "query") {
                    Some(LintFinding {
                        code: "ORG022",
                        severity: LintSeverity::Warning,
                        message: format!("source block uses eval-sensitive header `:eval {value}`"),
                        location: location_for_source_record(source, record),
                    })
                } else {
                    None
                }
            })
        })
        .collect()
}

fn tangle_target_findings(records: &[SourceBlockRecord], source: &str) -> Vec<LintFinding> {
    records
        .iter()
        .filter_map(|record| {
            let tangle = record.tangle.as_ref()?;
            if tangle.mode == SourceBlockTangleMode::File
                && tangle
                    .target
                    .as_deref()
                    .map(str::trim)
                    .unwrap_or_default()
                    .is_empty()
            {
                Some(LintFinding {
                    code: "ORG023",
                    severity: LintSeverity::Warning,
                    message: "source block tangle target is empty".to_string(),
                    location: location_for_source_record(source, record),
                })
            } else {
                None
            }
        })
        .collect()
}

fn missing_noweb_target_findings(records: &[SourceBlockRecord], source: &str) -> Vec<LintFinding> {
    let names = source_block_names(records);
    records
        .iter()
        .flat_map(|record| {
            noweb_references(&record.value)
                .into_iter()
                .filter_map(|reference| {
                    (!names.contains(&reference.to_ascii_lowercase())).then(|| LintFinding {
                        code: "ORG021",
                        severity: LintSeverity::Warning,
                        message: format!("noweb reference `{reference}` has no local source block"),
                        location: location_for_source_record(source, record),
                    })
                })
        })
        .collect()
}

fn missing_babel_call_target_findings(
    document: &ParsedAst,
    records: &[SourceBlockRecord],
    source: &str,
) -> Vec<LintFinding> {
    let names = source_block_names(records);
    let mut findings = Vec::new();
    document.visit(|node| {
        let AstRef::Element(element) = node else {
            return;
        };
        let ElementData::BabelCall(keyword) = &element.data else {
            return;
        };
        let Some(target) = babel_call_target(&keyword.value) else {
            return;
        };
        if !names.contains(&target.to_ascii_lowercase()) {
            findings.push(LintFinding {
                code: "ORG021",
                severity: LintSeverity::Warning,
                message: format!("Babel call target `{target}` has no local source block"),
                location: location_for_range(source, keyword.ann.range),
            });
        }
    });
    findings
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
        .unwrap_or_else(|| value.trim());
    let target = value
        .split(|ch: char| ch == '(' || ch == '[' || ch.is_whitespace())
        .next()
        .unwrap_or_default()
        .trim();
    (!target.is_empty()).then(|| target.to_string())
}

fn location_for_source_record(source: &str, record: &SourceBlockRecord) -> LintLocation {
    location_for_offsets(
        source,
        record.source.range_start as usize,
        record.source.range_end as usize,
    )
}
