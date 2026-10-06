//! Org contract parser for `CONTRACT_ORG` validation.

use std::{collections::HashSet, path::Path};

#[path = "../../languages/org/v1/modules/org-contract/generated/expectation.rs"]
#[rustfmt::skip]
mod expectation_grammar;

use super::org_elements_query_expr::{
    org_contract_block_is_admitted, org_query_block_is_admitted,
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
    let has_named_assertion_blocks = source_blocks.iter().any(is_named_assertion_block);
    let mut contracts = Vec::new();
    for (index, section) in document.sections.iter().enumerate() {
        collect_contract_sections(
            section,
            bounded_section_end(&document.sections, index, usize::MAX),
            &source_blocks,
            source_path,
            has_named_assertion_blocks,
            &mut contracts,
        );
    }
    OrgContractRegistry::new(contracts)
}

/// Parses and validates a file that is explicitly registered as an Org contract source.
///
/// Unlike `parse_contracts_from_document`, this entry point never treats an empty
/// registry or a contract without assertions as a successful parse. Consumers such
/// as WASM deployment gates should use this function for `[contracts].sources`.
pub fn validate_contract_source(
    document: &Document<ParsedAnnotation>,
    source_path: Option<&Path>,
) -> super::OrgContractSourceValidation {
    let registry = parse_contracts_from_document(document, source_path);
    let path = source_path.map(|path| path.display().to_string());
    let display_path = path.as_deref().unwrap_or("<memory>");
    let mut diagnostics = Vec::new();
    let source_blocks = document.source_block_records();
    let has_named_assertion_blocks = source_blocks.iter().any(is_named_assertion_block);

    for (index, section) in document.sections.iter().enumerate() {
        collect_contract_source_diagnostics(
            section,
            bounded_section_end(&document.sections, index, usize::MAX),
            &source_blocks,
            has_named_assertion_blocks,
            path.as_deref(),
            &mut diagnostics,
        );
    }

    if registry.contracts.is_empty() {
        diagnostics.push(super::OrgContractSourceDiagnostic {
            code: "CONTRACT-E001",
            path: path.clone(),
            contract_id: None,
            message: format!(
                "{display_path}: contract source contains no valid CONTRACT_ID definitions"
            ),
        });
    }

    let mut origins = std::collections::BTreeSet::new();
    for contract in &registry.contracts {
        if !origins.insert(contract.id.as_str()) {
            diagnostics.push(super::OrgContractSourceDiagnostic {
                code: "CONTRACT-E002",
                path: path.clone(),
                contract_id: Some(contract.id.clone()),
                message: format!(
                    "{display_path}: duplicate CONTRACT_ID `{}` in contract source",
                    contract.id
                ),
            });
        }
        if contract.assertions.is_empty() {
            diagnostics.push(super::OrgContractSourceDiagnostic {
                code: "CONTRACT-E003",
                path: path.clone(),
                contract_id: Some(contract.id.clone()),
                message: format!(
                    "{display_path}: CONTRACT_ID `{}` contains no valid assertions",
                    contract.id
                ),
            });
        }
    }

    super::OrgContractSourceValidation {
        registry,
        diagnostics,
    }
}

