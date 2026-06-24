//! Contract evaluation facts for `CONTRACT_ORG`.

use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    path::{Path, PathBuf},
    process::Command,
};

use super::{
    OrgContract, OrgContractAssertion, OrgContractAssertionEvaluation, OrgContractAssertionStatus,
    OrgContractDocumentPredicate, OrgContractEvaluation, OrgContractEvaluationContext,
    OrgContractEvaluationScope, OrgContractQuery, OrgContractRelativeScope, OrgElementGraph,
    OrgElementId, OrgElementsIndexQuery, ParsedAnnotation, ParsedAst, Property, Section,
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
    let scoped_context = context_with_effective_dir(document, &scope, context);
    let assertions = contract
        .assertions
        .iter()
        .map(|assertion| evaluate_assertion(&graph, assertion, &scope, &scoped_context))
        .collect();
    OrgContractEvaluation {
        contract_id: contract.id.clone(),
        scope,
        assertions,
    }
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
    let bindings = binding_sets
        .into_iter()
        .map(|(name, ids)| (name, ids.into_iter().collect()))
        .collect();
    OrgContractAssertionEvaluation {
        assertion_id: assertion.id.clone(),
        severity: assertion.severity,
        expectation: assertion.expectation.clone(),
        actual_count,
        status: if assertion.expectation.check(actual_count) {
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

fn context_with_effective_dir(
    document: &ParsedAst,
    scope: &OrgContractEvaluationScope,
    context: &OrgContractEvaluationContext,
) -> OrgContractEvaluationContext {
    let mut scoped = context.clone();
    if scoped.dir_scope().is_none() {
        if let Some(dir) = effective_dir_for_scope(document, scope) {
            scoped = scoped.with_dir_scope(expand_dir_path(&dir, document));
        }
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
    let expanded_macros = expand_property_macros(value.trim(), document);
    let expanded_commands = expand_command_substitutions(&expanded_macros);
    PathBuf::from(expand_environment_tokens(&expanded_commands))
}

fn expand_property_macros(value: &str, document: &ParsedAst) -> String {
    let mut expanded = String::new();
    let mut index = 0;
    while index < value.len() {
        let rest = &value[index..];
        if let Some((name, arguments, consumed, original)) = org_macro_token(rest) {
            if let Some(template) = document
                .macro_definitions
                .iter()
                .rev()
                .find(|definition| definition.name == name)
                .map(|definition| definition.template.as_str())
            {
                expanded.push_str(&expand_macro_template(template, &arguments));
            } else {
                expanded.push_str(original);
            }
            index += consumed;
        } else {
            let ch = rest
                .chars()
                .next()
                .expect("non-empty slice has a first char");
            expanded.push(ch);
            index += ch.len_utf8();
        }
    }
    expanded
}

fn expand_environment_tokens(value: &str) -> String {
    let mut expanded = String::new();
    let mut index = 0;
    while index < value.len() {
        let rest = &value[index..];
        if let Some((token, consumed, original)) = dollar_path_token(rest) {
            if let Some(resolved) = resolve_path_token(token) {
                expanded.push_str(&resolved);
            } else {
                expanded.push_str(original);
            }
            index += consumed;
        } else {
            let ch = rest
                .chars()
                .next()
                .expect("non-empty slice has a first char");
            expanded.push(ch);
            index += ch.len_utf8();
        }
    }
    expanded
}

fn expand_command_substitutions(value: &str) -> String {
    let mut expanded = String::new();
    let mut index = 0;
    while index < value.len() {
        let rest = &value[index..];
        if let Some((command, consumed, original)) = command_substitution_token(rest) {
            if let Some(output) = run_command_substitution(command) {
                expanded.push_str(&output);
            } else {
                expanded.push_str(original);
            }
            index += consumed;
        } else {
            let ch = rest
                .chars()
                .next()
                .expect("non-empty slice has a first char");
            expanded.push(ch);
            index += ch.len_utf8();
        }
    }
    expanded
}

fn command_substitution_token(input: &str) -> Option<(&str, usize, &str)> {
    let rest = input.strip_prefix("$(")?;
    let mut depth = 1usize;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    for (index, ch) in rest.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && !in_single_quote {
            escaped = true;
            continue;
        }
        if ch == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            continue;
        }
        if ch == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            continue;
        }
        if in_single_quote || in_double_quote {
            continue;
        }
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let consumed = 2 + index + ch.len_utf8();
                    return Some((&rest[..index], consumed, &input[..consumed]));
                }
            }
            _ => {}
        }
    }
    None
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

fn org_macro_token(input: &str) -> Option<(&str, Vec<String>, usize, &str)> {
    let rest = input.strip_prefix("{{{")?;
    let end = rest.find("}}}")?;
    let body = &rest[..end];
    let consumed = 3 + end + 3;
    let (name, arguments) = parse_macro_body(body);
    Some((name, arguments, consumed, &input[..consumed]))
}

fn parse_macro_body(body: &str) -> (&str, Vec<String>) {
    let Some(open) = body.find('(') else {
        return (body.trim(), Vec::new());
    };
    if !body.ends_with(')') {
        return (body.trim(), Vec::new());
    }
    let name = body[..open].trim();
    let args = body[open + 1..body.len() - 1]
        .split(',')
        .map(str::trim)
        .filter(|arg| !arg.is_empty())
        .map(ToString::to_string)
        .collect();
    (name, args)
}

fn expand_macro_template(template: &str, arguments: &[String]) -> String {
    let mut expanded =
        String::with_capacity(template.len() + arguments.iter().map(String::len).sum::<usize>());
    let mut all_arguments = None;
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '$' {
            expanded.push(ch);
            continue;
        }

        match chars.peek().copied() {
            Some('$') => {
                chars.next();
                expanded.push('$');
            }
            Some('0') => {
                chars.next();
                expanded.push_str(all_arguments.get_or_insert_with(|| arguments.join(", ")));
            }
            Some(digit) if digit.is_ascii_digit() => {
                chars.next();
                let index = digit
                    .to_digit(10)
                    .expect("ASCII digit must convert to a number")
                    .saturating_sub(1) as usize;
                if let Some(argument) = arguments.get(index) {
                    expanded.push_str(argument);
                }
            }
            _ => expanded.push('$'),
        }
    }

    expanded
}

fn dollar_path_token(input: &str) -> Option<(&str, usize, &str)> {
    let rest = input.strip_prefix('$')?;
    if let Some(rest) = rest.strip_prefix('{') {
        let end = rest.find('}')?;
        let consumed = 2 + end + 1;
        return Some((&rest[..end], consumed, &input[..consumed]));
    }
    let token_len = rest
        .char_indices()
        .take_while(|(_, ch)| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
        .map(|(index, ch)| index + ch.len_utf8())
        .last()?;
    Some((&rest[..token_len], token_len + 1, &input[..token_len + 1]))
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
