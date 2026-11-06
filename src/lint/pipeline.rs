//! Ordered lint stages and public execution entrypoints.

use super::{
    LintFinding, LintOptions, LintReport, RuntimeValidationBindingKind, RuntimeValidationReceipt,
    RuntimeValidationSourceContext,
    attachments::attachment_findings,
    babel,
    contracts::{builtin_contract_org_findings, contract_org_findings},
    crypt::crypt_findings,
    document::{
        duplicate_macro_definition_findings, duplicate_target_findings, finding_from_diagnostic,
        include_path_findings, link_abbreviation_definition_findings, missing_macro_findings,
        options_keyword_findings, todo_declaration_findings,
    },
    file_links::file_link_findings,
    lifecycle::lifecycle_findings,
    priority::priority_cookie_findings,
    progress::progress_findings,
    properties::property_drawer_findings,
    sdd::sdd_findings,
    syntax,
    table_formulas::table_formula_findings,
    task_blockers::task_blocker_findings,
};
use crate::{
    Org,
    ast::{OrgContractEvaluationContext, ParsedAst},
};

/// Lints Org source with the default parser configuration.
pub fn lint_org(source: &str) -> LintReport {
    lint_org_with_options(source, &LintOptions::default())
}

/// Bounded execution policy for source-block runtime linting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLintExecutionPolicy {
    timeout: std::time::Duration,
    output_byte_budget: std::num::NonZeroUsize,
    runtime_program: RuntimeLintProgramBinding,
}

/// Exact executable selected for runtime linting; no alternate is attempted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeLintProgramBinding {
    /// Resolve the one `typst` PATH lookup token through the process environment.
    ///
    /// The resulting receipt records `typst-path`; it does not claim a resolved
    /// executable path or executable digest.
    Typst,
    /// Execute one caller-supplied path, primarily for hermetic dependency injection.
    ///
    /// The resulting receipt records `exact-path`; it still does not claim an
    /// executable digest.
    Exact(std::path::PathBuf),
}

impl RuntimeLintExecutionPolicy {
    /// Creates a runtime lint policy with strictly positive execution bounds.
    pub fn bounded(
        timeout: std::time::Duration,
        output_byte_budget: usize,
    ) -> Result<Self, String> {
        if timeout.is_zero() {
            return Err("runtime lint timeout must be greater than zero".to_string());
        }
        let output_byte_budget =
            std::num::NonZeroUsize::new(output_byte_budget).ok_or_else(|| {
                "runtime lint output byte budget must be greater than zero".to_string()
            })?;
        Ok(Self {
            timeout,
            output_byte_budget,
            runtime_program: RuntimeLintProgramBinding::Typst,
        })
    }

    /// Selects exactly one runtime executable binding; no alternate is attempted.
    pub fn with_program_binding(mut self, binding: RuntimeLintProgramBinding) -> Self {
        self.runtime_program = binding;
        self
    }

    pub(crate) fn timeout(&self) -> std::time::Duration {
        self.timeout
    }

    pub(crate) fn output_byte_budget(&self) -> usize {
        self.output_byte_budget.get()
    }

    pub(crate) fn runtime_program(&self) -> &std::path::Path {
        match &self.runtime_program {
            RuntimeLintProgramBinding::Typst => std::path::Path::new("typst"),
            RuntimeLintProgramBinding::Exact(program) => program,
        }
    }

    /// Receipt binding classification. Neither classification is an executable
    /// identity or digest proof.
    pub(crate) fn binding_kind(&self) -> RuntimeValidationBindingKind {
        match self.runtime_program {
            RuntimeLintProgramBinding::Typst => RuntimeValidationBindingKind::TypstPath,
            RuntimeLintProgramBinding::Exact(_) => RuntimeValidationBindingKind::ExactPath,
        }
    }
}

impl Default for RuntimeLintExecutionPolicy {
    fn default() -> Self {
        Self::bounded(std::time::Duration::from_secs(30), 1_048_576)
            .expect("default runtime lint execution policy must be bounded")
    }
}

/// Lints Org source with explicit lint options.
pub fn lint_org_with_options(source: &str, options: &LintOptions) -> LintReport {
    lint_org_with_options_and_runtime_policy(
        source,
        options,
        &RuntimeLintExecutionPolicy::default(),
    )
}

