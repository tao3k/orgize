//! Typed admission only; Scheme owns block roles, parameters and cooked forms.
use crate::ast::{OrgContractKind, OrgContractScope, OrgContractSeverity, SourceBlockSyntaxRecord};
use crate::org_aot::NativeExpressionValue;

pub(crate) struct ContractBlock {
    record: SourceBlockSyntaxRecord,
    pub(super) role: String,
    pub(super) normalized_name: String,
    pub(super) named_id: Option<String>,
    pub(super) parameter_name: Option<String>,
    pub(super) severity: Option<OrgContractSeverity>,
    pub(super) query_forms: Option<Vec<NativeExpressionValue>>,
    pub(super) contract_forms: Option<Vec<NativeExpressionValue>>,
    pub(super) expect_forms: Option<Vec<NativeExpressionValue>>,
}
impl std::ops::Deref for ContractBlock {
    type Target = SourceBlockSyntaxRecord;
    fn deref(&self) -> &Self::Target {
        &self.record
    }
}
pub(super) fn optional(present: &str, value: String) -> Option<String> {
    match present {
        "true" => Some(value),
        "false" => {
            assert!(value.is_empty(), "absent native source value");
            None
        }
        _ => panic!("native source presence flag"),
    }
}
pub(super) fn kind(value: &str) -> Option<OrgContractKind> {
    match value {
        "org-elements" => Some(OrgContractKind::OrgElementsAssertions),
        "other" => None,
        _ => panic!("native Contract kind"),
    }
}
pub(super) fn scope(value: &str) -> Option<OrgContractScope> {
    match value {
        "document" => Some(OrgContractScope::Document),
        "subtree" => Some(OrgContractScope::Subtree),
        "other" => None,
        _ => panic!("native Contract scope"),
    }
}
pub(super) fn severity(value: &str) -> Option<OrgContractSeverity> {
    match value {
        "error" => Some(OrgContractSeverity::Error),
        "warning" => Some(OrgContractSeverity::Warning),
        "" => None,
        _ => panic!("native Contract severity"),
    }
}
pub(super) fn admit_block(record: SourceBlockSyntaxRecord, row: Vec<String>) -> ContractBlock {
    let [
        role,
        normalized_name,
        named_id,
        name_present,
        parameter_name,
        severity_present,
        parameter_severity,
        parsed_severity,
    ]: [String; 8] = row.try_into().expect("native Contract block arity");
    assert!(
        matches!(
            role.as_str(),
            "query" | "selector" | "expect" | "contract" | "template" | "other"
        ),
        "native Contract block role"
    );
    let _ = optional(&severity_present, parameter_severity);
    ContractBlock {
        record,
        role,
        normalized_name,
        named_id: (!named_id.is_empty()).then_some(named_id),
        parameter_name: optional(&name_present, parameter_name),
        severity: severity(&parsed_severity),
        query_forms: None,
        contract_forms: None,
        expect_forms: None,
    }
}
