//! Org contract parser for `CONTRACT_ORG` validation.

use std::{collections::HashSet, path::Path};

use super::{
    ASSERT_ID_PROPERTY, ASSERT_SEVERITY_PROPERTY, CONTRACT_ALIAS_PROPERTY, CONTRACT_ID_PROPERTY,
    CONTRACT_KIND_ORG_ELEMENTS, CONTRACT_KIND_PROPERTY, CONTRACT_SCOPE_PROPERTY, Document,
    OrgContract, OrgContractAssertion, OrgContractBinding, OrgContractCompareOp,
    OrgContractExpectation, OrgContractKind, OrgContractQuery, OrgContractReference,
    OrgContractRegistry, OrgContractRelativeScope, OrgContractScope, OrgContractSeverity,
    OrgElementQueryPredicate, OrgElementSelector, OrgElementsIndexCategory, OrgElementsIndexKind,
    ParsedAnnotation, Property, Section, SourceBlockRecord, SourceBlockRecordKind,
};

/// Parses a host-loaded Org contract registry from an Org document.
pub fn parse_contracts_from_document(
    document: &Document<ParsedAnnotation>,
    source_path: Option<&Path>,
) -> OrgContractRegistry {
    let source_blocks = document.source_block_records();
    let mut contracts = Vec::new();
    for section in &document.sections {
        collect_contract_sections(section, &source_blocks, source_path, &mut contracts);
    }
    OrgContractRegistry::new(contracts)
}

/// Parses a `CONTRACT_ORG` property/keyword value.
pub fn parse_contract_reference(value: &str) -> OrgContractReference {
    let raw = value.trim().to_string();
    if raw.is_empty() {
        return OrgContractReference {
            raw,
            path: None,
            contract_id: None,
        };
    }

    if let Some(reference) = org_link_contract_reference(&raw) {
        return reference;
    }

    let normalized = macro_reference_argument(&raw).unwrap_or_else(|| raw.clone());

    let without_file = normalized
        .strip_prefix("file:")
        .unwrap_or(&normalized)
        .to_string();
    if let Some((path, contract_id)) = without_file.split_once('#') {
        return OrgContractReference {
            raw,
            path: contract_reference_path(path),
            contract_id: (!contract_id.trim().is_empty()).then(|| contract_id.trim().to_string()),
        };
    }

    if looks_like_file_reference(&without_file) {
        return OrgContractReference {
            raw,
            path: Some(without_file),
            contract_id: None,
        };
    }

    OrgContractReference {
        raw,
        path: None,
        contract_id: Some(normalized),
    }
}

fn collect_contract_sections(
    section: &Section<ParsedAnnotation>,
    source_blocks: &[SourceBlockRecord],
    source_path: Option<&Path>,
    contracts: &mut Vec<OrgContract>,
) {
    if let Some(contract) = parse_contract_section(section, source_blocks, source_path) {
        contracts.push(contract);
    }
    for child in &section.subsections {
        collect_contract_sections(child, source_blocks, source_path, contracts);
    }
}

fn parse_contract_section(
    section: &Section<ParsedAnnotation>,
    source_blocks: &[SourceBlockRecord],
    source_path: Option<&Path>,
) -> Option<OrgContract> {
    let id = section_property_value(&section.properties, CONTRACT_ID_PROPERTY)?;
    if id.trim().is_empty() {
        return None;
    }

    let kind = section_property_value(&section.properties, CONTRACT_KIND_PROPERTY)
        .unwrap_or_else(|| CONTRACT_KIND_ORG_ELEMENTS.to_string());
    let kind = OrgContractKind::parse(&kind)?;

    let aliases = contract_aliases(
        source_path,
        id.as_str(),
        section_property_value(&section.properties, CONTRACT_ALIAS_PROPERTY),
    );
    let scope = section_property_value(&section.properties, CONTRACT_SCOPE_PROPERTY)
        .and_then(|value| OrgContractScope::parse(&value))
        .unwrap_or_default();

    let mut assertions = Vec::new();
    for child in &section.subsections {
        collect_assertions(child, source_blocks, &mut assertions);
    }

    Some(OrgContract {
        id,
        aliases,
        scope,
        kind,
        assertions,
    })
}

fn collect_assertions(
    section: &Section<ParsedAnnotation>,
    source_blocks: &[SourceBlockRecord],
    assertions: &mut Vec<OrgContractAssertion>,
) {
    if let Some(assertion) = parse_assertion(section, source_blocks) {
        assertions.push(assertion);
        return;
    }
    for child in &section.subsections {
        collect_assertions(child, source_blocks, assertions);
    }
}

