//! Contract evaluation facts for `CONTRACT_ORG`.

use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    path::{Path, PathBuf},
    process::Command,
};

use super::{
    AstRef, OrgContract, OrgContractAssertion, OrgContractAssertionEvaluation,
    OrgContractAssertionStatus, OrgContractDocumentPredicate, OrgContractEvaluation,
    OrgContractEvaluationContext, OrgContractEvaluationScope, OrgContractQuery,
    OrgContractRelativeScope, OrgContractValueField, OrgElementGraph, OrgElementId,
    OrgElementsIndexQuery, OrgElementsIndexSummaryValue, ParsedAnnotation, ParsedAst, Property,
    Section,
};

const DIR_PROPERTY: &str = "DIR";

/// Evaluates a resolved Org contract over a source-backed document scope.
pub fn evaluate_org_contract(
    document: &ParsedAst,
    contract: &OrgContract,
    scope: OrgContractEvaluationScope,
) -> OrgContractEvaluation {
    evaluate_org_contract_with_context(
        document,
        contract,
        scope,
        &OrgContractEvaluationContext::default(),
    )
}

/// Evaluates a resolved Org contract with host-owned document context.
pub fn evaluate_org_contract_with_context(
    document: &ParsedAst,
    contract: &OrgContract,
    scope: OrgContractEvaluationScope,
    context: &OrgContractEvaluationContext,
) -> OrgContractEvaluation {
    let graph = document.org_elements_graph();
    evaluate_org_contract_with_graph_context(document, &graph, contract, scope, context)
}

pub(crate) fn evaluate_org_contract_with_graph_context(
    document: &ParsedAst,
    graph: &OrgElementGraph<ParsedAnnotation>,
    contract: &OrgContract,
    scope: OrgContractEvaluationScope,
    context: &OrgContractEvaluationContext,
) -> OrgContractEvaluation {
    let mut document_context = context.clone();
    document_context.metadata_keys = document_keyword_keys(document);
    let scoped_context = context_with_effective_dir(document, &scope, &document_context);
    let assertions = contract
        .assertions
        .iter()
        .map(|assertion| evaluate_assertion(graph, assertion, &scope, &scoped_context))
        .collect();
    OrgContractEvaluation {
        contract_id: contract.id.clone(),
        scope,
        assertions,
    }
}

fn document_keyword_keys(document: &ParsedAst) -> Vec<String> {
    let mut keys = BTreeSet::new();
    document.visit(|node| {
        if let AstRef::Keyword(keyword) = node {
            keys.insert(keyword.key.to_ascii_uppercase());
        }
    });
    keys.into_iter().collect()
}

fn evaluate_assertion(
    graph: &OrgElementGraph<ParsedAnnotation>,
    assertion: &OrgContractAssertion,
    scope: &OrgContractEvaluationScope,
    context: &OrgContractEvaluationContext,
) -> OrgContractAssertionEvaluation {
    let mut binding_sets = BTreeMap::<String, BTreeSet<OrgElementId>>::new();
    for binding in &assertion.bindings {
        let query = scoped_contract_query(&binding.query, scope);
        binding_sets.insert(
            binding.name.clone(),
            query_graph_ids(graph, &query, &binding_sets, context),
        );
    }
    let query = scoped_contract_query(&assertion.query, scope);
    let matched = query_graph_ids(graph, &query, &binding_sets, context);
    let actual_count = matched.len();
    let binding_counts = binding_sets
        .iter()
        .map(|(name, ids)| (name.clone(), ids.len()))
        .collect::<BTreeMap<_, _>>();
    let passed = match &assertion.expectation {
        super::OrgContractExpectation::ValueSetEqual {
            binding,
            source_field,
            target_field,
        } => binding_sets.get(binding).is_some_and(|source_ids| {
            projected_values(graph, source_ids, source_field)
                == projected_values(graph, &matched, target_field)
        }),
        expectation => expectation.check(actual_count, &binding_counts),
    };
    let bindings = binding_sets
        .into_iter()
        .map(|(name, ids)| (name, ids.into_iter().collect()))
        .collect();
    OrgContractAssertionEvaluation {
        assertion_id: assertion.id.clone(),
        severity: assertion.severity,
        expectation: assertion.expectation.clone(),
        actual_count,
        status: if passed {
            OrgContractAssertionStatus::Passed
        } else {
            OrgContractAssertionStatus::Failed
        },
        matched_ids: matched.into_iter().collect(),
        bindings,
        message_template: assertion.message.clone(),
        fix_template: assertion.fix.clone(),
    }
}

