//! CLI loading for host-owned `CONTRACT_ORG` registries.

use std::{fs, path::PathBuf};

use crate::{
    Org,
    ast::{OrgContractRegistry, parse_contracts_from_document},
};

pub(super) fn load_org_contract_registries(
    paths: &[PathBuf],
) -> Result<OrgContractRegistry, String> {
    let mut registry = OrgContractRegistry::default();
    for path in paths {
        let source =
            fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let document = Org::parse(&source).document();
        let loaded = parse_contracts_from_document(&document, Some(path.as_path()));
        registry.contracts.extend(loaded.contracts);
    }
    Ok(registry)
}