fn parse_assertion(
    section: &Section<ParsedAnnotation>,
    source_blocks: &[SourceBlockRecord],
) -> Option<OrgContractAssertion> {
    let id = section_property_value(&section.properties, ASSERT_ID_PROPERTY)?;
    if id.trim().is_empty() {
        return None;
    }

    let severity = section_property_value(&section.properties, ASSERT_SEVERITY_PROPERTY)
        .and_then(|value| parse_severity(&value))
        .unwrap_or_default();

    let mut query = None;
    let mut bindings = Vec::new();
    let mut expectation = None;
    let mut message = None;
    let mut fix = None;
    let mut query_source = None;
    let mut expect_source = None;

    for block in section_source_blocks(section, source_blocks) {
        let language = block.language.as_deref().unwrap_or_default().trim();
        if language.eq_ignore_ascii_case("org-elements-query") {
            query = Some(parse_query_block(&block.value));
            query_source = Some(block.source.clone());
        } else if language.eq_ignore_ascii_case("org-elements-selector") {
            query = parse_selector_block(&block.value);
            query_source = Some(block.source.clone());
        } else if language.eq_ignore_ascii_case("org-contract") {
            if let Some((parsed_bindings, parsed_query, parsed_expectation)) =
                parse_org_contract_block(&block.value)
            {
                bindings = parsed_bindings;
                query = Some(parsed_query);
                expectation = Some(parsed_expectation);
                query_source = Some(block.source.clone());
                expect_source = Some(block.source.clone());
            }
        } else if language.eq_ignore_ascii_case("org-elements-expect") {
            expectation = parse_expectation(&block.value);
            expect_source = Some(block.source.clone());
        } else if language.eq_ignore_ascii_case("jinja2") {
            match block_parameter_name(block).as_deref() {
                Some("message") => message = Some(block.value.clone()),
                Some("fix") => fix = Some(block.value.clone()),
                _ => {}
            }
        }
    }

    Some(OrgContractAssertion {
        id,
        severity,
        bindings,
        query: query?,
        expectation: expectation.unwrap_or(OrgContractExpectation::Exists),
        message,
        fix,
        query_source,
        expect_source,
    })
}

fn section_source_blocks<'a>(
    section: &Section<ParsedAnnotation>,
    source_blocks: &'a [SourceBlockRecord],
) -> Vec<&'a SourceBlockRecord> {
    let start = usize::from(section.ann.range.start());
    let end = usize::from(section.ann.range.end());
    source_blocks
        .iter()
        .filter(|record| {
            matches!(record.kind, SourceBlockRecordKind::Block)
                && (record.source.range_start as usize) >= start
                && (record.source.range_end as usize) <= end
        })
        .collect()
}

fn block_parameter_name(block: &SourceBlockRecord) -> Option<String> {
    let parameters = block.parameters.as_deref()?;
    let mut parts = parameters.split_whitespace();
    while let Some(part) = parts.next() {
        if part == ":name" {
            return parts.next().map(str::to_string);
        }
    }
    None
}

