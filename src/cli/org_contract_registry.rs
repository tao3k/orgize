//! CLI loading for host-owned `CONTRACT_ORG` registries.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    Org,
    ast::{
        CONTRACT_ORG_PROPERTY, OrgContractReference, OrgContractRegistry,
        parse_contract_references, parse_contracts_from_document,
    },
};

pub(super) fn load_org_contract_registries(
    paths: &[PathBuf],
) -> Result<OrgContractRegistry, String> {
    Ok(load_org_contract_registries_with_sources(paths)?.0)
}

pub(super) fn load_org_contract_registries_with_sources(
    paths: &[PathBuf],
) -> Result<(OrgContractRegistry, Vec<(PathBuf, String)>), String> {
    let mut loader = OrgContractRegistryLoader::default();
    for path in paths {
        loader.load_path(path, true)?;
    }
    Ok((loader.registry, loader.loaded_sources))
}

pub(super) fn load_org_contract_registry_for_lint(
    explicit_registry_paths: &[PathBuf],
    lint_source_paths: &[PathBuf],
) -> Result<OrgContractRegistry, String> {
    let mut loader = OrgContractRegistryLoader::default();
    for path in explicit_registry_paths {
        loader.load_path(path, true)?;
    }
    for path in lint_source_paths {
        loader.load_path(path, false)?;
    }
    Ok(loader.registry)
}

#[derive(Default)]
struct OrgContractRegistryLoader {
    registry: OrgContractRegistry,
    loaded_paths: BTreeSet<PathBuf>,
    loaded_sources: Vec<(PathBuf, String)>,
}

impl OrgContractRegistryLoader {
    fn load_path(&mut self, path: &Path, load_dependencies: bool) -> Result<(), String> {
        let load_key = path
            .canonicalize()
            .unwrap_or_else(|_| normalize_lexical_path(path));
        if !self.loaded_paths.insert(load_key.clone()) {
            return Ok(());
        }

        let source =
            fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let document = Org::parse(&source).document();
        let loaded = parse_contracts_from_document(&document, Some(path));
        let dependency_paths = if load_dependencies {
            registry_dependency_paths(path, &document)?
        } else {
            Vec::new()
        };
        self.registry.contracts.extend(loaded.contracts);
        self.loaded_sources.push((load_key, source));

        for dependency_path in dependency_paths {
            self.load_path(&dependency_path, true)?;
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
        .flat_map(|property| parse_contract_references(property.value.as_str()))
        .map(|reference| registry_dependency_path(source_path, reference))
        .collect()
}

fn registry_dependency_path(
    source_path: &Path,
    reference: OrgContractReference,
) -> Result<PathBuf, String> {
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
