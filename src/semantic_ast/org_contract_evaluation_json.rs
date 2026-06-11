//! JSON trace export for `CONTRACT_ORG` evaluation facts.

use serde_json::{Map, Value, json};

use super::{
    OrgContractAssertionEvaluation, OrgContractAssertionStatus, OrgContractEvaluation,
    OrgContractEvaluationScope, OrgContractExpectation, OrgContractSeverity, OrgElementId,
};

/// Renders one contract evaluation as a source-backed JSON trace.
pub fn evaluation_to_json_value(evaluation: &OrgContractEvaluation) -> Value {
    json!({
        "schemaVersion": 1,
        "contractId": evaluation.contract_id,
        "scope": scope_to_json_value(&evaluation.scope),
        "assertions": evaluation
            .assertions
            .iter()
            .map(assertion_to_json_value)
            .collect::<Vec<_>>(),
    })
}

/// Renders contract evaluations as a compact JSON array.
pub fn evaluations_to_json_value(evaluations: &[OrgContractEvaluation]) -> Value {
    Value::Array(evaluations.iter().map(evaluation_to_json_value).collect())
}

fn scope_to_json_value(scope: &OrgContractEvaluationScope) -> Value {
    match scope {
        OrgContractEvaluationScope::Document { range } => json!({
            "kind": "document",
            "range": {
                "start": usize::from(range.start()),
                "end": usize::from(range.end()),
            },
        }),
        OrgContractEvaluationScope::Section {
            title,
            outline_path,
            range,
        } => json!({
            "kind": "section",
            "title": title,
            "outlinePath": outline_path,
            "range": {
                "start": usize::from(range.start()),
                "end": usize::from(range.end()),
            },
        }),
    }
}

fn assertion_to_json_value(assertion: &OrgContractAssertionEvaluation) -> Value {
    let mut object = Map::new();
    object.insert("assertionId".to_string(), json!(assertion.assertion_id));
    object.insert(
        "severity".to_string(),
        json!(severity_label(assertion.severity)),
    );
    object.insert(
        "expectation".to_string(),
        expectation_to_json_value(&assertion.expectation),
    );
    object.insert("actualCount".to_string(), json!(assertion.actual_count));
    object.insert("status".to_string(), json!(status_label(assertion.status)));
    object.insert(
        "matchedIds".to_string(),
        ids_to_json_value(&assertion.matched_ids),
    );
    object.insert(
        "bindings".to_string(),
        bindings_to_json_value(&assertion.bindings),
    );
    if let Some(template) = &assertion.message_template {
        object.insert("messageTemplate".to_string(), json!(template));
    }
    if let Some(template) = &assertion.fix_template {
        object.insert("fixTemplate".to_string(), json!(template));
    }
    Value::Object(object)
}

fn severity_label(severity: OrgContractSeverity) -> &'static str {
    severity.as_str()
}

fn status_label(status: OrgContractAssertionStatus) -> &'static str {
    match status {
        OrgContractAssertionStatus::Passed => "passed",
        OrgContractAssertionStatus::Failed => "failed",
    }
}

fn expectation_to_json_value(expectation: &OrgContractExpectation) -> Value {
    match expectation {
        OrgContractExpectation::Exists => json!({
            "kind": "exists",
        }),
        OrgContractExpectation::NotExists => json!({
            "kind": "notExists",
        }),
        OrgContractExpectation::Count(operator, count) => json!({
            "kind": "count",
            "operator": operator.as_str(),
            "count": count,
        }),
    }
}

fn bindings_to_json_value(
    bindings: &std::collections::BTreeMap<String, Vec<OrgElementId>>,
) -> Value {
    Value::Object(
        bindings
            .iter()
            .map(|(name, ids)| (name.clone(), ids_to_json_value(ids)))
            .collect(),
    )
}

fn ids_to_json_value(ids: &[OrgElementId]) -> Value {
    Value::Array(ids.iter().map(|id| json!(id.as_usize())).collect())
}
