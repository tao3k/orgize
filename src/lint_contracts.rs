//! `CONTRACT_ORG` semantic assertion linting.

use std::path::Path;

use rowan::TextRange;

use crate::Org;
use crate::ast::{
    CONTRACT_ORG_PROPERTY, ElementData, Keyword, OrgContract, OrgContractAssertionEvaluation,
    OrgContractEvaluationContext, OrgContractEvaluationScope, OrgContractQuery,
    OrgContractRegistry, OrgContractScope, OrgElementQueryPredicate,
    OrgElementsIndexSummaryPredicate, OrgElementsIndexSummaryTextPredicate,
    OrgElementsIndexSummaryValue, ParsedAnnotation, ParsedAst, Property, Section,
    evaluate_org_contract_with_context, parse_contract_reference, parse_contracts_from_document,
};

use super::{LintFinding, LintSeverity, location_for_range};

#[path = "lint_contracts_builtin.rs"]
mod builtin;

pub(crate) fn builtin_contract_org_findings(
    document: &ParsedAst,
    source: &str,
) -> Vec<LintFinding> {
    if !has_top_level_body_text(document, source) || has_contract_org_override(document) {
        return Vec::new();
    }

    let registry = builtin_lint_contract_registry();
    let context = OrgContractEvaluationContext::default();
    let mut findings = Vec::new();
    for contract in &registry.contracts {
        if contract.scope != OrgContractScope::Document {
            continue;
        }
        push_contract_findings(
            document,
            source,
            contract,
            ContractScopeInstance::document(),
            &context,
            &mut findings,
        );
    }
    findings
}

fn builtin_lint_contract_registry() -> OrgContractRegistry {
    let mut contracts = Vec::new();
    for (_name, source) in builtin::BUILTIN_LINT_CONTRACT_SOURCES {
        let document = Org::parse(source).document();
        contracts.extend(parse_contracts_from_document(&document, None).contracts);
    }
    OrgContractRegistry::new(contracts)
}

fn has_top_level_body_text(document: &ParsedAst, source: &str) -> bool {
    document.children.iter().any(|element| {
        if !matches!(&element.data, ElementData::Paragraph(_)) {
            return false;
        }

        let start = usize::from(element.ann.range.start());
        let end = usize::from(element.ann.range.end());
        source
            .get(start..end)
            .is_some_and(|text| text.chars().any(|ch| !ch.is_whitespace()))
    })
}

fn has_contract_org_override(document: &ParsedAst) -> bool {
    !document_contract_bindings(document).is_empty()
        || document.sections.iter().any(section_has_contract_override)
}

fn section_has_contract_override(section: &Section<ParsedAnnotation>) -> bool {
    !section_contract_bindings(section).is_empty()
        || section
            .subsections
            .iter()
            .any(section_has_contract_override)
}

pub(crate) fn contract_org_findings(
    document: &ParsedAst,
    source: &str,
    registry: &OrgContractRegistry,
    context: &OrgContractEvaluationContext,
) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    let document_contracts = resolve_bindings(
        document_contract_bindings(document),
        registry,
        source,
        context.source_path(),
        &mut findings,
    );
    let mut document_default_contracts = Vec::new();
    for contract in document_contracts {
        if contract.scope == OrgContractScope::Document {
            push_contract_findings(
                document,
                source,
                contract,
                ContractScopeInstance::document(),
                context,
                &mut findings,
            );
        } else if contract.scope == OrgContractScope::Subtree {
            document_default_contracts.push(contract);
        }
    }

    {
        let mut collector = SectionContractFindingCollector {
            document,
            source,
            registry,
            context,
            findings: &mut findings,
        };
        for section in &document.sections {
            collector.collect(section, Vec::new(), &document_default_contracts);
        }
    }
    findings
}

struct SectionContractFindingCollector<'a> {
    document: &'a ParsedAst,
    source: &'a str,
    registry: &'a OrgContractRegistry,
    context: &'a OrgContractEvaluationContext,
    findings: &'a mut Vec<LintFinding>,
}

