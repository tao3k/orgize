//! CLI loading for host-owned `CONTRACT_ORG` registries.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    Org,
    ast::{
        CONTRACT_ORG_PROPERTY, OrgContractRegistry, parse_contract_reference,
        parse_contracts_from_document,
    },
};

pub(super) fn load_org_contract_registries(
    paths: &[PathBuf],
) -> Result<OrgContractRegistry, String> {
    let mut loader = OrgContractRegistryLoader::default();
    for path in paths {
        loader.load_path(path)?;
    }
    Ok(loader.registry)
}

#[derive(Default)]
struct OrgContractRegistryLoader {
    registry: OrgContractRegistry,
    loaded_paths: BTreeSet<PathBuf>,
}

impl OrgContractRegistryLoader {
    fn load_path(&mut self, path: &Path) -> Result<(), String> {
        let load_key = path
            .canonicalize()
            .unwrap_or_else(|_| normalize_lexical_path(path));
        if !self.loaded_paths.insert(load_key) {
            return Ok(());
        }

        let source =
            fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let document = Org::parse(&source).document();
        let dependency_paths = registry_dependency_paths(path, &document)?;
        let loaded = parse_contracts_from_document(&document, Some(path));
        self.registry.contracts.extend(loaded.contracts);

        for dependency_path in dependency_paths {
            self.load_path(&dependency_path)?;
        }
        Ok(())
    }
}

fn registry_dependency_paths(
    source_path: &Path,
    document: &crate::ast::Document<crate::ast::ParsedAnnotation>,
) -> Result<Vec<PathBuf>, String> {
    if document
        .metadata
        .iter()
        .any(|keyword| keyword.key.eq_ignore_ascii_case(CONTRACT_ORG_PROPERTY))
    {
        return Err(format!(
            "{}: registry CONTRACT_ORG dependencies must be declared in a property drawer, not as #+CONTRACT_ORG metadata",
            source_path.display()
        ));
    }

    document
        .properties
        .iter()
        .filter(|property| property.key.eq_ignore_ascii_case(CONTRACT_ORG_PROPERTY))
        .map(|property| registry_dependency_path(source_path, property.value.as_str()))
        .collect()
}

fn registry_dependency_path(source_path: &Path, value: &str) -> Result<PathBuf, String> {
    let reference = parse_contract_reference(value);
    if reference.raw.trim().is_empty() {
        return Err(format!(
            "{}: registry CONTRACT_ORG dependency is empty",
            source_path.display()
        ));
    }
    let reference = reference.with_source_relative_path(Some(source_path));
    reference.path.map(PathBuf::from).ok_or_else(|| {
        format!(
            "{}: registry CONTRACT_ORG dependency `{}` must include a path",
            source_path.display(),
            reference.raw
        )
    })
}

fn normalize_lexical_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}
