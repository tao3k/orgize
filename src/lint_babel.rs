//! Babel/source-block lint checks.

use std::collections::BTreeMap;

use crate::ast::{
    ParsedAst, SourceBlockEvalPolicy, SourceBlockHeaderArgKind, SourceBlockHeaderArgSource,
    SourceBlockRecord, SourceBlockReferenceKind, SourceBlockResultCollection,
    SourceBlockTangleMode,
};

use super::lint_model::{LintFinding, LintLocation, LintSeverity, location_for_range_bounds};

pub(crate) struct BabelLintOutcome {
    pub(crate) findings: Vec<LintFinding>,
    pub(crate) receipts: Vec<crate::lint::RuntimeValidationReceipt>,
}

pub(crate) fn babel_findings_with_runtime_receipts(
    document: &ParsedAst,
    source: &str,
    source_path: Option<&std::path::Path>,
    runtime_policy: &crate::lint::RuntimeLintExecutionPolicy,
    source_context: Option<&crate::lint::RuntimeValidationSourceContext>,
) -> BabelLintOutcome {
    let records = document.source_block_records();
    let mut findings = Vec::new();
    findings.extend(duplicate_source_block_name_findings(&records, source));
    findings.extend(eval_header_findings(&records, source));
    findings.extend(execution_context_findings(&records, source));
    let runtime = runtime_lint_findings(
        &records,
        source,
        source_path,
        runtime_policy,
        source_context,
    );
    findings.extend(runtime.findings);
    findings.extend(result_file_findings(&records, source));
    findings.extend(tangle_target_findings(&records, source));
    findings.extend(missing_source_reference_findings(document, source));
    BabelLintOutcome {
        findings,
        receipts: runtime.receipts,
    }
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

fn runtime_lint_findings(
    records: &[SourceBlockRecord],
    source: &str,
    source_path: Option<&std::path::Path>,
    runtime_policy: &crate::lint::RuntimeLintExecutionPolicy,
    source_context: Option<&crate::lint::RuntimeValidationSourceContext>,
) -> RuntimeLintFindings {
    let mut findings = Vec::new();
    let mut receipts = Vec::new();
    for record in records {
        let Some(outcome) =
            runtime_lint_finding(record, source, source_path, runtime_policy, source_context)
        else {
            continue;
        };
        if let Some(finding) = outcome.finding {
            findings.push(finding);
        }
        if let Some(receipt) = outcome.receipt {
            receipts.push(receipt);
        }
    }
    RuntimeLintFindings { findings, receipts }
}

struct RuntimeLintFindings {
    findings: Vec<LintFinding>,
    receipts: Vec<crate::lint::RuntimeValidationReceipt>,
}

struct RuntimeLintFinding {
    finding: Option<LintFinding>,
    receipt: Option<crate::lint::RuntimeValidationReceipt>,
}

fn runtime_lint_finding(
    record: &SourceBlockRecord,
    source: &str,
    source_path: Option<&std::path::Path>,
    runtime_policy: &crate::lint::RuntimeLintExecutionPolicy,
    source_context: Option<&crate::lint::RuntimeValidationSourceContext>,
) -> Option<RuntimeLintFinding> {
    let language = record.language.as_deref()?.to_ascii_lowercase();
    if language != "typst" || runtime_lint_disabled(record) {
        return None;
    }
    let runtime = record
        .normalized_header_args
        .iter()
        .rev()
        .find(|arg| arg.kind == SourceBlockHeaderArgKind::Runtime)
        .and_then(|arg| arg.value.as_deref())
        .unwrap_or(&language);
    let binding = match crate::runtime::resolve_eval_binding(&language, runtime, None) {
        Ok(mut binding) => {
            let program = runtime_policy.runtime_program();
            binding.program = if runtime_policy.binding_kind()
                == crate::lint::RuntimeValidationBindingKind::ExactPath
            {
                program
                    .canonicalize()
                    .unwrap_or_else(|_| program.to_path_buf())
                    .as_os_str()
                    .to_string_lossy()
                    .into_owned()
            } else {
                program.as_os_str().to_string_lossy().into_owned()
            };
            binding
        }
        Err(error) => {
            let receipt = source_context.map(|context| {
                runtime_receipt(
                    context.clone(),
                    runtime_policy,
                    runtime_policy
                        .runtime_program()
                        .as_os_str()
                        .to_string_lossy()
                        .into_owned(),
                    Vec::new(),
                    true,
                    0,
                    0,
                    std::time::Duration::ZERO,
                    crate::lint::RuntimeValidationTerminationOutcome::IoFailed,
                    None,
                    false,
                    crate::lint::RuntimeValidationStatus::Rejected,
                    Some(crate::lint::RuntimeValidationDiagnosticCode::Org043),
                )
            });
            return Some(RuntimeLintFinding {
                finding: Some(LintFinding {
                    code: "ORG043",
                    severity: LintSeverity::Error,
                    message: format!("Typst runtime lint failed: {error}"),
                    location: location_for_source_record(source, record),
                }),
                receipt,
            });
        }
    };
    let observation = crate::runtime::execute_runtime_observed(
        &binding,
        &record.value,
        source_context
            .map(crate::lint::RuntimeValidationSourceContext::working_directory)
            .or_else(|| source_path.and_then(std::path::Path::parent)),
        runtime_policy.timeout(),
        runtime_policy.output_byte_budget(),
    );
    let accepted = observation.termination_outcome
        == crate::runtime::RuntimeTerminationOutcome::Exited
        && observation.output.exit_code == Some(0);
    let receipt = source_context.map(|context| {
        runtime_receipt(
            context.clone(),
            runtime_policy,
            binding.program.clone(),
            binding.args.clone(),
            binding.uses_stdin(),
            observation.stdout_bytes,
            observation.stderr_bytes,
            observation.elapsed,
            termination_outcome(observation.termination_outcome),
            observation.output.exit_code,
            observation.child_reaped,
            if accepted {
                crate::lint::RuntimeValidationStatus::Accepted
            } else {
                crate::lint::RuntimeValidationStatus::Rejected
            },
            (!accepted).then_some(crate::lint::RuntimeValidationDiagnosticCode::Org043),
        )
    });
    if accepted {
        return Some(RuntimeLintFinding {
            finding: None,
            receipt,
        });
    }
    let message = match observation.termination_outcome {
        crate::runtime::RuntimeTerminationOutcome::Exited => format!(
            "Typst runtime lint failed with exit code {}: {}",
            observation
                .output
                .exit_code
                .map_or_else(|| "signal".to_string(), |code| code.to_string()),
            runtime_diagnostic(&observation.output.stderr)
        ),
        crate::runtime::RuntimeTerminationOutcome::TimedOut
        | crate::runtime::RuntimeTerminationOutcome::OutputBudgetExceeded
        | crate::runtime::RuntimeTerminationOutcome::SpawnFailed
        | crate::runtime::RuntimeTerminationOutcome::IoFailed => format!(
            "Typst runtime lint failed: {}",
            observation
                .diagnostic
                .as_deref()
                .unwrap_or("runtime execution failed")
        ),
    };
    Some(RuntimeLintFinding {
        finding: Some(LintFinding {
            code: "ORG043",
            severity: LintSeverity::Error,
            message,
            location: location_for_source_record(source, record),
        }),
        receipt,
    })
}

#[allow(clippy::too_many_arguments)]
fn runtime_receipt(
    source_context: crate::lint::RuntimeValidationSourceContext,
    runtime_policy: &crate::lint::RuntimeLintExecutionPolicy,
    program: String,
    args: Vec<String>,
    stdin: bool,
    stdout_bytes: usize,
    stderr_bytes: usize,
    elapsed: std::time::Duration,
    termination_outcome: crate::lint::RuntimeValidationTerminationOutcome,
    exit_code: Option<i32>,
    child_reaped: bool,
    status: crate::lint::RuntimeValidationStatus,
    diagnostic_code: Option<crate::lint::RuntimeValidationDiagnosticCode>,
) -> crate::lint::RuntimeValidationReceipt {
    crate::lint::RuntimeValidationReceipt {
        source_context,
        binding: crate::lint::RuntimeValidationBinding {
            kind: runtime_policy.binding_kind(),
            program,
            args,
            stdin,
        },
        policy: crate::lint::RuntimeValidationPolicy {
            timeout_ms: runtime_policy.timeout().as_millis(),
            output_byte_budget: runtime_policy.output_byte_budget(),
        },
        observation: crate::lint::RuntimeValidationObservation {
            output_bytes: crate::lint::RuntimeValidationStreamBytes::new(
                stdout_bytes,
                stderr_bytes,
            )
            .expect("runtime stream byte observations must remain representable"),
            elapsed: crate::lint::RuntimeValidationElapsed::from_duration(elapsed),
            termination_outcome,
            exit_status: crate::lint::RuntimeValidationExitStatus::from_exit_code(exit_code),
            child_lifecycle: crate::lint::RuntimeValidationChildLifecycle::from_runtime(
                termination_outcome,
                child_reaped,
            ),
        },
        status,
        diagnostic_code,
    }
}

fn termination_outcome(
    outcome: crate::runtime::RuntimeTerminationOutcome,
) -> crate::lint::RuntimeValidationTerminationOutcome {
    match outcome {
        crate::runtime::RuntimeTerminationOutcome::Exited => {
            crate::lint::RuntimeValidationTerminationOutcome::Exited
        }
        crate::runtime::RuntimeTerminationOutcome::TimedOut => {
            crate::lint::RuntimeValidationTerminationOutcome::TimedOut
        }
        crate::runtime::RuntimeTerminationOutcome::OutputBudgetExceeded => {
            crate::lint::RuntimeValidationTerminationOutcome::OutputBudgetExceeded
        }
        crate::runtime::RuntimeTerminationOutcome::SpawnFailed => {
            crate::lint::RuntimeValidationTerminationOutcome::SpawnFailed
        }
        crate::runtime::RuntimeTerminationOutcome::IoFailed => {
            crate::lint::RuntimeValidationTerminationOutcome::IoFailed
        }
    }
}

fn runtime_lint_disabled(record: &SourceBlockRecord) -> bool {
    record
        .normalized_header_args
        .iter()
        .rev()
        .find(|arg| arg.kind == SourceBlockHeaderArgKind::Lint)
        .and_then(|arg| arg.value.as_deref())
        .is_some_and(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "no" | "false" | "never" | "off"
            )
        })
}

fn runtime_diagnostic(stderr: &str) -> String {
    const MAX_DIAGNOSTIC_CHARS: usize = 2_000;
    let diagnostic = stderr.trim();
    if diagnostic.is_empty() {
        return "runtime produced no diagnostic".to_string();
    }
    let mut chars = diagnostic.chars();
    let excerpt = chars
        .by_ref()
        .take(MAX_DIAGNOSTIC_CHARS)
        .collect::<String>();
    if chars.next().is_some() {
        format!("{excerpt}…")
    } else {
        excerpt
    }
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
            location: location_for_range_bounds(
                source,
                reference.source.range_start as usize,
                reference.source.range_end as usize,
            ),
        })
        .collect()
}

fn location_for_source_record(source: &str, record: &SourceBlockRecord) -> LintLocation {
    location_for_range_bounds(
        source,
        record.source.range_start as usize,
        record.source.range_end as usize,
    )
}
