//! Typed runtime-validation evidence and schema-v1 receipt projection.

use std::path::{Path, PathBuf};

use crate::Org;

use crate::lint::{
    LintOptions, LintReport, RuntimeLintExecutionPolicy, collect_lint_findings, sort_lint_findings,
};

/// Caller-supplied logical identity for a runtime-validation source document.
///
/// This context is intentionally independent from the process current
/// directory. The proof-acceptance entrypoint rejects omitted or inconsistent
/// context rather than deriving a package root from ambient state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeValidationSourceContext {
    source_path: PathBuf,
    working_directory: PathBuf,
    package_root: PathBuf,
}

impl RuntimeValidationSourceContext {
    /// Creates source context after checking that the selected working
    /// directory is exactly the source document's parent.
    pub fn new(
        source_path: PathBuf,
        working_directory: PathBuf,
        package_root: PathBuf,
    ) -> Result<Self, String> {
        if source_path.as_os_str().is_empty() {
            return Err("runtime validation source path must be supplied".to_string());
        }
        if working_directory.as_os_str().is_empty() {
            return Err("runtime validation working directory must be supplied".to_string());
        }
        if package_root.as_os_str().is_empty() {
            return Err("runtime validation package root must be supplied".to_string());
        }
        let source_parent = source_path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .ok_or_else(|| {
                "runtime validation source path must have an explicit parent directory".to_string()
            })?;
        if source_parent != working_directory {
            return Err(format!(
                "runtime validation working directory `{}` must equal source parent `{}`",
                working_directory.display(),
                source_parent.display()
            ));
        }
        Ok(Self {
            source_path,
            working_directory,
            package_root,
        })
    }

    /// Exact host path of the validated Org source.
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    /// Exact child process current directory selected by the caller.
    pub fn working_directory(&self) -> &Path {
        &self.working_directory
    }

    /// Caller-provided package-root logical identity; Orgize never infers it.
    pub fn package_root(&self) -> &Path {
        &self.package_root
    }

    fn validate_options(&self, options: &LintOptions) -> Result<(), String> {
        match options.source_path.as_deref() {
            Some(source_path) if source_path == self.source_path => Ok(()),
            Some(source_path) => Err(format!(
                "runtime validation source context `{}` does not match lint source path `{}`",
                self.source_path.display(),
                source_path.display()
            )),
            None => Err(
                "runtime validation proof entrypoint requires LintOptions.source_path".to_string(),
            ),
        }
    }
}

/// Admission decision for one runtime-validation receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeValidationStatus {
    Accepted,
    Rejected,
}

/// A schema-version-one, body-free runtime validation receipt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeValidationReceipt {
    pub source_context: RuntimeValidationSourceContext,
    pub binding: RuntimeValidationBinding,
    pub policy: RuntimeValidationPolicy,
    pub observation: RuntimeValidationObservation,
    pub status: RuntimeValidationStatus,
    pub diagnostic_code: Option<RuntimeValidationDiagnosticCode>,
}

/// Exact runtime binding selected for one validation execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeValidationBinding {
    pub kind: RuntimeValidationBindingKind,
    pub program: String,
    pub args: Vec<String>,
    pub stdin: bool,
}

/// Binding identity class recorded by the receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeValidationBindingKind {
    /// One `typst` PATH lookup token, not a resolved executable identity.
    TypstPath,
    /// One explicit caller-supplied path, still without a digest claim.
    ExactPath,
}

/// Positive execution limits applied to one validation execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeValidationPolicy {
    pub timeout_ms: u128,
    pub output_byte_budget: usize,
}

/// Observed process outcome without stream bodies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeValidationObservation {
    pub output_bytes: RuntimeValidationStreamBytes,
    pub elapsed: RuntimeValidationElapsed,
    pub termination_outcome: RuntimeValidationTerminationOutcome,
    pub exit_status: RuntimeValidationExitStatus,
    pub child_lifecycle: RuntimeValidationChildLifecycle,
}

/// Bounded byte observations for both process output streams.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeValidationStreamBytes {
    pub stdout: RuntimeValidationByteCount,
    pub stderr: RuntimeValidationByteCount,
}

impl RuntimeValidationStreamBytes {
    /// Creates stream observations only when their combined count is representable.
    pub fn new(stdout: usize, stderr: usize) -> Result<Self, String> {
        stdout.checked_add(stderr).ok_or_else(|| {
            "runtime validation combined stream byte count exceeds usize".to_string()
        })?;
        Ok(Self {
            stdout: RuntimeValidationByteCount(stdout),
            stderr: RuntimeValidationByteCount(stderr),
        })
    }

    /// Exact combined observed bytes, already checked during construction.
    pub fn combined(self) -> usize {
        self.stdout.0 + self.stderr.0
    }
}

/// Exact count observed from one bounded output stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeValidationByteCount(usize);

impl RuntimeValidationByteCount {
    /// Numeric projection used by the version-one JSON receipt.
    pub fn get(self) -> usize {
        self.0
    }
}

/// Wall-clock duration observed for one runtime validation attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeValidationElapsed(std::time::Duration);

impl RuntimeValidationElapsed {
    pub(crate) fn from_duration(duration: std::time::Duration) -> Self {
        Self(duration)
    }