fn parse_query_block(value: &str) -> OrgContractQuery {
    let mut query = OrgContractQuery::default();
    for raw_line in value.lines() {
        let line = strip_query_comment(raw_line);
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("outline_path starts_with") {
            let rhs = rest.trim();
            if rhs == "$scope.outline_path" {
                query.use_scope_outline_path = true;
            } else if let Some(value) = query_value(rhs) {
                query.outline_path_prefix = outline_path_value(&value);
                query.has_outline_path_prefix = true;
            }
            continue;
        }

        if let Some((lhs, rhs)) = split_line(&line, " contains ") {
            if let Some(key) = lhs.strip_prefix("summary.") {
                if let Some(value) = query_value(rhs) {
                    query.summary_contains.push((key.trim().to_string(), value));
                }
            } else if let Some(key) = lhs.strip_prefix("property.")
                && let Some(value) = query_value(rhs)
            {
                query
                    .property_contains
                    .push((key.trim().to_string(), value));
            }
            continue;
        }

        let Some((lhs, rhs)) = split_line(&line, "=") else {
            continue;
        };
        match lhs {
            "category" => {
                query.category =
                    query_value(rhs).and_then(|value| OrgElementsIndexCategory::from_label(&value));
            }
            "kind" => query.kind = query_value(rhs).map(OrgElementsIndexKind::new),
            "affiliated_name" => query.affiliated_name = query_value(rhs),
            "context" => query.context = query_value(rhs),
            "limit" => query.limit = query_value(rhs).and_then(|value| value.parse().ok()),
            "within" if rhs == "\"$scope\"" || rhs == "$scope" => {
                query.use_scope_outline_path = true;
            }
            "outline_path_prefix" => {
                if let Some(value) = query_value(rhs) {
                    if value == "$scope.outline_path" {
                        query.use_scope_outline_path = true;
                    } else {
                        query.outline_path_prefix = outline_path_value(&value);
                        query.has_outline_path_prefix = true;
                    }
                }
            }
            key if key.starts_with("summary.") => {
                if let Some(value) = query_value(rhs) {
                    query
                        .summary_equals
                        .push((key.trim_start_matches("summary.").to_string(), value));
                }
            }
            key if key.starts_with("property.") => {
                if let Some(value) = query_value(rhs) {
                    query
                        .property_equals
                        .push((key.trim_start_matches("property.").to_string(), value));
                }
            }
            _ => {}
        }
    }
    query
}

fn parse_org_contract_block(
    value: &str,
) -> Option<(
    Vec<OrgContractBinding>,
    OrgContractQuery,
    OrgContractExpectation,
)> {
    let mut lines = value
        .lines()
        .map(strip_query_comment)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .into_iter()
        .peekable();
    let mut bindings = Vec::new();
    while let Some(line) = lines.peek() {
        let Some(binding) = parse_org_contract_binding(line) else {
            break;
        };
        bindings.push(binding);
        lines.next();
    }
    let assert_line = lines.next()?;
    let assert_rest = assert_line.strip_prefix("assert")?.trim();
    let (expectation, rest) = if let Some(rest) = assert_rest.strip_prefix("not exists") {
        (OrgContractExpectation::NotExists, rest.trim())
    } else if let Some(rest) = assert_rest.strip_prefix("exists") {
        (OrgContractExpectation::Exists, rest.trim())
    } else if let Some(rest) = assert_rest.strip_prefix("count") {
        (
            OrgContractExpectation::Count(OrgContractCompareOp::Ge, 1),
            rest.trim(),
        )
    } else {
        (OrgContractExpectation::Exists, assert_rest)
    };

    let (kind, inline_condition) = if let Some((kind, condition)) = rest.split_once(" where ") {
        (kind.trim(), Some(condition.trim()))
    } else if let Some(kind) = rest.strip_suffix(" where") {
        (kind.trim(), None)
    } else {
        (rest.trim(), None)
    };
    if kind.is_empty() {
        return None;
    }

    let mut query = OrgContractQuery::default();
    apply_org_contract_kind(kind, &mut query);
    let mut expectation = expectation;
    if let Some(condition) = inline_condition {
        parse_org_contract_conditions(condition, &mut query, &mut expectation);
    }
    for line in lines {
        parse_org_contract_conditions(&line, &mut query, &mut expectation);
    }
    Some((bindings, query, expectation))
}

fn parse_org_contract_binding(line: &str) -> Option<OrgContractBinding> {
    let rest = line.strip_prefix("let ")?.trim();
    let (name, query_source) = rest.split_once('=')?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let query = parse_org_contract_query_source(query_source.trim())?;
    Some(OrgContractBinding {
        name: name.to_string(),
        query,
    })
}

fn parse_org_contract_query_source(value: &str) -> Option<OrgContractQuery> {
    let (kind, inline_condition) = if let Some((kind, condition)) = value.split_once(" where ") {
        (kind.trim(), Some(condition.trim()))
    } else if let Some(kind) = value.strip_suffix(" where") {
        (kind.trim(), None)
    } else {
        (value.trim(), None)
    };
    if kind.is_empty() {
        return None;
    }
    let mut query = OrgContractQuery::default();
    apply_org_contract_kind(kind, &mut query);
    if let Some(condition) = inline_condition {
        parse_org_contract_conditions(condition, &mut query, &mut OrgContractExpectation::Exists);
    }
    Some(query)
}

fn parse_org_contract_conditions(
    conditions: &str,
    query: &mut OrgContractQuery,
    expectation: &mut OrgContractExpectation,
) {
    for condition in conditions.split(" and ") {
        parse_org_contract_condition(condition, query, expectation);
    }
}