impl<'a> SectionContractFindingCollector<'a> {
    fn collect(
        &mut self,
        section: &Section<ParsedAnnotation>,
        mut outline_path: Vec<String>,
        inherited_contracts: &[&'a OrgContract],
    ) {
        outline_path.push(section.raw_title.trim_end().to_string());

        let section_bindings = section_contract_bindings(section);
        let section_contracts = if section_bindings.is_empty() {
            inherited_contracts.to_vec()
        } else {
            resolve_bindings(
                section_bindings,
                self.registry,
                self.source,
                self.context.source_path(),
                self.findings,
            )
            .into_iter()
            .filter(|contract| contract.scope == OrgContractScope::Subtree)
            .collect()
        };

        for contract in section_contracts {
            push_contract_findings(
                self.document,
                self.source,
                contract,
                ContractScopeInstance::section(section, outline_path.clone()),
                self.context,
                self.findings,
            );
        }

        for child in &section.subsections {
            self.collect(child, outline_path.clone(), &[]);
        }
    }
}

fn push_contract_findings(
    document: &ParsedAst,
    source: &str,
    contract: &OrgContract,
    scope: ContractScopeInstance,
    context: &OrgContractEvaluationContext,
    findings: &mut Vec<LintFinding>,
) {
    let evaluation =
        evaluate_org_contract_with_context(document, contract, scope.evaluation_scope(), context);
    for (assertion, assertion_evaluation) in contract.assertions.iter().zip(evaluation.assertions) {
        if !assertion_evaluation.status.is_failed() {
            continue;
        }
        findings.push(LintFinding {
            code: "ORG044",
            severity: match assertion_evaluation.severity {
                crate::ast::OrgContractSeverity::Error => LintSeverity::Error,
                crate::ast::OrgContractSeverity::Warning => LintSeverity::Warning,
            },
            message: assertion_message(AssertionMessageContext {
                contract_id: contract.id.as_str(),
                assertion_id: assertion_evaluation.assertion_id.as_str(),
                template: assertion_evaluation.message_template.as_deref(),
                fix_template: assertion_evaluation.fix_template.as_deref(),
                scope: &scope,
                query: &assertion.query,
                evaluation: &assertion_evaluation,
            }),
            location: location_for_range(source, scope.range),
        });
    }
}

struct AssertionMessageContext<'a> {
    contract_id: &'a str,
    assertion_id: &'a str,
    template: Option<&'a str>,
    fix_template: Option<&'a str>,
    scope: &'a ContractScopeInstance,
    query: &'a OrgContractQuery,
    evaluation: &'a OrgContractAssertionEvaluation,
}

fn assertion_message(context: AssertionMessageContext<'_>) -> String {
    let rendered = context
        .template
        .map(|template| render_contract_template(template, context.scope, context.evaluation))
        .unwrap_or_default();
    let predicate_detail = boolean_query_summary(context.query)
        .map(|summary| format!("; predicate {summary}"))
        .unwrap_or_default();
    let detail = format!(
        "contract `{contract_id}` assertion `{assertion_id}` failed in {} `{}`: expected {}, actual {actual}{predicate_detail}",
        context.scope.kind.as_str(),
        context.scope.title.as_deref().unwrap_or("<document>"),
        context.evaluation.expectation.expected_summary(),
        contract_id = context.contract_id,
        assertion_id = context.assertion_id,
        actual = context.evaluation.actual_count,
    );
    if rendered.trim().is_empty() {
        append_rendered_fix(
            detail,
            context.fix_template,
            context.scope,
            context.evaluation,
        )
    } else {
        append_rendered_fix(
            format!("{} ({detail})", rendered.trim()),
            context.fix_template,
            context.scope,
            context.evaluation,
        )
    }
}

fn boolean_query_summary(query: &OrgContractQuery) -> Option<String> {
    let summaries = query
        .predicates
        .iter()
        .filter_map(boolean_predicate_summary)
        .collect::<Vec<_>>();
    (!summaries.is_empty()).then(|| summaries.join(", "))
}

fn boolean_predicate_summary(predicate: &OrgElementQueryPredicate) -> Option<String> {
    match predicate {
        OrgElementQueryPredicate::All(predicates) => {
            if predicates.iter().any(|predicate| {
                matches!(
                    predicate,
                    OrgElementQueryPredicate::Any(_)
                        | OrgElementQueryPredicate::Not(_)
                        | OrgElementQueryPredicate::All(_)
                )
            }) {
                Some(predicate_summary(predicate))
            } else {
                None
            }
        }
        OrgElementQueryPredicate::Any(_) | OrgElementQueryPredicate::Not(_) => {
            Some(predicate_summary(predicate))
        }
        _ => None,
    }
}