fn collect_contract_source_diagnostics(
    section: &Section<ParsedAnnotation>,
    end: usize,
    source_blocks: &[SourceBlockRecord],
    has_named_assertion_blocks: bool,
    path: Option<&str>,
    diagnostics: &mut Vec<super::OrgContractSourceDiagnostic>,
) {
    let contract_id = section_property_value(&section.properties, CONTRACT_ID_PROPERTY);
    let contract_kind = section_property_value(&section.properties, CONTRACT_KIND_PROPERTY);
    let contract_scope = section_property_value(&section.properties, CONTRACT_SCOPE_PROPERTY);
    let contract_alias = section_property_value(&section.properties, CONTRACT_ALIAS_PROPERTY);
    let is_contract_section = contract_id.is_some()
        || contract_kind.is_some()
        || contract_scope.is_some()
        || contract_alias.is_some();

    if is_contract_section {
        let normalized_id = contract_id
            .as_deref()
            .map(|value| super::org_native_values::scalar("contract-policy", &["name", value]))
            .filter(|value| !value.is_empty());
        if normalized_id.is_none() {
            push_contract_source_diagnostic(
                diagnostics,
                "CONTRACT-E004",
                path,
                None,
                "contract section is missing a non-empty CONTRACT_ID",
            );
        }
        if let Some(kind) = contract_kind.as_deref()
            && OrgContractKind::parse(kind).is_none()
        {
            push_contract_source_diagnostic(
                diagnostics,
                "CONTRACT-E005",
                path,
                normalized_id.as_deref(),
                format!(
                    "CONTRACT_ID uses unsupported CONTRACT_KIND `{}`",
                    kind.trim()
                ),
            );
        }
        if let Some(scope) = contract_scope.as_deref()
            && OrgContractScope::parse(scope).is_none()
        {
            push_contract_source_diagnostic(
                diagnostics,
                "CONTRACT-E006",
                path,
                normalized_id.as_deref(),
                format!(
                    "CONTRACT_ID uses unsupported CONTRACT_SCOPE `{}`",
                    scope.trim()
                ),
            );
        }
        collect_assertion_source_diagnostics(
            section,
            end,
            source_blocks,
            has_named_assertion_blocks,
            normalized_id.as_deref(),
            path,
            diagnostics,
        );
    }

    for (index, child) in section.subsections.iter().enumerate() {
        collect_contract_source_diagnostics(
            child,
            bounded_section_end(&section.subsections, index, end),
            source_blocks,
            has_named_assertion_blocks,
            path,
            diagnostics,
        );
    }
}

fn collect_assertion_source_diagnostics(
    section: &Section<ParsedAnnotation>,
    end: usize,
    source_blocks: &[SourceBlockRecord],
    has_named_assertion_blocks: bool,
    contract_id: Option<&str>,
    path: Option<&str>,
    diagnostics: &mut Vec<super::OrgContractSourceDiagnostic>,
) {
    let direct_end = section
        .subsections
        .first()
        .map(section_start)
        .unwrap_or(end);
    let direct_blocks = section_source_blocks(section, direct_end, source_blocks);
    let assertion_id = section_property_value(&section.properties, ASSERT_ID_PROPERTY);
    let assertion_severity = section_property_value(&section.properties, ASSERT_SEVERITY_PROPERTY);
    let has_unnamed_query = direct_blocks.iter().any(|block| {
        let is_query = matches!(
            contract_block_role(block).as_str(),
            "query" | "selector" | "contract"
        );
        is_query && (!has_named_assertion_blocks || !is_named_assertion_block(block))
    });
    let is_assertion_section =
        assertion_id.is_some() || assertion_severity.is_some() || has_unnamed_query;

    if is_assertion_section {
        let normalized_assertion_id = assertion_id
            .as_deref()
            .map(|value| super::org_native_values::scalar("contract-policy", &["name", value]))
            .filter(|value| !value.is_empty());
        if normalized_assertion_id.is_none() {
            push_contract_source_diagnostic(
                diagnostics,
                "CONTRACT-E007",
                path,
                contract_id,
                "assertion section is missing a non-empty ASSERT_ID",
            );
        }
        if let Some(severity) = assertion_severity.as_deref()
            && parse_severity(severity).is_none()
        {
            push_contract_source_diagnostic(
                diagnostics,
                "CONTRACT-E008",
                path,
                contract_id,
                format!(
                    "ASSERT_ID uses unsupported ASSERT_SEVERITY `{}`",
                    severity.trim()
                ),
            );
        }
        if normalized_assertion_id.is_some()
            && parse_assertion(section, direct_end, source_blocks).is_none()
        {
            push_contract_source_diagnostic(
                diagnostics,
                "CONTRACT-E009",
                path,
                contract_id,
                format!(
                    "ASSERT_ID `{}` has no valid contract query",
                    normalized_assertion_id.unwrap_or_default()
                ),
            );
        }
    }

    for (index, child) in section.subsections.iter().enumerate() {
        collect_assertion_source_diagnostics(
            child,
            bounded_section_end(&section.subsections, index, end),
            source_blocks,
            has_named_assertion_blocks,
            contract_id,
            path,
            diagnostics,
        );
    }
}

