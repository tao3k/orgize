//! Org contract parser for `CONTRACT_ORG` validation.

use std::{collections::HashSet, path::Path};

use super::org_elements_query_expr::{
    parse_org_contract_expression_block, parse_org_elements_query_expression_block,
};
use super::{
    ASSERT_ID_PROPERTY, ASSERT_SEVERITY_PROPERTY, CONTRACT_ALIAS_PROPERTY, CONTRACT_ID_PROPERTY,
    CONTRACT_KIND_ORG_ELEMENTS, CONTRACT_KIND_PROPERTY, CONTRACT_SCOPE_PROPERTY, Document,
    OrgContract, OrgContractAssertion, OrgContractCompareOp, OrgContractExpectation,
    OrgContractKind, OrgContractQuery, OrgContractReference, OrgContractRegistry, OrgContractScope,
    OrgContractSeverity, OrgElementSelector, OrgElementsIndexCategory, ParsedAnnotation, Property,
    Section, SourceBlockRecord, SourceBlockRecordKind,
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
        if language.eq_ignore_ascii_case("org-elements-query")
            || language.eq_ignore_ascii_case("org-elements-query-expr")
            || language.eq_ignore_ascii_case("org-elements-expr")
        {
            query = parse_org_elements_query_expression_block(&block.value);
            query_source = Some(block.source.clone());
        } else if language.eq_ignore_ascii_case("org-elements-selector") {
            query = parse_selector_block(&block.value);
            query_source = Some(block.source.clone());
        } else if language.eq_ignore_ascii_case("org-contract") {
            if let Some((parsed_bindings, parsed_query, parsed_expectation)) =
                parse_org_contract_expression_block(&block.value)
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
        .map(strip_block_comment)
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

fn strip_block_comment(line: &str) -> String {
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
    (!value.is_empty() && !value.starts_with('/')).then(|| value.replace('\\', "/"))
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
        || value.contains('\\')
}