fn predicate_summary(predicate: &OrgElementQueryPredicate) -> String {
    match predicate {
        OrgElementQueryPredicate::All(predicates) => {
            format!("all({})", predicate_list_summary(predicates))
        }
        OrgElementQueryPredicate::Any(predicates) => {
            format!("any({})", predicate_list_summary(predicates))
        }
        OrgElementQueryPredicate::Not(predicate) => {
            format!("not({})", predicate_summary(predicate))
        }
        OrgElementQueryPredicate::Category(category) => {
            format!("category == {}", category.as_str())
        }
        OrgElementQueryPredicate::Kind(kind) => format!("kind == {}", kind.as_str()),
        OrgElementQueryPredicate::AffiliatedName(name) => {
            format!("affiliatedName == {name:?}")
        }
        OrgElementQueryPredicate::Context(context) => format!("context == {context:?}"),
        OrgElementQueryPredicate::PropertyEquals(predicate) => {
            summary_predicate_summary("property", "==", predicate)
        }
        OrgElementQueryPredicate::PropertyContains(predicate) => {
            text_predicate_summary("property", "contains", predicate)
        }
        OrgElementQueryPredicate::SummaryEquals(predicate) => {
            summary_predicate_summary("summary", "==", predicate)
        }
        OrgElementQueryPredicate::SummaryContains(predicate) => {
            text_predicate_summary("summary", "contains", predicate)
        }
    }
}

fn predicate_list_summary(predicates: &[OrgElementQueryPredicate]) -> String {
    predicates
        .iter()
        .map(predicate_summary)
        .collect::<Vec<_>>()
        .join(", ")
}

fn summary_predicate_summary(
    field: &str,
    operator: &str,
    predicate: &OrgElementsIndexSummaryPredicate,
) -> String {
    format!(
        "{field}({}) {operator} {}",
        predicate.key,
        summary_value_summary(&predicate.value)
    )
}

fn text_predicate_summary(
    field: &str,
    operator: &str,
    predicate: &OrgElementsIndexSummaryTextPredicate,
) -> String {
    format!(
        "{field}({}) {operator} {:?}",
        predicate.key, predicate.needle
    )
}