fn push_contract_source_diagnostic(
    diagnostics: &mut Vec<super::OrgContractSourceDiagnostic>,
    code: &'static str,
    path: Option<&str>,
    contract_id: Option<&str>,
    message: impl Into<String>,
) {
    let message = message.into();
    let display_path = path.unwrap_or("<memory>");
    diagnostics.push(super::OrgContractSourceDiagnostic {
        code,
        path: path.map(str::to_string),
        contract_id: contract_id.map(str::to_string),
        message: format!("{display_path}: {message}"),
    });
}

/// Parses a `CONTRACT_ORG` property/keyword value.
pub fn parse_contract_reference(value: &str) -> OrgContractReference {
    let row = super::org_native_values::rows("contract-reference", &[value])
        .pop()
        .expect("native contract reference");
    let [raw, path, has_path, contract_id, has_id]: [String; 5] =
        row.try_into().expect("native reference arity");
    let present = |flag: &str| match flag {
        "true" => true,
        "false" => false,
        _ => panic!("native reference presence"),
    };
    OrgContractReference {
        raw,
        path: present(&has_path).then_some(path),
        contract_id: present(&has_id).then_some(contract_id),
    }
}
/// Native top-level reference grammar; links and macros remain atomic.
pub fn parse_contract_references(value: &str) -> Vec<OrgContractReference> {
    let values = super::org_native_values::rows("contract-values", &[value])
        .pop()
        .expect("native contract values");
    if values.is_empty() {
        vec![parse_contract_reference(value)]
    } else {
        values
            .iter()
            .map(|value| parse_contract_reference(value))
            .collect()
    }
}

/// Parses a `CONTRACT_ORG` value and resolves a relative file target from its owning Org file.
pub fn parse_contract_reference_from_source(
    value: &str,
    source_path: Option<&Path>,
) -> OrgContractReference {
    let mut reference = parse_contract_reference(value);
    let Some(path) = reference.path.as_deref() else {
        return reference;
    };
    let path = Path::new(path);
    if path.is_absolute() {
        return reference;
    }
    let Some(parent) = source_path.and_then(Path::parent) else {
        return reference;
    };
    reference.path = Some(normalize_lexical_path(&parent.join(path)));
    reference
}

fn normalize_lexical_path(path: &Path) -> String {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if components.last().is_some_and(|component| component != "..") {
                    components.pop();
                } else {
                    components.push("..".into());
                }
            }
            component => components.push(component.as_os_str().to_os_string()),
        }
    }
    normalize_path(&components.iter().collect::<std::path::PathBuf>())
}

fn collect_contract_sections(
    section: &Section<ParsedAnnotation>,
    end: usize,
    source_blocks: &[SourceBlockRecord],
    source_path: Option<&Path>,
    has_named_assertion_blocks: bool,
    contracts: &mut Vec<OrgContract>,
) {
    if let Some(contract) = parse_contract_section(
        section,
        end,
        source_blocks,
        source_path,
        has_named_assertion_blocks,
    ) {
        contracts.push(contract);
    }
    for (index, child) in section.subsections.iter().enumerate() {
        collect_contract_sections(
            child,
            bounded_section_end(&section.subsections, index, end),
            source_blocks,
            source_path,
            has_named_assertion_blocks,
            contracts,
        );
    }
}

