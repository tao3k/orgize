//! `CONTRACT_ORG` semantic assertion linting.

use std::collections::{BTreeMap, BTreeSet};

use rowan::TextRange;

use crate::ast::{
    CONTRACT_ORG_PROPERTY, Keyword, OrgContract, OrgContractAssertion, OrgContractExpectation,
    OrgContractQuery, OrgContractRegistry, OrgContractRelativeScope, OrgContractScope,
    OrgElementGraph, OrgElementId, OrgElementQueryPredicate, OrgElementsIndexQuery,
    OrgElementsIndexSummaryPredicate, OrgElementsIndexSummaryTextPredicate,
    OrgElementsIndexSummaryValue, ParsedAnnotation, ParsedAst, Property, Section,
    parse_contract_reference,
};

use super::lint_model::{LintFinding, LintSeverity, location_for_range};

pub(crate) fn contract_org_findings(
    document: &ParsedAst,
    source: &str,
    registry: &OrgContractRegistry,
) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    let document_contract = document_contract_binding(document)
        .and_then(|binding| resolve_binding(binding, registry, source, &mut findings));
    let document_default_contract = match document_contract {
        Some(contract) if contract.scope == OrgContractScope::Document => {
            push_contract_findings(
                document,
                source,
                contract,
                ContractScopeInstance::document(),
                &mut findings,
            );
            None
        }
        Some(contract) if contract.scope == OrgContractScope::Subtree => Some(contract),
        _ => None,
    };

    for section in &document.sections {
        collect_section_contract_findings(
            document,
            source,
            registry,
            section,
            Vec::new(),
            document_default_contract,
            &mut findings,
        );
    }
    findings
}

fn collect_section_contract_findings<'a>(
    document: &ParsedAst,
    source: &str,
    registry: &'a OrgContractRegistry,
    section: &Section<ParsedAnnotation>,
    mut outline_path: Vec<String>,
    inherited_contract: Option<&'a OrgContract>,
    findings: &mut Vec<LintFinding>,
) {
    outline_path.push(section.raw_title.trim_end().to_string());

    let section_contract = match section_contract_binding(section) {
        Some(binding) => resolve_binding(binding, registry, source, findings)
            .filter(|contract| contract.scope == OrgContractScope::Subtree),
        None => inherited_contract,
    };

    if let Some(contract) = section_contract {
        push_contract_findings(
            document,
            source,
            contract,
            ContractScopeInstance::section(section, outline_path.clone()),
            findings,
        );
    }

    for child in &section.subsections {
        collect_section_contract_findings(
            document,
            source,
            registry,
            child,
            outline_path.clone(),
            None,
            findings,
        );
    }
}

fn push_contract_findings(
    document: &ParsedAst,
    source: &str,
    contract: &OrgContract,
    scope: ContractScopeInstance,
    findings: &mut Vec<LintFinding>,
) {
    let graph = document.org_elements_graph();
    for assertion in &contract.assertions {
        let actual = assertion_actual_count(&graph, assertion, &scope);
        if assertion.expectation.check(actual) {
            continue;
        }
        findings.push(LintFinding {
            code: "ORG044",
            severity: match assertion.severity {
                crate::ast::OrgContractSeverity::Error => LintSeverity::Error,
                crate::ast::OrgContractSeverity::Warning => LintSeverity::Warning,
            },
            message: assertion_message(AssertionMessageContext {
                contract_id: contract.id.as_str(),
                assertion_id: assertion.id.as_str(),
                template: assertion.message.as_deref(),
                fix_template: assertion.fix.as_deref(),
                scope: &scope,
                query: &assertion.query,
                expectation: &assertion.expectation,
                actual,
            }),
            location: location_for_range(source, scope.range),
        });
    }
}

fn assertion_actual_count(
    graph: &OrgElementGraph<ParsedAnnotation>,
    assertion: &OrgContractAssertion,
    scope: &ContractScopeInstance,
) -> usize {
    let mut bindings = BTreeMap::<String, BTreeSet<OrgElementId>>::new();
    for binding in &assertion.bindings {
        let query = scoped_contract_query(&binding.query, scope);
        bindings.insert(
            binding.name.clone(),
            query_graph_ids(graph, &query, &bindings),
        );
    }
    let query = scoped_contract_query(&assertion.query, scope);
    query_graph_ids(graph, &query, &bindings).len()
}

fn scoped_contract_query(
    query: &OrgContractQuery,
    scope: &ContractScopeInstance,
) -> OrgContractQuery {
    match scope.kind {
        ContractScopeKind::Document => query.clone(),
        ContractScopeKind::Section => query
            .clone()
            .apply_subtree_scope_prefix(scope.outline_path.clone()),
    }
}