fn parse_org_contract_condition(
    condition: &str,
    query: &mut OrgContractQuery,
    expectation: &mut OrgContractExpectation,
) {
    let condition = condition
        .trim()
        .strip_prefix("and ")
        .unwrap_or(condition.trim())
        .trim();
    if condition.is_empty() || condition == "where" {
        return;
    }
    if let Some(parsed_expectation) = parse_count_comparison(condition) {
        *expectation = parsed_expectation;
        return;
    }
    if parse_org_contract_relation_condition(condition, query) {
        return;
    }
    if let Some(predicate) = parse_org_contract_boolean_condition(condition) {
        query.predicates.push(predicate);
    }
}

fn parse_org_contract_relation_condition(condition: &str, query: &mut OrgContractQuery) -> bool {
    if let Some(argument) = function_argument(condition, "within")
        .or_else(|| function_argument(condition, "descendant_of"))
    {
        if argument == "$scope" {
            query.use_scope_outline_path = true;
        } else {
            query.relative_to = Some(OrgContractRelativeScope::DescendantOfBinding(
                argument.to_string(),
            ));
        }
        return true;
    }
    if let Some(argument) = function_argument(condition, "child_of") {
        if argument == "$scope" {
            query.use_scope_outline_path = true;
            query.scope_outline_depth = Some(1);
        } else {
            query.relative_to = Some(OrgContractRelativeScope::ChildOfBinding(
                argument.to_string(),
            ));
        }
        return true;
    }
    if let Some(argument) = function_argument(condition, "at") {
        if argument == "$scope" {
            query.use_scope_outline_path = true;
            query.scope_outline_depth = Some(0);
        } else {
            query.relative_to = Some(OrgContractRelativeScope::AtBinding(argument.to_string()));
        }
        return true;
    }
    false
}

fn parse_org_contract_boolean_condition(condition: &str) -> Option<OrgElementQueryPredicate> {
    let condition = condition.trim();
    if let Some((left, right)) = condition.split_once(" or ") {
        let mut predicates = vec![parse_org_contract_boolean_condition(left)?];
        predicates.extend(
            right
                .split(" or ")
                .map(parse_org_contract_boolean_condition)
                .collect::<Option<Vec<_>>>()?,
        );
        return Some(OrgElementQueryPredicate::any(predicates));
    }
    if let Some(rest) = condition.strip_prefix("not ") {
        return parse_org_contract_boolean_condition(rest).map(OrgElementQueryPredicate::negate);
    }
    parse_org_contract_header_predicate(condition)
        .or_else(|| parse_org_contract_summary_predicate(condition))
        .or_else(|| parse_org_contract_property_predicate(condition))
}

fn parse_org_contract_property_predicate(condition: &str) -> Option<OrgElementQueryPredicate> {
    parse_org_contract_field_predicate(
        condition,
        "property",
        OrgElementQueryPredicate::property_eq,
        OrgElementQueryPredicate::property_contains,
    )
}

fn parse_org_contract_summary_predicate(condition: &str) -> Option<OrgElementQueryPredicate> {
    parse_org_contract_field_predicate(
        condition,
        "summary",
        OrgElementQueryPredicate::summary_eq,
        OrgElementQueryPredicate::summary_contains,
    )
}

fn parse_org_contract_header_predicate(condition: &str) -> Option<OrgElementQueryPredicate> {
    let (lhs, rhs) = split_line(condition, "=")?;
    let value = query_value(rhs)?;
    match lhs {
        "affiliated_name" => Some(OrgElementQueryPredicate::AffiliatedName(value)),
        "context" => Some(OrgElementQueryPredicate::Context(value)),
        "category" => {
            OrgElementsIndexCategory::from_label(&value).map(OrgElementQueryPredicate::Category)
        }
        "kind" => Some(OrgElementQueryPredicate::Kind(OrgElementsIndexKind::new(
            value,
        ))),
        _ => None,
    }
}

fn parse_org_contract_field_predicate(
    condition: &str,
    field: &str,
    equals: impl Fn(String, String) -> OrgElementQueryPredicate,
    contains: impl Fn(String, String) -> OrgElementQueryPredicate,
) -> Option<OrgElementQueryPredicate> {
    let prefix = format!("{field}(");
    let rest = condition.strip_prefix(&prefix)?;
    let (key, rhs) = rest.split_once(')')?;
    let rhs = rhs.trim();
    if let Some(value) = rhs
        .strip_prefix("contains")
        .and_then(|value| query_value(value.trim()))
    {
        return Some(contains(key.trim().to_string(), value));
    }
    if let Some(value) = rhs
        .strip_prefix('=')
        .and_then(|value| query_value(value.trim()))
    {
        return Some(equals(key.trim().to_string(), value));
    }
    None
}

