//! Built-in lint evaluation over the Scheme-AOT Org Contract pack.

use rowan::TextRange;

use crate::{
    Org,
    contract_feature::{ContractOperator, ContractScopeNodeId, ContractSeverity},
};

use super::super::{LintFinding, LintSeverity, location_for_range};

pub(super) fn contract_findings(org: &Org, source: &str) -> Vec<LintFinding> {
    let contract = crate::org_aot::org_contract_pack()
        .rules
        .iter()
        .find(|rule| rule.id == "orgize.builtin.document-metadata.v1")
        .expect("Scheme-AOT builtin document metadata contract is generated");
    let results = org
        .evaluate_contract(contract, ContractScopeNodeId(0))
        .expect("builtin contract matches its Scheme-AOT graph");
    contract
        .assertions
        .iter()
        .zip(results)
        .filter(|(_, result)| !result.passed)
        .map(|(assertion, result)| LintFinding {
            code: "ORG044",
            severity: match result.severity {
                ContractSeverity::Error => LintSeverity::Error,
                ContractSeverity::Warning | ContractSeverity::Info => LintSeverity::Warning,
            },
            message: assertion_message(contract.id, assertion, result.matched_count),
            location: location_for_range(source, TextRange::empty(0.into())),
        })
        .collect()
}

fn assertion_message(
    contract_id: &str,
    assertion: &crate::contract_feature::ContractAssertionRule,
    matched_count: usize,
) -> String {
    let expected = match assertion.expectation.operator {
        ContractOperator::AtLeast if assertion.expectation.count == 1 => "exists".to_string(),
        ContractOperator::AtLeast => format!("count >= {}", assertion.expectation.count),
        ContractOperator::Exactly => format!("count == {}", assertion.expectation.count),
        ContractOperator::AtMost => format!("count <= {}", assertion.expectation.count),
    };
    let detail = format!(
        "contract `{contract_id}` assertion `{}` failed in document `<document>`: expected {expected}, actual {matched_count}",
        assertion.id,
    );
    let prefix = assertion.message.unwrap_or_default();
    let mut message = if prefix.is_empty() {
        detail
    } else {
        format!("{prefix} ({detail})")
    };
    if let Some(fix) = assertion.fix.filter(|fix| !fix.is_empty()) {
        message.push_str(&format!(" Suggested fix: {fix}"));
    }
    message
}
