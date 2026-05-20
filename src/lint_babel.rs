//! Babel/source-block lint checks.

use std::collections::BTreeMap;

use crate::ast::{
    ParsedAst, SourceBlockEvalPolicy, SourceBlockHeaderArgKind, SourceBlockHeaderArgSource,
    SourceBlockRecord, SourceBlockReferenceKind, SourceBlockResultCollection,
    SourceBlockTangleMode,
};

use super::lint_model::{LintFinding, LintLocation, LintSeverity, location_for_offsets};

pub(crate) fn babel_findings(document: &ParsedAst, source: &str) -> Vec<LintFinding> {
    let records = document.source_block_records();
    let mut findings = Vec::new();
    findings.extend(duplicate_source_block_name_findings(&records, source));
    findings.extend(eval_header_findings(&records, source));
    findings.extend(execution_context_findings(&records, source));
    findings.extend(result_file_findings(&records, source));
    findings.extend(tangle_target_findings(&records, source));
    findings.extend(missing_source_reference_findings(document, source));
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

fn execution_context_findings(records: &[SourceBlockRecord], source: &str) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    for record in records {
        let execution = &record.execution;
        if execution.session.source == SourceBlockHeaderArgSource::Explicit
            && execution.session.active
            && eval_policy_can_execute(execution.eval.policy)
        {
            findings.push(LintFinding {
                code: "ORG041",
                severity: LintSeverity::Warning,
                message: format!(
                    "source block uses stateful Babel session `:session {}`",
                    execution.session.raw
                ),
                location: location_for_source_record(source, record),
            });
        }

        if execution.cache.source == SourceBlockHeaderArgSource::Explicit
            && execution.cache.enabled
            && eval_policy_can_execute(execution.eval.policy)
        {
            findings.push(LintFinding {
                code: "ORG042",
                severity: LintSeverity::Warning,
                message: format!(
                    "source block enables Babel result cache `:cache {}`",
                    execution.cache.raw
                ),
                location: location_for_source_record(source, record),
            });
        }
    }
    findings
}

fn eval_policy_can_execute(policy: SourceBlockEvalPolicy) -> bool {
    !matches!(
        policy,
        SourceBlockEvalPolicy::No | SourceBlockEvalPolicy::Never
    )
}

fn result_file_findings(records: &[SourceBlockRecord], source: &str) -> Vec<LintFinding> {
    records
        .iter()
        .filter_map(|record| {
            if record.result_options.collection != Some(SourceBlockResultCollection::File)
                || record.result_options.file.is_some()
            {
                return None;
            }
            Some(LintFinding {
                code: "ORG043",
                severity: LintSeverity::Warning,
                message: "source block declares `:results file` without an explicit `:file` target"
                    .to_string(),
                location: location_for_source_record(source, record),
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

fn missing_source_reference_findings(document: &ParsedAst, source: &str) -> Vec<LintFinding> {
    document
        .source_block_references()
        .into_iter()
        .filter(|reference| !reference.resolved)
        .map(|reference| LintFinding {
            code: "ORG021",
            severity: LintSeverity::Warning,
            message: match reference.kind {
                SourceBlockReferenceKind::BabelCall => {
                    format!(
                        "Babel call target `{}` has no local source block",
                        reference.target
                    )
                }
                SourceBlockReferenceKind::HeaderVar => {
                    let variable = reference.variable.as_deref().unwrap_or("unknown");
                    format!(
                        "source block header variable `{variable}` references missing source block `{}`",
                        reference.target
                    )
                }
                SourceBlockReferenceKind::InlineCall => {
                    format!(
                        "inline Babel call target `{}` has no local source block",
                        reference.target
                    )
                }
                SourceBlockReferenceKind::Noweb => {
                    format!(
                        "noweb reference `{}` has no local source block",
                        reference.target
                    )
                }
            },
            location: location_for_offsets(
                source,
                reference.source.range_start as usize,
                reference.source.range_end as usize,
            ),
        })
        .collect()
}

fn location_for_source_record(source: &str, record: &SourceBlockRecord) -> LintLocation {
    location_for_offsets(
        source,
        record.source.range_start as usize,
        record.source.range_end as usize,
    )
}