fn projected_values(
    graph: &OrgElementGraph<ParsedAnnotation>,
    ids: &BTreeSet<OrgElementId>,
    field: &OrgContractValueField,
) -> BTreeSet<String> {
    ids.iter()
        .filter_map(|id| {
            graph
                .by_id
                .get(id)
                .and_then(|index| graph.records.get(*index))
        })
        .filter_map(|record| match field {
            OrgContractValueField::Property(key) => record
                .properties
                .get(key)
                .or_else(|| {
                    key.strip_prefix(':')
                        .and_then(|key| record.properties.get(key))
                })
                .or_else(|| {
                    (!key.starts_with(':'))
                        .then(|| record.properties.get(&format!(":{key}")))
                        .flatten()
                }),
            OrgContractValueField::Summary(key) => record.summary.get(key),
        })
        .flat_map(summary_value_strings)
        .collect()
}

fn summary_value_strings(value: &OrgElementsIndexSummaryValue) -> Vec<String> {
    match value {
        OrgElementsIndexSummaryValue::Null => Vec::new(),
        OrgElementsIndexSummaryValue::Text(value) => vec![value.clone()],
        OrgElementsIndexSummaryValue::StringList(values) => values.clone(),
        OrgElementsIndexSummaryValue::Bool(value) => vec![value.to_string()],
        OrgElementsIndexSummaryValue::Integer(value) => vec![value.to_string()],
    }
}

fn scoped_contract_query(
    query: &OrgContractQuery,
    scope: &OrgContractEvaluationScope,
) -> OrgContractQuery {
    match scope {
        OrgContractEvaluationScope::Document { .. } => query.clone(),
        OrgContractEvaluationScope::Section { outline_path, .. } => query
            .clone()
            .apply_subtree_scope_prefix(outline_path.clone()),
    }
}

fn query_graph_ids(
    graph: &OrgElementGraph<ParsedAnnotation>,
    query: &OrgContractQuery,
    bindings: &BTreeMap<String, BTreeSet<OrgElementId>>,
    context: &OrgContractEvaluationContext,
) -> BTreeSet<OrgElementId> {
    if !dir_scope_matches_source_path(context) {
        return BTreeSet::new();
    }
    if !document_predicates_match(&query.document_predicates, context) {
        return BTreeSet::new();
    }
    if !query.alternatives.is_empty() {
        let mut ids = BTreeSet::new();
        let base_query = query_without_alternatives(query);
        for alternative in &query.alternatives {
            ids.extend(query_graph_ids(
                graph,
                &combine_contract_queries(base_query.clone(), alternative),
                bindings,
                context,
            ));
        }
        return ids;
    }
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

fn query_without_alternatives(query: &OrgContractQuery) -> OrgContractQuery {
    let mut query = query.clone();
    query.alternatives.clear();
    query
}

fn combine_contract_queries(
    mut target: OrgContractQuery,
    source: &OrgContractQuery,
) -> OrgContractQuery {
    target.alternatives.extend(source.alternatives.clone());
    if source.category.is_some() {
        target.category = source.category;
    }
    if source.kind.is_some() {
        target.kind = source.kind.clone();
    }
    if source.affiliated_name.is_some() {
        target.affiliated_name = source.affiliated_name.clone();
    }
    if source.context.is_some() {
        target.context = source.context.clone();
    }
    if !source.outline_path_prefix.is_empty() {
        target.outline_path_prefix = source.outline_path_prefix.clone();
    }
    if source.outline_path_exact_len.is_some() {
        target.outline_path_exact_len = source.outline_path_exact_len;
    }
    target
        .property_equals
        .extend(source.property_equals.clone());
    target
        .property_contains
        .extend(source.property_contains.clone());
    target.summary_equals.extend(source.summary_equals.clone());
    target
        .summary_contains
        .extend(source.summary_contains.clone());
    target.predicates.extend(source.predicates.clone());
    target
        .document_predicates
        .extend(source.document_predicates.clone());
    if source.limit.is_some() {
        target.limit = source.limit;
    }
    target.use_scope_outline_path |= source.use_scope_outline_path;
    target.has_outline_path_prefix |= source.has_outline_path_prefix;
    if source.scope_outline_depth.is_some() {
        target.scope_outline_depth = source.scope_outline_depth;
    }
    if source.relative_to.is_some() {
        target.relative_to = source.relative_to.clone();
    }
    target
}

fn context_with_effective_dir(
    document: &ParsedAst,
    scope: &OrgContractEvaluationScope,
    context: &OrgContractEvaluationContext,
) -> OrgContractEvaluationContext {
    let mut scoped = context.clone();
    if scoped.dir_scope().is_none()
        && let Some(dir) = effective_dir_for_scope(document, scope)
    {
        scoped = scoped.with_dir_scope(expand_dir_path(&dir, document));
    }
    scoped
}

fn effective_dir_for_scope(
    document: &ParsedAst,
    scope: &OrgContractEvaluationScope,
) -> Option<String> {
    match scope {
        OrgContractEvaluationScope::Document { .. } => {
            property_value(&document.properties, DIR_PROPERTY)
        }
        OrgContractEvaluationScope::Section { outline_path, .. } => {
            find_section_by_outline_path(&document.sections, outline_path)
                .and_then(|section| property_value(&section.effective_properties, DIR_PROPERTY))
                .or_else(|| property_value(&document.properties, DIR_PROPERTY))
        }
    }
}

fn find_section_by_outline_path<'a>(
    sections: &'a [Section<ParsedAnnotation>],
    outline_path: &[String],
) -> Option<&'a Section<ParsedAnnotation>> {
    let (head, tail) = outline_path.split_first()?;
    let section = sections.iter().find(|section| section.raw_title == *head)?;
    if tail.is_empty() {
        Some(section)
    } else {
        find_section_by_outline_path(&section.subsections, tail)
    }
}

