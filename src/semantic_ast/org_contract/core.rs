//! Org contract parser for `CONTRACT_ORG` validation.

use std::{collections::HashSet, path::Path};

use super::blocks::ContractBlock;
use super::source_plan::{SourcePlan, plan_document, plan_document_records};

use crate::ast::org_elements_query_expr::{
    compile_contract_values, compile_query_values, contract_values_are_admitted,
    query_values_are_admitted, selector_properties_from_values,
};
use crate::ast::{
    ASSERT_ID_PROPERTY, ASSERT_SEVERITY_PROPERTY, CONTRACT_ALIAS_PROPERTY, CONTRACT_ID_PROPERTY,
    CONTRACT_KIND_PROPERTY, CONTRACT_SCOPE_PROPERTY, Document, OrgContract, OrgContractAssertion,
    OrgContractCompareOp, OrgContractExpectation, OrgContractQuery, OrgContractReference,
    OrgContractRegistry, OrgElementSelector, OrgElementsIndexCategory, ParsedAnnotation, Section,
    SourceBlockRecordKind, SourceBlockSyntaxRecord,
};

/// Parses a host-loaded Org contract registry from an Org document.
pub fn parse_contracts_from_document(
    document: &Document<ParsedAnnotation>,
    source_path: Option<&Path>,
) -> OrgContractRegistry {
    let plan = plan_document(document);
    registry_from_plan(document, source_path, &plan)
}