fn function_argument<'a>(condition: &'a str, name: &str) -> Option<&'a str> {
    let rest = condition.trim().strip_prefix(name)?.trim();
    let rest = rest.strip_prefix('(')?;
    let (argument, tail) = rest.split_once(')')?;
    tail.trim().is_empty().then_some(argument.trim())
}

fn parse_count_comparison(condition: &str) -> Option<OrgContractExpectation> {
    for op in [
        OrgContractCompareOp::Le,
        OrgContractCompareOp::Lt,
        OrgContractCompareOp::Ge,
        OrgContractCompareOp::Gt,
        OrgContractCompareOp::Eq,
        OrgContractCompareOp::Ne,
    ] {
        if let Some(count) = condition
            .strip_prefix(op.as_str())
            .and_then(|value| value.trim().parse::<usize>().ok())
        {
            return Some(OrgContractExpectation::Count(op, count));
        }
    }
    None
}

fn apply_org_contract_kind(kind: &str, query: &mut OrgContractQuery) {
    let kind = kind.trim().trim_matches('"');
    match kind {
        "org-data" => {
            query.category = Some(OrgElementsIndexCategory::Document);
            query.kind = Some(OrgElementsIndexKind::new("org-data"));
        }
        "headline" => {
            query.category = Some(OrgElementsIndexCategory::Section);
            query.kind = Some(OrgElementsIndexKind::new("headline"));
        }
        "node-property" => {
            query.category = Some(OrgElementsIndexCategory::Property);
            query.kind = Some(OrgElementsIndexKind::new("node-property"));
        }
        "keyword" => {
            query.category = Some(OrgElementsIndexCategory::Keyword);
            query.kind = Some(OrgElementsIndexKind::new("keyword"));
        }
        "link" | "timestamp" | "bold" | "italic" | "underline" | "strike-through"
        | "superscript" | "subscript" | "code" | "verbatim" | "target" | "radio-target"
        | "footnote-reference" | "citation" | "inline-src-block" | "inline-babel-call"
        | "macro" | "plain-text" => {
            query.category = Some(OrgElementsIndexCategory::Object);
            query.kind = Some(OrgElementsIndexKind::new(kind));
        }
        _ => {
            query.category = Some(OrgElementsIndexCategory::Element);
            query.kind = Some(OrgElementsIndexKind::new(kind));
        }
    }
}

fn parse_selector_block(value: &str) -> Option<OrgContractQuery> {
    let selector = OrgElementSelector::parse_plist(value.trim()).ok()?;
    let mut query = OrgContractQuery {
        category: Some(OrgElementsIndexCategory::Element),
        kind: Some(selector.element_type),
        affiliated_name: selector.name,
        ..OrgContractQuery::default()
    };
    if let Some(language) = selector.language {
        query
            .summary_equals
            .push(("language".to_string(), language));
    }
    Some(query)
}

fn parse_expectation(value: &str) -> Option<OrgContractExpectation> {
    let line = value
        .lines()
        .map(strip_query_comment)
        .find(|line| !line.is_empty())?;

    if line == "exists" {
        return Some(OrgContractExpectation::Exists);
    }
    if line == "not exists" {
        return Some(OrgContractExpectation::NotExists);
    }
    let rest = line.strip_prefix("count")?.trim();
    for op in [
        OrgContractCompareOp::Le,
        OrgContractCompareOp::Lt,
        OrgContractCompareOp::Ge,
        OrgContractCompareOp::Gt,
        OrgContractCompareOp::Eq,
        OrgContractCompareOp::Ne,
    ] {
        if let Some(count) = rest
            .strip_prefix(op.as_str())
            .and_then(|value| value.trim().parse::<usize>().ok())
        {
            return Some(OrgContractExpectation::Count(op, count));
        }
    }
    None
}

fn parse_severity(value: &str) -> Option<OrgContractSeverity> {
    match value.trim().to_ascii_lowercase().as_str() {
        "error" => Some(OrgContractSeverity::Error),
        "warning" | "warn" => Some(OrgContractSeverity::Warning),
        _ => None,
    }
}