    /// Millisecond projection used by the version-one JSON receipt.
    pub fn as_millis(self) -> u128 {
        self.0.as_millis()
    }
}

/// Process exit condition; a signal is deliberately not represented as code zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeValidationExitStatus {
    Code(i32),
    Signaled,
}

impl RuntimeValidationExitStatus {
    pub(crate) fn from_exit_code(exit_code: Option<i32>) -> Self {
        match exit_code {
            Some(code) => Self::Code(code),
            None => Self::Signaled,
        }
    }

    /// Nullable numeric projection used by the version-one JSON receipt.
    pub fn code(self) -> Option<i32> {
        match self {
            Self::Code(code) => Some(code),
            Self::Signaled => None,
        }
    }
}

/// Child lifecycle observation, separating an unspawned child from reaping failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeValidationChildLifecycle {
    NotSpawned,
    Reaped,
    NotReaped,
}

impl RuntimeValidationChildLifecycle {
    pub(crate) fn from_runtime(
        outcome: RuntimeValidationTerminationOutcome,
        child_reaped: bool,
    ) -> Self {
        if outcome == RuntimeValidationTerminationOutcome::SpawnFailed {
            Self::NotSpawned
        } else if child_reaped {
            Self::Reaped
        } else {
            Self::NotReaped
        }
    }

    /// Boolean projection used by the version-one JSON receipt.
    pub fn child_reaped(self) -> bool {
        matches!(self, Self::Reaped)
    }
}

/// Terminal process state distinguished by schema-v1 receipts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeValidationTerminationOutcome {
    Exited,
    TimedOut,
    OutputBudgetExceeded,
    SpawnFailed,
    IoFailed,
}

/// Stable diagnostic code, intentionally distinct from human-readable text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeValidationDiagnosticCode {
    Org043,
}

impl RuntimeValidationReceipt {
    /// Serializes the stable v1 receipt shape without runtime output bodies.
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "schemaId": "org-babel-runtime-validation-receipt.v1",
            "schemaVersion": "1",
            "sourceContext": {
                "sourcePath": self.source_context.source_path().to_string_lossy(),
                "workingDirectory": self.source_context.working_directory().to_string_lossy(),
                "packageRoot": self.source_context.package_root().to_string_lossy(),
            },
            "binding": {
                "kind": match self.binding.kind {
                    RuntimeValidationBindingKind::TypstPath => "typst-path",
                    RuntimeValidationBindingKind::ExactPath => "exact-path",
                },
                "program": self.binding.program,
                "args": self.binding.args,
                "stdin": self.binding.stdin,
            },
            "policy": {
                "timeoutMs": self.policy.timeout_ms,
                "outputByteBudget": self.policy.output_byte_budget,
            },
            "observation": {
                "stdoutBytes": self.observation.output_bytes.stdout.get(),
                "stderrBytes": self.observation.output_bytes.stderr.get(),
                "elapsedMs": self.observation.elapsed.as_millis(),
                "terminationOutcome": match self.observation.termination_outcome {
                    RuntimeValidationTerminationOutcome::Exited => "exited",
                    RuntimeValidationTerminationOutcome::TimedOut => "timed-out",
                    RuntimeValidationTerminationOutcome::OutputBudgetExceeded => "output-budget-exceeded",
                    RuntimeValidationTerminationOutcome::SpawnFailed => "spawn-failed",
                    RuntimeValidationTerminationOutcome::IoFailed => "io-failed",
                },
                "exitCode": self.observation.exit_status.code(),
                "childReaped": self.observation.child_lifecycle.child_reaped(),
            },
            "status": match self.status {
                RuntimeValidationStatus::Accepted => "accepted",
                RuntimeValidationStatus::Rejected => "rejected",
            },
            "diagnosticCode": self.diagnostic_code.map(|code| match code {
                RuntimeValidationDiagnosticCode::Org043 => "ORG043",
            }),
        })
    }
}

/// Lint report plus runtime receipts intended for dependency-level admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeValidationEvidenceReport {
    pub lint_report: LintReport,
    pub receipts: Vec<RuntimeValidationReceipt>,
}

/// Lints Org source and returns body-free, schema-v1 runtime receipts.
///
/// The caller must supply an explicit source path, working directory, and
/// package-root logical identity. This is the dependency-facing, fail-closed
/// admission entrypoint; it never guesses a package root from the current
/// process directory.
pub fn lint_org_with_runtime_validation_evidence(
    source: &str,
    options: &LintOptions,
    source_context: Option<&RuntimeValidationSourceContext>,
    runtime_policy: &RuntimeLintExecutionPolicy,
) -> Result<RuntimeValidationEvidenceReport, String> {
    let source_context = source_context.ok_or_else(|| {
        "runtime validation proof entrypoint requires explicit source context".to_string()
    })?;
    source_context.validate_options(options)?;
    let org = Org::parse(source);
    let (mut findings, receipts) = collect_lint_findings(
        &org.document(),
        source,
        options,
        runtime_policy,
        Some(source_context),
    );
    sort_lint_findings(&mut findings);
    Ok(RuntimeValidationEvidenceReport {
        lint_report: LintReport { findings },
        receipts,
    })
}