fn registry_from_plan(
    document: &Document<ParsedAnnotation>,
    source_path: Option<&Path>,
    plan: &SourcePlan<'_>,
) -> OrgContractRegistry {
    let has_named_assertion_blocks = plan.iter().any(is_named_assertion_block);
    let mut contracts = Vec::new();
    for (index, section) in document.sections.iter().enumerate() {
        collect_contract_sections(
            section,
            bounded_section_end(&document.sections, index, usize::MAX),
            plan,
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
) -> crate::ast::OrgContractSourceValidation {
    let plan = plan_document(document);
    let registry = registry_from_plan(document, source_path, &plan);
    let path = source_path.map(|path| path.display().to_string());
    let display_path = path.as_deref().unwrap_or("<memory>");
    let mut diagnostics = Vec::new();
    let has_named_assertion_blocks = plan.iter().any(is_named_assertion_block);

    for (index, section) in document.sections.iter().enumerate() {
        collect_contract_source_diagnostics(
            section,
            bounded_section_end(&document.sections, index, usize::MAX),
            &plan,
            has_named_assertion_blocks,
            path.as_deref(),
            &mut diagnostics,
        );
    }

    if registry.contracts.is_empty() {
        diagnostics.push(crate::ast::OrgContractSourceDiagnostic {
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
            diagnostics.push(crate::ast::OrgContractSourceDiagnostic {
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
            diagnostics.push(crate::ast::OrgContractSourceDiagnostic {
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

    crate::ast::OrgContractSourceValidation {
        registry,
        diagnostics,
    }
}

fn collect_contract_source_diagnostics(
    section: &Section<ParsedAnnotation>,
    end: usize,
    plan: &SourcePlan<'_>,
    has_named_assertion_blocks: bool,
    path: Option<&str>,
    diagnostics: &mut Vec<crate::ast::OrgContractSourceDiagnostic>,
) {
    let contract_id = plan.section(section).value(CONTRACT_ID_PROPERTY);
    let contract_kind = plan.section(section).value(CONTRACT_KIND_PROPERTY);
    let contract_scope = plan.section(section).value(CONTRACT_SCOPE_PROPERTY);
    let contract_alias = plan.section(section).value(CONTRACT_ALIAS_PROPERTY);
    let is_contract_section = contract_id.is_some()
        || contract_kind.is_some()
        || contract_scope.is_some()
        || contract_alias.is_some();

    if is_contract_section {
        let normalized_id = contract_id.as_deref().filter(|value| !value.is_empty());
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
            && plan.section(section).kind.is_none()
        {
            push_contract_source_diagnostic(
                diagnostics,
                "CONTRACT-E005",
                path,
                normalized_id,
                format!(
                    "CONTRACT_ID uses unsupported CONTRACT_KIND `{}`",
                    kind.trim()
                ),
            );
        }
        if let Some(scope) = contract_scope.as_deref()
            && plan.section(section).scope.is_none()
        {
            push_contract_source_diagnostic(
                diagnostics,
                "CONTRACT-E006",
                path,
                normalized_id,
                format!(
                    "CONTRACT_ID uses unsupported CONTRACT_SCOPE `{}`",
                    scope.trim()
                ),
            );
        }
        collect_assertion_source_diagnostics(
            section,
            end,
            plan,
            has_named_assertion_blocks,
            normalized_id,
            path,
            diagnostics,
        );
    }

    for (index, child) in section.subsections.iter().enumerate() {
        collect_contract_source_diagnostics(
            child,
            bounded_section_end(&section.subsections, index, end),
            plan,
            has_named_assertion_blocks,
            path,
            diagnostics,
        );
    }
}

fn collect_assertion_source_diagnostics(
    section: &Section<ParsedAnnotation>,
    end: usize,
    plan: &SourcePlan<'_>,
    has_named_assertion_blocks: bool,
    contract_id: Option<&str>,
    path: Option<&str>,
    diagnostics: &mut Vec<crate::ast::OrgContractSourceDiagnostic>,
) {
    let direct_end = section
        .subsections
        .first()
        .map(section_start)
        .unwrap_or(end);
    let direct_blocks = section_plan(section, direct_end, plan);
    let assertion_id = plan.section(section).value(ASSERT_ID_PROPERTY);
    let assertion_severity = plan.section(section).value(ASSERT_SEVERITY_PROPERTY);
    let has_unnamed_query = direct_blocks.iter().any(|block| {
        let is_query = matches!(
            contract_block_role(block),
            "query" | "selector" | "contract"
        );
        is_query && (!has_named_assertion_blocks || !is_named_assertion_block(block))
    });
    let is_assertion_section =
        assertion_id.is_some() || assertion_severity.is_some() || has_unnamed_query;

    if is_assertion_section {
        let normalized_assertion_id = assertion_id.as_deref().filter(|value| !value.is_empty());
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
            && plan.section(section).severity.is_none()
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
        if normalized_assertion_id.is_some() && parse_assertion(section, direct_end, plan).is_none()
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
            plan,
            has_named_assertion_blocks,
            contract_id,
            path,
            diagnostics,
        );
    }
}

fn push_contract_source_diagnostic(
    diagnostics: &mut Vec<crate::ast::OrgContractSourceDiagnostic>,
    code: &'static str,
    path: Option<&str>,
    contract_id: Option<&str>,
    message: impl Into<String>,
) {
    let message = message.into();
    let display_path = path.unwrap_or("<memory>");
    diagnostics.push(crate::ast::OrgContractSourceDiagnostic {
        code,
        path: path.map(str::to_string),
        contract_id: contract_id.map(str::to_string),
        message: format!("{display_path}: {message}"),
    });
}

/// Parses a `CONTRACT_ORG` property/keyword value.
pub fn parse_contract_reference(value: &str) -> OrgContractReference {
    let row = crate::ast::org_values::rows("contract-reference", &[value])
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
    let values = crate::ast::org_values::rows("contract-values", &[value])
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
    plan: &SourcePlan<'_>,
    source_path: Option<&Path>,
    has_named_assertion_blocks: bool,
    contracts: &mut Vec<OrgContract>,
) {
    if let Some(contract) =
        parse_contract_section(section, end, plan, source_path, has_named_assertion_blocks)
    {
        contracts.push(contract);
    }
    for (index, child) in section.subsections.iter().enumerate() {
        collect_contract_sections(
            child,
            bounded_section_end(&section.subsections, index, end),
            plan,
            source_path,
            has_named_assertion_blocks,
            contracts,
        );
    }
}

fn parse_contract_section(
    section: &Section<ParsedAnnotation>,
    end: usize,
    plan: &SourcePlan<'_>,
    source_path: Option<&Path>,
    has_named_assertion_blocks: bool,
) -> Option<OrgContract> {
    let id = plan.section(section).value(CONTRACT_ID_PROPERTY)?;
    if id.is_empty() {
        return None;
    }

    let facts = plan.section(section);
    let kind = facts.kind?;
    let aliases = contract_aliases(source_path, id.as_str(), &facts.aliases);
    let scope = facts.scope.unwrap_or_default();

    let mut assertions = if has_named_assertion_blocks {
        parse_named_assertions(&section_plan(section, end, plan))
    } else {
        Vec::new()
    };
    collect_assertions(section, end, plan, &mut assertions);

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
    plan: &SourcePlan<'_>,
    assertions: &mut Vec<OrgContractAssertion>,
) {
    if let Some(assertion) = parse_assertion(section, end, plan) {
        assertions.push(assertion);
        return;
    }

    for (index, child) in section.subsections.iter().enumerate() {
        collect_assertions(
            child,
            bounded_section_end(&section.subsections, index, end),
            plan,
            assertions,
        );
    }
}

fn parse_assertion(
    section: &Section<ParsedAnnotation>,
    end: usize,
    plan: &SourcePlan<'_>,
) -> Option<OrgContractAssertion> {
    let id = plan.section(section).value(ASSERT_ID_PROPERTY)?;
    if id.is_empty() {
        return None;
    }

    let severity = plan.section(section).severity.unwrap_or_default();

    let mut query = None;
    let mut bindings = Vec::new();
    let mut expectation = None;
    let mut message = None;
    let mut fix = None;
    let mut query_source = None;
    let mut expect_source = None;

    for block in section_plan(section, end, plan) {
        let role = contract_block_role(block);
        if role == "query" {
            query = Some(
                block
                    .query_forms
                    .as_deref()
                    .and_then(compile_query_values)?,
            );
            query_source = Some(block.source.clone());
        } else if role == "selector" {
            query = Some(parse_selector_block(block)?);
            query_source = Some(block.source.clone());
        } else if role == "contract" {
            {
                let (parsed_bindings, parsed_query, parsed_expectation) = block
                    .contract_forms
                    .as_deref()
                    .and_then(compile_contract_values)?;
                bindings = parsed_bindings;
                query = Some(parsed_query);
                expectation = Some(parsed_expectation);
                query_source = Some(block.source.clone());
                expect_source = Some(block.source.clone());
            }
        } else if role == "expect" {
            expectation = Some(parse_expectation(block)?);
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

fn parse_named_assertions(blocks: &[&ContractBlock]) -> Vec<OrgContractAssertion> {
    blocks
        .iter()
        .filter_map(|block| parse_named_assertion(block, blocks))
        .collect()
}

fn is_named_assertion_block(block: &ContractBlock) -> bool {
    contract_block_role(block) == "contract" && block.named_id.is_some()
}

fn native_named_assertion_id(block: &ContractBlock) -> Option<String> {
    block.named_id.clone()
}

fn parse_named_assertion(
    block: &ContractBlock,
    blocks: &[&ContractBlock],
) -> Option<OrgContractAssertion> {
    if !is_named_assertion_block(block) {
        return None;
    }

    let id = native_named_assertion_id(block)?;

    let (bindings, query, expectation) = block
        .contract_forms
        .as_deref()
        .and_then(compile_contract_values)?;
    let severity = block.severity.unwrap_or_default();
    let message = named_block_value(blocks, &format!("{id}.message"));
    let fix = named_block_value(blocks, &format!("{id}.fix"));

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

fn named_block_value(blocks: &[&ContractBlock], name: &str) -> Option<String> {
    blocks
        .iter()
        .find(|block| {
            block.role == "template" && block.name.is_some() && block.normalized_name == name
        })
        .map(|block| block.value.clone())
}

fn section_plan<'a>(
    section: &Section<ParsedAnnotation>,
    end: usize,
    plan: &'a SourcePlan<'_>,
) -> Vec<&'a ContractBlock> {
    let start = usize::from(section.ann.range.start());
    plan.iter()
        .filter(|record| source_block_in_range(record, start, end))
        .collect()
}

fn source_block_in_range(record: &SourceBlockSyntaxRecord, start: usize, end: usize) -> bool {
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

fn block_parameter_name(block: &ContractBlock) -> Option<String> {
    block.parameter_name.clone()
}

fn contract_block_role(block: &ContractBlock) -> &str {
    &block.role
}

/// Admission of raw source blocks precedes registry extraction/evaluation.
/// Reuse the owning compilers; never infer validity from a populated registry.
pub(crate) fn contract_block_syntax_error(block: &ContractBlock) -> Option<&'static str> {
    let role = block.role.as_str();
    let valid = if role == "query" {
        block
            .query_forms
            .as_deref()
            .is_some_and(query_values_are_admitted)
    } else if role == "selector" {
        parse_selector_block(block).is_some()
    } else if role == "expect" {
        parse_expectation(block).is_some()
    } else if role == "contract" {
        contract_values_are_admitted(
            block.contract_forms.as_deref(),
            block.query_forms.as_deref(),
        )
    } else {
        return None;
    };
    (!valid).then_some("invalid or unsupported Org Query/Contract source-block syntax")
}

fn parse_selector_block(block: &ContractBlock) -> Option<OrgContractQuery> {
    let properties = selector_properties_from_values(block.query_forms.as_deref()?).ok()?;
    let selector = OrgElementSelector::from_native_properties(properties).ok()?;
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

fn parse_expectation(block: &ContractBlock) -> Option<OrgContractExpectation> {
    use crate::org_aot::NativeExpressionValue::Atom;
    let values = block.expect_forms.as_deref()?;
    match values {
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

fn contract_aliases(
    source_path: Option<&Path>,
    id: &str,
    declared_aliases: &[String],
) -> Vec<String> {
    let mut aliases = Vec::new();
    let mut seen = HashSet::new();

    for alias in declared_aliases {
        push_alias(&mut aliases, &mut seen, alias.clone());
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

/// Shared native source admission for syntax lint; empty source catalogs do no work.
pub(crate) fn contract_source_blocks(document: &Document<ParsedAnnotation>) -> Vec<ContractBlock> {
    let records = document.source_block_syntax_records();
    if records.is_empty() {
        return Vec::new();
    }
    plan_document_records(document, records).blocks
}