fn section_property_value(properties: &[Property<ParsedAnnotation>], key: &str) -> Option<String> {
    properties
        .iter()
        .rev()
        .find(|property| property.key.eq_ignore_ascii_case(key))
        .map(|property| property.value.trim().to_string())
}

fn split_line<'a>(line: &'a str, separator: &str) -> Option<(&'a str, &'a str)> {
    line.split_once(separator)
        .map(|(left, right)| (left.trim(), right.trim()))
}

fn strip_query_comment(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.starts_with('#') {
        return String::new();
    }
    trimmed
        .split_once(" #")
        .map_or(trimmed, |(before, _)| before)
        .trim()
        .to_string()
}

fn query_value(value: &str) -> Option<String> {
    let value = value
        .trim()
        .trim_end_matches(',')
        .trim_matches('"')
        .trim_matches('\'')
        .trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn outline_path_value(value: &str) -> Vec<String> {
    value
        .split('/')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

fn contract_aliases(
    source_path: Option<&Path>,
    id: &str,
    declared_aliases: Option<String>,
) -> Vec<String> {
    let mut aliases = Vec::new();
    let mut seen = HashSet::new();

    if let Some(declared_aliases) = declared_aliases {
        for alias in declared_aliases
            .split(',')
            .flat_map(|alias| alias.split_whitespace())
            .map(str::trim)
            .filter(|alias| !alias.is_empty())
        {
            push_alias(&mut aliases, &mut seen, alias.to_string());
        }
    }

    if let Some(source_path) = source_path {
        for base in contract_path_alias_bases(source_path) {
            push_alias(&mut aliases, &mut seen, format!("{base}#{id}"));
            push_alias(&mut aliases, &mut seen, format!("file:{base}#{id}"));
            push_alias(&mut aliases, &mut seen, base.clone());
            push_alias(&mut aliases, &mut seen, format!("file:{base}"));
        }
    }

    aliases
}

fn contract_path_alias_bases(path: &Path) -> Vec<String> {
    let mut bases = vec![normalize_path(path)];
    if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
        bases.push(file_name.to_string());
    }
    if path.is_absolute()
        && let Ok(current_dir) = std::env::current_dir()
        && let Ok(relative) = path.strip_prefix(current_dir)
    {
        bases.push(normalize_path(relative));
    }
    bases.sort();
    bases.dedup();
    bases
}

fn push_alias(aliases: &mut Vec<String>, seen: &mut HashSet<String>, alias: String) {
    if alias.is_empty() || !seen.insert(alias.clone()) {
        return;
    }
    aliases.push(alias);
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn org_link_contract_reference(value: &str) -> Option<OrgContractReference> {
    let inner = value.strip_prefix("[[")?.strip_suffix("]]")?;
    let (target, description) = inner
        .split_once("][")
        .map_or((inner.trim(), None), |(target, description)| {
            (target.trim(), Some(description.trim()))
        });
    if target.is_empty() {
        return None;
    }

    let without_file = target.strip_prefix("file:").unwrap_or(target).trim();
    if let Some((path, contract_id)) = without_file.split_once('#') {
        return Some(OrgContractReference {
            raw: value.to_string(),
            path: contract_reference_path(path),
            contract_id: (!contract_id.trim().is_empty()).then(|| contract_id.trim().to_string()),
        });
    }

    if looks_like_file_reference(without_file) {
        return Some(OrgContractReference {
            raw: value.to_string(),
            path: contract_reference_path(without_file),
            contract_id: description
                .filter(|description| !description.is_empty())
                .map(str::to_string),
        });
    }

    None
}

fn contract_reference_path(value: &str) -> Option<String> {
    let value = value.trim();
    let value = value.strip_prefix("./").unwrap_or(value);
    (!value.is_empty() && !value.starts_with('/')).then(|| value.to_string())
}

fn macro_reference_argument(value: &str) -> Option<String> {
    let inner = value.strip_prefix("{{{")?.strip_suffix("}}}")?.trim();
    let start = inner.find('(')?;
    let end = inner.rfind(')')?;
    (end > start + 1)
        .then(|| inner[start + 1..end].trim().to_string())
        .filter(|argument| !argument.is_empty())
}

fn looks_like_file_reference(value: &str) -> bool {
    value.starts_with("./")
        || value.starts_with("../")
        || value.starts_with('/')
        || value.ends_with(".org")
        || value.contains('/')
}