fn property_value(properties: &[Property<ParsedAnnotation>], key: &str) -> Option<String> {
    properties
        .iter()
        .rev()
        .find(|property| property.key.eq_ignore_ascii_case(key))
        .map(|property| property.value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn dir_scope_matches_source_path(context: &OrgContractEvaluationContext) -> bool {
    let Some(dir_scope) = context.dir_scope() else {
        return true;
    };
    let Some(source_path) = context.source_path() else {
        return false;
    };
    let dir_scope = absolute_dir_scope(dir_scope, source_path);
    source_path.starts_with(dir_scope)
}

fn absolute_dir_scope(dir_scope: &Path, source_path: &Path) -> PathBuf {
    if dir_scope.is_absolute() {
        return dir_scope.to_path_buf();
    }
    source_path
        .parent()
        .map(|parent| parent.join(dir_scope))
        .unwrap_or_else(|| dir_scope.to_path_buf())
}

fn expand_dir_path(value: &str, document: &ParsedAst) -> PathBuf {
    // Contract DIR path syntax:
    //
    //   dir-path = literal *( org-macro / command-substitution / env-token )
    //   command-substitution = "$(" host-command ")"
    //
    // The Org parser keeps DIR as ordinary property text. Contract evaluation
    // is the boundary that turns it into an effective path scope.
    let expanded_macros = expand_property_macros(value, document);
    let expanded_commands = expand_command_substitutions(&expanded_macros);
    PathBuf::from(expand_environment_tokens(&expanded_commands))
}

fn expand_property_macros(value: &str, document: &ParsedAst) -> String {
    let fields = std::iter::once(value)
        .chain(
            document
                .macro_definitions
                .iter()
                .flat_map(|definition| [definition.name.as_str(), definition.template.as_str()]),
        )
        .collect::<Vec<_>>();
    crate::org_aot::expand_native_macro_fields(15, &fields)
        .expect("initialized native property macro operation")
}

fn expand_environment_tokens(value: &str) -> String {
    execute_dir_plan(value, 14, resolve_path_token)
}
fn expand_command_substitutions(value: &str) -> String {
    execute_dir_plan(value, 13, run_command_substitution)
}
fn execute_dir_plan(value: &str, operation: u8, resolve: fn(&str) -> Option<String>) -> String {
    let rows = crate::org_aot::native_semantic_rows(operation, &[value])
        .expect("initialized native DIR plan operation");
    let mut expanded = String::new();
    for row in rows {
        let [kind, value, original]: [String; 3] =
            row.try_into().expect("native DIR plan row arity");
        match kind.as_str() {
            "literal" => expanded.push_str(&value),
            "command" if operation == 13 => {
                expanded.push_str(resolve(&value).as_deref().unwrap_or(&original));
            }
            "environment" if operation == 14 => {
                expanded.push_str(resolve(&value).as_deref().unwrap_or(&original));
            }
            _ => panic!("native DIR plan action"),
        }
    }
    expanded
}

fn run_command_substitution(command: &str) -> Option<String> {
    let command = command.trim();
    if command.is_empty() {
        return None;
    }

    let output = Command::new("sh").arg("-c").arg(command).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let mut stdout = String::from_utf8(output.stdout).ok()?;
    while stdout.ends_with('\n') || stdout.ends_with('\r') {
        stdout.pop();
    }
    Some(stdout)
}

fn resolve_path_token(token: &str) -> Option<String> {
    env::var(token).ok()
}

fn document_predicates_match(
    predicates: &[OrgContractDocumentPredicate],
    context: &OrgContractEvaluationContext,
) -> bool {
    predicates
        .iter()
        .all(|predicate| predicate.matches_context(context))
}
