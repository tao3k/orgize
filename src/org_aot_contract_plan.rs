//! Scheme-AOT Org Contract pack mounted in the Cargo library.

use crate::contract_feature::{
    ContractAssertionRule, ContractBindingRule, ContractExpectationRule, ContractOperator,
    ContractPack, ContractQueryRule, ContractRelation, ContractRule, ContractScope,
    ContractSeverity,
};
use crate::org_element_query::{OrgElementFieldMatch, OrgElementPropertyRule};

include!("../languages/org/v1/modules/org-contract/generated/contract-plan.rs");