fn summary_value_summary(value: &OrgElementsIndexSummaryValue) -> String {
    match value {
        OrgElementsIndexSummaryValue::Null => "null".to_string(),
        OrgElementsIndexSummaryValue::Bool(value) => value.to_string(),
        OrgElementsIndexSummaryValue::Integer(value) => value.to_string(),
        OrgElementsIndexSummaryValue::Text(value) => format!("{value:?}"),
        OrgElementsIndexSummaryValue::StringList(values) => {
            format!(
                "[{}]",
                values
                    .iter()
                    .map(|value| format!("{value:?}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

fn append_rendered_fix(
    message: String,
    fix_template: Option<&str>,
    scope: &ContractScopeInstance,
    evaluation: &OrgContractAssertionEvaluation,
) -> String {
    let rendered_fix = fix_template
        .map(|template| render_contract_template(template, scope, evaluation))
        .unwrap_or_default();
    let rendered_fix = rendered_fix
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if rendered_fix.is_empty() {
        message
    } else {
        format!("{message} Suggested fix: {rendered_fix}")
    }
}

fn render_contract_template(
    template: &str,
    scope: &ContractScopeInstance,
    evaluation: &OrgContractAssertionEvaluation,
) -> String {
    template
        .replace("{{ scope.title }}", scope.title.as_deref().unwrap_or(""))
        .replace("{{scope.title}}", scope.title.as_deref().unwrap_or(""))
        .replace("{{ scope.kind }}", scope.kind.as_str())
        .replace("{{scope.kind}}", scope.kind.as_str())
        .replace("{{ result.count }}", &evaluation.actual_count.to_string())
        .replace("{{result.count}}", &evaluation.actual_count.to_string())
        .replace("{{ expected }}", &evaluation.expectation.expected_summary())
        .replace("{{expected}}", &evaluation.expectation.expected_summary())
}

fn resolve_binding<'a>(
    binding: ContractBinding,
    registry: &'a OrgContractRegistry,
    source: &str,
    source_path: Option<&Path>,
    findings: &mut Vec<LintFinding>,
) -> Option<&'a OrgContract> {
    if binding.reference.raw.trim().is_empty() {
        findings.push(LintFinding {
            code: "ORG044",
            severity: LintSeverity::Warning,
            message: "CONTRACT_ORG is empty; load or choose an Org contract id".to_string(),
            location: location_for_range(source, binding.range),
        });
        return None;
    }
    let reference = binding.reference.with_source_relative_path(source_path);
    let Some(contract) = registry
        .resolve(&reference)
        .or_else(|| registry.resolve(&binding.reference))
    else {
        findings.push(LintFinding {
            code: "ORG044",
            severity: LintSeverity::Warning,
            message: format!(
                "CONTRACT_ORG `{}` was not found in the loaded Org contract registry",
                binding.reference.raw
            ),
            location: location_for_range(source, binding.range),
        });
        return None;
    };
    Some(contract)
}

fn resolve_bindings<'a>(
    bindings: Vec<ContractBinding>,
    registry: &'a OrgContractRegistry,
    source: &str,
    source_path: Option<&Path>,
    findings: &mut Vec<LintFinding>,
) -> Vec<&'a OrgContract> {
    bindings
        .into_iter()
        .filter_map(|binding| resolve_binding(binding, registry, source, source_path, findings))
        .collect()
}

fn document_contract_bindings(document: &ParsedAst) -> Vec<ContractBinding> {
    let properties = property_contract_bindings(&document.properties);
    if properties.is_empty() {
        keyword_contract_bindings(&document.metadata)
    } else {
        properties
    }
}

fn section_contract_bindings(section: &Section<ParsedAnnotation>) -> Vec<ContractBinding> {
    property_contract_bindings(&section.properties)
}

fn property_contract_bindings(properties: &[Property<ParsedAnnotation>]) -> Vec<ContractBinding> {
    properties
        .iter()
        .filter(|property| property.key.eq_ignore_ascii_case(CONTRACT_ORG_PROPERTY))
        .map(|property| ContractBinding {
            reference: parse_contract_reference(property.value.as_str()),
            range: property.ann.range,
        })
        .collect()
}

fn keyword_contract_bindings(keywords: &[Keyword<ParsedAnnotation>]) -> Vec<ContractBinding> {
    keywords
        .iter()
        .filter(|keyword| keyword.key.eq_ignore_ascii_case(CONTRACT_ORG_PROPERTY))
        .map(|keyword| ContractBinding {
            reference: parse_contract_reference(keyword.value.as_str()),
            range: keyword.ann.range,
        })
        .collect()
}

#[derive(Clone, Debug)]
struct ContractBinding {
    reference: crate::ast::OrgContractReference,
    range: TextRange,
}

#[derive(Clone, Debug)]
struct ContractScopeInstance {
    kind: ContractScopeKind,
    title: Option<String>,
    outline_path: Vec<String>,
    range: TextRange,
}

impl ContractScopeInstance {
    fn document() -> Self {
        Self {
            kind: ContractScopeKind::Document,
            title: None,
            outline_path: Vec::new(),
            range: TextRange::new(0.into(), 0.into()),
        }
    }

    fn section(section: &Section<ParsedAnnotation>, outline_path: Vec<String>) -> Self {
        Self {
            kind: ContractScopeKind::Section,
            title: Some(section.raw_title.trim_end().to_string()),
            outline_path,
            range: section.ann.range,
        }
    }

    fn evaluation_scope(&self) -> OrgContractEvaluationScope {
        match self.kind {
            ContractScopeKind::Document => OrgContractEvaluationScope::document(),
            ContractScopeKind::Section => OrgContractEvaluationScope::section(
                self.title.clone().unwrap_or_default(),
                self.outline_path.clone(),
                self.range,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum ContractScopeKind {
    Document,
    Section,
}

impl ContractScopeKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Section => "section",
        }
    }
}
