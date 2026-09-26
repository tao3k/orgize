//! Scheme-AOT Org Contract pack mounted in the Cargo library.

use crate::contract_feature::{
    ContractAssertionRule, ContractBindingRule, ContractExpectationRule, ContractFieldMatch,
    ContractOperator, ContractPack, ContractQueryRule, ContractRelation, ContractRule,
    ContractScope, ContractSeverity,
};

include!("../languages/org/v1/modules/org-contract/generated/contract-plan.rs");
