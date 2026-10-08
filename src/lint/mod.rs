//! Org document linting over the native Scheme parser projection.

#[path = "rules/attachments.rs"]
mod attachments;
#[path = "rules/babel.rs"]
mod babel;
#[path = "rules/contracts.rs"]
mod contracts;
#[path = "rules/crypt.rs"]
mod crypt;
#[path = "rules/document.rs"]
mod document;
#[path = "rules/file_links.rs"]
mod file_links;
#[path = "rules/lifecycle.rs"]
mod lifecycle;
pub(crate) mod model;
mod pipeline;
#[path = "rules/priority.rs"]
mod priority;
#[path = "rules/progress.rs"]
mod progress;
#[path = "rules/properties.rs"]
mod properties;
mod render;
mod runtime_validation;
#[path = "rules/sdd.rs"]
mod sdd;
#[path = "rules/syntax.rs"]
mod syntax;
#[path = "rules/table_formulas.rs"]
mod table_formulas;
#[path = "rules/task_blockers.rs"]
mod task_blockers;

pub(crate) use model::location_for_range;
pub use model::{LintFinding, LintLocation, LintOptions, LintReport, LintSeverity};
pub use pipeline::{
    RuntimeLintExecutionPolicy, RuntimeLintProgramBinding, lint_document,
    lint_document_with_options, lint_document_with_options_and_runtime_policy, lint_org,
    lint_org_with_options, lint_org_with_options_and_runtime_policy,
};
pub(crate) use pipeline::{collect_lint_findings, sort_lint_findings};
pub use runtime_validation::{
    RuntimeValidationBinding, RuntimeValidationBindingKind, RuntimeValidationByteCount,
    RuntimeValidationChildLifecycle, RuntimeValidationDiagnosticCode, RuntimeValidationElapsed,
    RuntimeValidationEvidenceReport, RuntimeValidationExitStatus, RuntimeValidationObservation,
    RuntimeValidationPolicy, RuntimeValidationReceipt, RuntimeValidationSourceContext,
    RuntimeValidationStatus, RuntimeValidationStreamBytes, RuntimeValidationTerminationOutcome,
    lint_org_with_runtime_validation_evidence,
};