fn query_graph_ids(
    graph: &OrgElementGraph<ParsedAnnotation>,
    query: &OrgContractQuery,
    bindings: &BTreeMap<String, BTreeSet<OrgElementId>>,
) -> BTreeSet<OrgElementId> {
    let Some(index_query) = index_query_with_relative_scope(query, bindings) else {
        return BTreeSet::new();
    };
    graph
        .query(&index_query)
        .iter()
        .map(|record| record.id)
        .collect()
}

fn index_query_with_relative_scope(
    query: &OrgContractQuery,
    bindings: &BTreeMap<String, BTreeSet<OrgElementId>>,
) -> Option<OrgElementsIndexQuery> {
    let index_query = query.to_index_query();
    match &query.relative_to {
        None => Some(index_query),
        Some(OrgContractRelativeScope::DescendantOfBinding(binding)) => {
            let roots = bindings.get(binding)?;
            (!roots.is_empty()).then(|| index_query.descendant_of_any(roots.iter().copied()))
        }
        Some(OrgContractRelativeScope::ChildOfBinding(binding)) => {
            let roots = bindings.get(binding)?;
            (!roots.is_empty()).then(|| index_query.child_of_any(roots.iter().copied()))
        }
        Some(OrgContractRelativeScope::AtBinding(binding)) => {
            let roots = bindings.get(binding)?;
            (!roots.is_empty()).then(|| index_query.at_any(roots.iter().copied()))
        }
    }
}

struct AssertionMessageContext<'a> {
    contract_id: &'a str,
    assertion_id: &'a str,
    template: Option<&'a str>,
    fix_template: Option<&'a str>,
    scope: &'a ContractScopeInstance,
    query: &'a OrgContractQuery,
    expectation: &'a OrgContractExpectation,
    actual: usize,
}

fn assertion_message(context: AssertionMessageContext<'_>) -> String {
    let rendered = context
        .template
        .map(|template| {
            render_contract_template(template, context.scope, context.actual, context.expectation)
        })
        .unwrap_or_default();
    let predicate_detail = boolean_query_summary(context.query)
        .map(|summary| format!("; predicate {summary}"))
        .unwrap_or_default();
    let detail = format!(
        "contract `{contract_id}` assertion `{assertion_id}` failed in {} `{}`: expected {}, actual {actual}{predicate_detail}",
        context.scope.kind.as_str(),
        context.scope.title.as_deref().unwrap_or("<document>"),
        context.expectation.expected_summary(),
        contract_id = context.contract_id,
        assertion_id = context.assertion_id,
        actual = context.actual,
    );
    if rendered.trim().is_empty() {
        append_rendered_fix(
            detail,
            context.fix_template,
            context.scope,
            context.actual,
            context.expectation,
        )
    } else {
        append_rendered_fix(
            format!("{} ({detail})", rendered.trim()),
            context.fix_template,
            context.scope,
            context.actual,
            context.expectation,
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
    actual: usize,
    expectation: &OrgContractExpectation,
) -> String {
    let rendered_fix = fix_template
        .map(|template| render_contract_template(template, scope, actual, expectation))
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
    actual: usize,
    expectation: &OrgContractExpectation,
) -> String {
    template
        .replace("{{ scope.title }}", scope.title.as_deref().unwrap_or(""))
        .replace("{{scope.title}}", scope.title.as_deref().unwrap_or(""))
        .replace("{{ scope.kind }}", scope.kind.as_str())
        .replace("{{scope.kind}}", scope.kind.as_str())
        .replace("{{ result.count }}", &actual.to_string())
        .replace("{{result.count}}", &actual.to_string())
        .replace("{{ expected }}", &expectation.expected_summary())
        .replace("{{expected}}", &expectation.expected_summary())
}

fn resolve_binding<'a>(
    binding: ContractBinding,
    registry: &'a OrgContractRegistry,
    source: &str,
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
    let Some(contract) = registry.resolve(&binding.reference) else {
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

fn document_contract_binding(document: &ParsedAst) -> Option<ContractBinding> {
    property_contract_binding(&document.properties)
        .or_else(|| keyword_contract_binding(&document.metadata))
}

fn section_contract_binding(section: &Section<ParsedAnnotation>) -> Option<ContractBinding> {
    property_contract_binding(&section.properties)
}

fn property_contract_binding(properties: &[Property<ParsedAnnotation>]) -> Option<ContractBinding> {
    properties
        .iter()
        .rev()
        .find(|property| property.key.eq_ignore_ascii_case(CONTRACT_ORG_PROPERTY))
        .map(|property| ContractBinding {
            reference: parse_contract_reference(property.value.as_str()),
            range: property.ann.range,
        })
}

fn keyword_contract_binding(keywords: &[Keyword<ParsedAnnotation>]) -> Option<ContractBinding> {
    keywords
        .iter()
        .rev()
        .find(|keyword| keyword.key.eq_ignore_ascii_case(CONTRACT_ORG_PROPERTY))
        .map(|keyword| ContractBinding {
            reference: parse_contract_reference(keyword.value.as_str()),
            range: keyword.ann.range,
        })
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