/// Lints Org source with explicit lint options and runtime execution policy.
pub fn lint_org_with_options_and_runtime_policy(
    source: &str,
    options: &LintOptions,
    runtime_policy: &RuntimeLintExecutionPolicy,
) -> LintReport {
    let org = Org::parse(source);
    let (mut findings, _) = collect_lint_findings(
        &org.document(),
        source,
        options,
        runtime_policy,
        None,
        Some(&org),
    );
    sort_lint_findings(&mut findings);
    LintReport { findings }
}

/// Lints an already projected semantic document.
pub fn lint_document(document: &ParsedAst, source: &str) -> LintReport {
    lint_document_with_options(document, source, &LintOptions::default())
}

/// Lints an already projected semantic document with explicit lint options.
pub fn lint_document_with_options(
    document: &ParsedAst,
    source: &str,
    options: &LintOptions,
) -> LintReport {
    lint_document_with_options_and_runtime_policy(
        document,
        source,
        options,
        &RuntimeLintExecutionPolicy::default(),
    )
}

/// Lints an already projected document with an explicit runtime execution policy.
pub fn lint_document_with_options_and_runtime_policy(
    document: &ParsedAst,
    source: &str,
    options: &LintOptions,
    runtime_policy: &RuntimeLintExecutionPolicy,
) -> LintReport {
    let (mut findings, _) =
        collect_lint_findings(document, source, options, runtime_policy, None, None);
    sort_lint_findings(&mut findings);
    LintReport { findings }
}

pub(crate) fn collect_lint_findings(
    document: &ParsedAst,
    source: &str,
    options: &LintOptions,
    runtime_policy: &RuntimeLintExecutionPolicy,
    source_context: Option<&RuntimeValidationSourceContext>,
    source_org: Option<&Org>,
) -> (Vec<LintFinding>, Vec<RuntimeValidationReceipt>) {
    let mut findings = Vec::new();
    findings.extend(syntax::source_syntax_findings(document, source));

    findings.extend(
        document
            .diagnostics
            .iter()
            .map(|diagnostic| finding_from_diagnostic(diagnostic, source)),
    );
    findings.extend(duplicate_target_findings(&document.targets, source));
    findings.extend(include_path_findings(&document.includes, source, options));
    findings.extend(duplicate_macro_definition_findings(
        &document.macro_definitions,
        source,
    ));
    findings.extend(missing_macro_findings(document, source));
    findings.extend(link_abbreviation_definition_findings(
        &document.metadata,
        source,
    ));
    findings.extend(options_keyword_findings(&document.metadata, source));
    findings.extend(priority_cookie_findings(
        document,
        source,
        &options.priority_profile,
    ));
    findings.extend(property_drawer_findings(
        document,
        source,
        &options.property_schema_registry,
    ));
    findings.extend(progress_findings(document, source));
    findings.extend(attachment_findings(document, source, options));
    let babel = babel::babel_findings_with_runtime_receipts(
        document,
        source,
        options.source_path.as_deref(),
        runtime_policy,
        source_context,
    );
    findings.extend(babel.findings);
    findings.extend(file_link_findings(document, source, options));
    findings.extend(lifecycle_findings(document, source, options));
    findings.extend(table_formula_findings(document, source));
    findings.extend(task_blocker_findings(document, source));
    findings.extend(sdd_findings(document, source));
    findings.extend(crypt_findings(document, source));
    findings.extend(builtin_contract_org_findings(document, source, source_org));
    let org_contract_context = OrgContractEvaluationContext {
        source_path: options.source_path.clone(),
        metadata_keys: Vec::new(),
        dir_scope: None,
    };
    findings.extend(contract_org_findings(
        document,
        source,
        &options.org_contract_registry,
        &org_contract_context,
    ));
    findings.extend(todo_declaration_findings(document, source));

    (findings, babel.receipts)
}

pub(crate) fn sort_lint_findings(findings: &mut [LintFinding]) {
    findings.sort_by(|left, right| {
        left.location
            .range_start
            .cmp(&right.location.range_start)
            .then_with(|| left.code.cmp(right.code))
            .then_with(|| left.message.cmp(&right.message))
    });
}