fn parse_contract_section(
    section: &Section<ParsedAnnotation>,
    end: usize,
    source_blocks: &[SourceBlockRecord],
    source_path: Option<&Path>,
    has_named_assertion_blocks: bool,
) -> Option<OrgContract> {
    let id = section_property_value(&section.properties, CONTRACT_ID_PROPERTY)?;
    if id.is_empty() {
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

    let mut assertions = if has_named_assertion_blocks {
        parse_named_assertions(&section_source_blocks(section, end, source_blocks))
    } else {
        Vec::new()
    };
    collect_assertions(section, end, source_blocks, &mut assertions);

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
    end: usize,
    source_blocks: &[SourceBlockRecord],
    assertions: &mut Vec<OrgContractAssertion>,
) {
    if let Some(assertion) = parse_assertion(section, end, source_blocks) {
        assertions.push(assertion);
        return;
    }

    for (index, child) in section.subsections.iter().enumerate() {
        collect_assertions(
            child,
            bounded_section_end(&section.subsections, index, end),
            source_blocks,
            assertions,
        );
    }
}

fn parse_assertion(
    section: &Section<ParsedAnnotation>,
    end: usize,
    source_blocks: &[SourceBlockRecord],
) -> Option<OrgContractAssertion> {
    let id = section_property_value(&section.properties, ASSERT_ID_PROPERTY)?;
    if id.is_empty() {
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

    for block in section_source_blocks(section, end, source_blocks) {
        let role = contract_block_role(block);
        if role == "query" {
            query = Some(parse_org_elements_query_expression_block(&block.value)?);
            query_source = Some(block.source.clone());
        } else if role == "selector" {
            query = Some(parse_selector_block(&block.value)?);
            query_source = Some(block.source.clone());
        } else if role == "contract" {
            {
                let (parsed_bindings, parsed_query, parsed_expectation) =
                    parse_org_contract_expression_block(&block.value)?;
                bindings = parsed_bindings;
                query = Some(parsed_query);
                expectation = Some(parsed_expectation);
                query_source = Some(block.source.clone());
                expect_source = Some(block.source.clone());
            }
        } else if role == "expect" {
            expectation = Some(parse_expectation(&block.value)?);
            expect_source = Some(block.source.clone());
        } else if role == "template" {
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

fn parse_named_assertions(blocks: &[&SourceBlockRecord]) -> Vec<OrgContractAssertion> {
    blocks
        .iter()
        .filter_map(|block| parse_named_assertion(block, blocks))
        .collect()
}

fn is_named_assertion_block(block: &SourceBlockRecord) -> bool {
    contract_block_role(block) == "contract" && native_named_assertion_id(block).is_some()
}

fn native_named_assertion_id(block: &SourceBlockRecord) -> Option<String> {
    let id =
        super::org_native_values::scalar("contract-policy", &["named-id", block.name.as_deref()?]);
    (!id.is_empty()).then_some(id)
}

fn parse_named_assertion(
    block: &SourceBlockRecord,
    blocks: &[&SourceBlockRecord],
) -> Option<OrgContractAssertion> {
    if !is_named_assertion_block(block) {
        return None;
    }

    let id = native_named_assertion_id(block)?;

    let (bindings, query, expectation) = parse_org_contract_expression_block(&block.value)?;
    let severity = block_parameter_value(block, ":severity")
        .and_then(|value| parse_severity(&value))
        .unwrap_or_default();
    let message = named_block_value(blocks, &format!("{id}.message"), "jinja2");
    let fix = named_block_value(blocks, &format!("{id}.fix"), "jinja2");

    Some(OrgContractAssertion {
        id,
        severity,
        bindings,
        query,
        expectation,
        message,
        fix,
        query_source: Some(block.source.clone()),
        expect_source: Some(block.source.clone()),
    })
}

fn named_block_value(blocks: &[&SourceBlockRecord], name: &str, language: &str) -> Option<String> {
    blocks
        .iter()
        .find(|block| {
            block_language_is(block, language)
                && block.name.as_deref().is_some_and(|block_name| {
                    super::org_native_values::scalar("contract-policy", &["name", block_name])
                        == name
                })
        })
        .map(|block| block.value.clone())
}

fn section_source_blocks<'a>(
    section: &Section<ParsedAnnotation>,
    end: usize,
    source_blocks: &'a [SourceBlockRecord],
) -> Vec<&'a SourceBlockRecord> {
    let start = usize::from(section.ann.range.start());
    source_blocks
        .iter()
        .filter(|record| source_block_in_range(record, start, end))
        .collect()
}

fn source_block_in_range(record: &SourceBlockRecord, start: usize, end: usize) -> bool {
    matches!(record.kind, SourceBlockRecordKind::Block)
        && (record.source.range_start as usize) >= start
        && (record.source.range_end as usize) <= end
}

fn bounded_section_end(
    siblings: &[Section<ParsedAnnotation>],
    index: usize,
    parent_end: usize,
) -> usize {
    siblings
        .get(index + 1)
        .map(section_start)
        .unwrap_or(parent_end)
}

fn section_start(section: &Section<ParsedAnnotation>) -> usize {
    usize::from(section.ann.range.start())
}

fn block_parameter_name(block: &SourceBlockRecord) -> Option<String> {
    block_parameter_value(block, ":name")
}

fn block_parameter_value(block: &SourceBlockRecord, key: &str) -> Option<String> {
    let raw = block.parameters.as_deref()?;
    super::org_native_values::rows("block-parameter", &[raw, key])
        .pop()
        .map(|row| {
            let [value]: [String; 1] = row.try_into().expect("native block parameter arity");
            value
        })
}

fn block_language_is(block: &SourceBlockRecord, language: &str) -> bool {
    contract_block_role(block)
        == super::org_native_values::scalar("contract-policy", &["language", language])
}

fn contract_block_role(block: &SourceBlockRecord) -> String {
    super::org_native_values::scalar(
        "contract-policy",
        &["language", block.language.as_deref().unwrap_or_default()],
    )
}

/// Admission of raw source blocks precedes registry extraction/evaluation.
/// Reuse the owning compilers; never infer validity from a populated registry.
pub(crate) fn contract_block_syntax_error(block: &SourceBlockRecord) -> Option<&'static str> {
    let role = contract_block_role(block);
    let valid = if role == "query" {
        org_query_block_is_admitted(&block.value)
    } else if role == "selector" {
        parse_selector_block(&block.value).is_some()
    } else if role == "expect" {
        parse_expectation(&block.value).is_some()
    } else if role == "contract" {
        org_contract_block_is_admitted(&block.value)
    } else {
        return None;
    };
    (!valid).then_some("invalid or unsupported Org Query/Contract source-block syntax")
}

fn parse_selector_block(value: &str) -> Option<OrgContractQuery> {
    let selector = OrgElementSelector::parse_plist(value).ok()?;
    if selector.element_type == crate::ast::OrgElementsIndexKind::new("keyword") {
        return Some(OrgContractQuery {
            document_predicates: vec![crate::ast::OrgContractDocumentPredicate::MetadataExists(
                selector.name?,
            )],
            ..OrgContractQuery::default()
        });
    }
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
    use crate::org_aot::NativeExpressionValue::Atom;
    let values =
        crate::org_aot::parse_native_expectation_values(value, &expectation_grammar::LANGUAGE)
            .ok()?;
    match values.as_slice() {
        [Atom(exists)] if exists == "exists" => Some(OrgContractExpectation::Exists),
        [Atom(not), Atom(exists)] if not == "not" && exists == "exists" => {
            Some(OrgContractExpectation::NotExists)
        }
        [Atom(count), Atom(operator), Atom(number)] if count == "count" => {
            let op = match operator.as_str() {
                "<=" => OrgContractCompareOp::Le,
                "<" => OrgContractCompareOp::Lt,
                ">=" => OrgContractCompareOp::Ge,
                ">" => OrgContractCompareOp::Gt,
                "==" => OrgContractCompareOp::Eq,
                "!=" => OrgContractCompareOp::Ne,
                _ => return None,
            };
            Some(OrgContractExpectation::Count(op, number.parse().ok()?))
        }
        _ => None,
    }
}

fn parse_severity(value: &str) -> Option<OrgContractSeverity> {
    let row = super::org_native_values::optional("severity", value)?;
    let [kind]: [String; 1] = row.try_into().expect("native severity arity");
    Some(match kind.as_str() {
        "error" => OrgContractSeverity::Error,
        "warning" => OrgContractSeverity::Warning,
        _ => panic!("native severity kind"),
    })
}

fn section_property_value(properties: &[Property<ParsedAnnotation>], key: &str) -> Option<String> {
    properties
        .iter()
        .rev()
        .find(|property| property.key.eq_ignore_ascii_case(key))
        .map(|property| {
            super::org_native_values::scalar("contract-policy", &["name", &property.value])
        })
}

fn contract_aliases(
    source_path: Option<&Path>,
    id: &str,
    declared_aliases: Option<String>,
) -> Vec<String> {
    let mut aliases = Vec::new();
    let mut seen = HashSet::new();

    if let Some(declared_aliases) = declared_aliases {
        for alias in super::org_native_values::rows("contract-aliases", &[&declared_aliases])
            .pop()
            .expect("native aliases")
        {
            push_alias(&mut aliases, &mut seen, alias);
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
