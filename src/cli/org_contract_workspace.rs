//! Workspace-level admission for Org repositories governed by an Org policy.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use serde_json::json;

use crate::{
    Org,
    ast::{
        BlockKind, ElementData, OrgContractAssertionStatus, OrgContractPairDocumentEquality,
        OrgContractPairNodeEquality, OrgContractWorkspaceReference, ParsedAst, Property, Section,
        org_contract_evaluations_to_json_value, parse_contract_references,
        parse_org_contract_pair_document_equality_block,
        parse_org_contract_pair_node_equality_block, parse_org_contract_workspace_reference_block,
    },
};

const CONTRACT_PROPERTY: &str = "CONTRACT_ORG";
const SEMANTIC_ID_PROPERTY: &str = "SEMANTIC_ID";
const COUNTERPART_PROPERTY: &str = "COUNTERPART";
const LANGUAGE_PROPERTY: &str = "LANGUAGE";

pub(crate) fn run(args: Vec<String>) -> Result<ExitCode, String> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-h" | "--help"))
    {
        print_usage();
        return Ok(ExitCode::SUCCESS);
    }
    let options = WorkspaceOptions::parse(args)?;
    let root = options
        .root
        .canonicalize()
        .map_err(|error| format!("{}: {error}", options.root.display()))?;
    let policy_path = options
        .policy
        .canonicalize()
        .map_err(|error| format!("{}: {error}", options.policy.display()))?;
    let policy_source = fs::read_to_string(&policy_path)
        .map_err(|error| format!("{}: {error}", policy_path.display()))?;
    let policy = WorkspacePolicy::parse(&policy_source)?;
    if let Some(required) = options.require_maintained.as_ref() {
        let target = if required.is_absolute() {
            required.clone()
        } else {
            root.join(required)
        }
        .canonicalize()
        .map_err(|error| format!("{}: {error}", required.display()))?;
        let relative = relative_path(&root, &target)?;
        let routes = policy.matching_routes(&relative);
        if routes.len() != 1 || policy.routes[routes[0]].role != RouteRole::Maintained {
            return Err(format!(
                "{relative}: required trace target must match exactly one maintained workspace route"
            ));
        }
    }
    let (registry, registry_sources) =
        super::org_contract_registry::load_org_contract_registries_with_sources(
            &options.registry_paths,
        )?;

    let mut discovered = Vec::new();
    collect_org_files(&root, &root, &policy.ignored_dirs, &mut discovered)?;
    discovered.sort();
    let discovered_sources = discovered
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(&path)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            Ok((path, source))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let mut findings = Vec::new();
    let mut admissions = Vec::with_capacity(discovered_sources.len());
    for (path, source) in &discovered_sources {
        let relative = relative_path(&root, path)?;
        let matching_routes = policy.matching_routes(&relative);
        if matching_routes.len() != 1 {
            findings.push(format!(
                "{relative}: expected exactly one workspace route, matched {}",
                matching_routes.len()
            ));
            continue;
        }
        let route_index = matching_routes[0];
        let route = &policy.routes[route_index];
        if route.role == RouteRole::Support {
            continue;
        }
        admissions.push(MaintainedAdmission {
            path: path.clone(),
            relative,
            route_index,
            source: source.clone(),
        });
    }
    let maintained = load_maintained_documents(&admissions)?;
    for document in &maintained {
        if document.diagnostic_count != 0 {
            findings.push(format!(
                "{relative}: parser produced {} diagnostic(s)",
                document.diagnostic_count,
                relative = document.relative,
            ));
        }
    }

    let mut paired = Vec::with_capacity(maintained.len());
    let mut file_receipts = Vec::new();
    let mut evaluation_count = 0_usize;
    let mut assertion_count = 0_usize;

    for item in &maintained {
        let route = &policy.routes[item.route_index];
        let contract_values = property_values(&item.document.properties, CONTRACT_PROPERTY);
        if contract_values.len() != 1 {
            findings.push(format!(
                "{}: expected exactly one {CONTRACT_PROPERTY} property, found {}",
                item.relative,
                contract_values.len()
            ));
        } else {
            let actual = parse_contract_references(contract_values[0])
                .into_iter()
                .map(|reference| reference.contract_id.unwrap_or(reference.raw))
                .collect::<Vec<_>>();
            if actual != route.contracts {
                findings.push(format!(
                    "{}: exact contract composition must be [{}], found [{}]",
                    item.relative,
                    route.contracts.join(" -> "),
                    actual.join(" -> ")
                ));
            }
        }

        if let Some(pair) = route.pair.as_ref() {
            let semantic_id = single_property(
                &item.document.properties,
                SEMANTIC_ID_PROPERTY,
                &item.relative,
                &mut findings,
            );
            let counterpart = single_property(
                &item.document.properties,
                COUNTERPART_PROPERTY,
                &item.relative,
                &mut findings,
            );
            let language = single_property(
                &item.document.properties,
                LANGUAGE_PROPERTY,
                &item.relative,
                &mut findings,
            );
            if language.as_deref() != Some(pair.language_value.as_str()) {
                findings.push(format!(
                    "{}: LANGUAGE must be `{}`",
                    item.relative, pair.language_value
                ));
            }
            if let (Some(semantic_id), Some(counterpart)) = (semantic_id, counterpart) {
                let node_identities = pair
                    .node_identity_property
                    .as_deref()
                    .map(|key| {
                        let mut values = Vec::new();
                        collect_section_property_values(&item.document.sections, key, &mut values);
                        values
                    })
                    .unwrap_or_default();
                let mut node_metadata = BTreeMap::new();
                if let Some(rule) = pair.node_equality.as_ref() {
                    collect_node_metadata(
                        &item.document.sections,
                        rule,
                        &item.relative,
                        &mut node_metadata,
                        &mut findings,
                    );
                }
                let mut document_metadata = BTreeMap::new();
                if let Some(rule) = pair.document_equality.as_ref() {
                    for key in &rule.properties {
                        let values = property_values(&item.document.properties, key);
                        if values.len() != 1 || values[0].is_empty() {
                            findings.push(format!(
                                "{}: paired document must declare one non-empty {key} property; found {}",
                                item.relative,
                                values.len()
                            ));
                            continue;
                        }
                        document_metadata.insert(key.clone(), values[0].to_string());
                    }
                }
                paired.push(PairedDocument {
                    relative: item.relative.clone(),
                    path: item.path.clone(),
                    semantic_id,
                    counterpart,
                    pair: pair.clone(),
                    node_identities,
                    node_metadata,
                    document_metadata,
                });
            }
        }

        let evaluations = super::org_contract_trace::collect_contract_evaluations(
            &item.document,
            &registry,
            &item.path.display().to_string(),
        )?;
        evaluation_count += evaluations.len();
        assertion_count += evaluations
            .iter()
            .map(|evaluation| evaluation.assertions.len())
            .sum::<usize>();
        for evaluation in &evaluations {
            for assertion in &evaluation.assertions {
                if assertion.status == OrgContractAssertionStatus::Failed {
                    findings.push(format!(
                        "{}: {} / {} => failed",
                        item.relative, evaluation.contract_id, assertion.assertion_id
                    ));
                }
            }
        }
        if options.json {
            file_receipts.push(json!({
                "path": item.relative,
                "route": route.name,
                "evaluations": org_contract_evaluations_to_json_value(&evaluations),
            }));
        }
    }

    super::org_contract_workspace_validation::validate_pairs(&root, &paired, &mut findings);
    super::org_contract_workspace_validation::validate_references(
        &maintained,
        &policy,
        &mut findings,
    );

    if options.json {
        let workspace_digest = workspace_digest(
            &root,
            (&policy_path, &policy_source),
            &registry_sources,
            &discovered_sources,
            &maintained,
        )?;
        let mut receipt = json!({
            "schemaVersion": 1,
            "workspaceContractId": policy.id,
            "status": if findings.is_empty() { "passed" } else { "failed" },
            "documentCount": maintained.len(),
            "evaluationCount": evaluation_count,
            "assertionCount": assertion_count,
            "orgizeRevision": env!("ORGIZE_SOURCE_REVISION"),
            "orgizeSourceDirty": env!("ORGIZE_SOURCE_DIRTY") == "true",
            "workspaceDigest": workspace_digest,
            "findings": findings,
        });
        if !options.summary_json {
            receipt["root"] = json!(root);
            receipt["files"] = json!(file_receipts);
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&receipt)
                .expect("workspace contract receipt should serialize")
        );
    } else if findings.is_empty() {
        println!(
            "orgize contract workspace: {} documents, {} evaluations, {} assertions passed",
            maintained.len(),
            evaluation_count,
            assertion_count
        );
    } else {
        for finding in &findings {
            eprintln!("orgize: {finding}");
        }
    }

    Ok(if findings.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn workspace_digest(
    root: &Path,
    policy: (&Path, &str),
    registries: &[(PathBuf, String)],
    inventory: &[(PathBuf, String)],
    documents: &[MaintainedDocument],
) -> Result<String, String> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"orgize.workspace-receipt.v1\0");
    hash_receipt_source(
        &mut hasher,
        "policy",
        &receipt_source_label(root, policy.0),
        policy.1,
    );
    for (path, source) in registries {
        hash_receipt_source(
            &mut hasher,
            "registry",
            &receipt_source_label(root, path),
            source,
        );
    }
    for (path, source) in inventory {
        hash_receipt_source(
            &mut hasher,
            "inventory",
            &receipt_source_label(root, path),
            source,
        );
    }
    for document in documents {
        hash_receipt_source(
            &mut hasher,
            "document",
            &receipt_source_label(root, &document.path),
            &document.source,
        );
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn hash_receipt_source(hasher: &mut blake3::Hasher, role: &str, label: &str, source: &str) {
    hasher.update(&(role.len() as u64).to_be_bytes());
    hasher.update(role.as_bytes());
    hasher.update(&(label.len() as u64).to_be_bytes());
    hasher.update(label.as_bytes());
    hasher.update(&(source.len() as u64).to_be_bytes());
    hasher.update(source.as_bytes());
}

fn receipt_source_label(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(path_to_policy_string)
        .unwrap_or_else(|_| format!("external:{}", path_to_policy_string(path)))
}

#[derive(Debug)]
struct WorkspaceOptions {
    root: PathBuf,
    policy: PathBuf,
    registry_paths: Vec<PathBuf>,
    require_maintained: Option<PathBuf>,
    json: bool,
    summary_json: bool,
}

impl WorkspaceOptions {
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut root = None;
        let mut policy = None;
        let mut registry_paths = Vec::new();
        let mut require_maintained = None;
        let mut json = false;
        let mut summary_json = false;
        let mut index = 0;
        while index < args.len() {
            match args[index].as_str() {
                "--root" => root = Some(next_path(&args, &mut index, "--root")?),
                "--policy" => policy = Some(next_path(&args, &mut index, "--policy")?),
                "--org-contract-registry" => {
                    registry_paths.push(next_path(&args, &mut index, "--org-contract-registry")?)
                }
                "--require-maintained" => {
                    require_maintained = Some(next_path(&args, &mut index, "--require-maintained")?)
                }
                "--json" => json = true,
                "--summary-json" => {
                    json = true;
                    summary_json = true;
                }
                flag if flag.starts_with('-') => {
                    return Err(format!("unknown contract workspace flag `{flag}`"));
                }
                path => return Err(format!("contract workspace does not accept path `{path}`")),
            }
            index += 1;
        }
        if registry_paths.is_empty() {
            return Err("contract workspace requires --org-contract-registry PATH.org".to_string());
        }
        Ok(Self {
            root: root.ok_or_else(|| "contract workspace requires --root DIR".to_string())?,
            policy: policy
                .ok_or_else(|| "contract workspace requires --policy PATH.org".to_string())?,
            registry_paths,
            require_maintained,
            json,
            summary_json,
        })
    }
}

fn next_path(args: &[String], index: &mut usize, flag: &str) -> Result<PathBuf, String> {
    *index += 1;
    args.get(*index)
        .map(PathBuf::from)
        .ok_or_else(|| format!("contract workspace {flag} requires a path"))
}

#[derive(Debug)]
pub(super) struct WorkspacePolicy {
    id: String,
    ignored_dirs: BTreeSet<String>,
    pub(super) routes: Vec<WorkspaceRoute>,
    file_routes: BTreeMap<String, usize>,
    directory_routes: BTreeMap<String, usize>,
}

impl WorkspacePolicy {
    fn parse(source: &str) -> Result<Self, String> {
        let document = Org::parse(source).document();
        if !document.diagnostics.is_empty() {
            return Err(format!(
                "workspace policy parser produced {} diagnostic(s)",
                document.diagnostics.len()
            ));
        }
        let id = required_policy_property(&document.properties, "WORKSPACE_CONTRACT_ID")?;
        let ignored_dirs = property_values(&document.properties, "IGNORE_DIRS")
            .into_iter()
            .flat_map(str::split_whitespace)
            .map(str::to_string)
            .collect();
        let mut routes = Vec::new();
        let mut file_routes = BTreeMap::new();
        let mut directory_routes = BTreeMap::new();
        for section in &document.sections {
            let paths = property_values(&section.properties, "PATH");
            if paths.is_empty() {
                continue;
            }
            if paths.len() != 1 {
                return Err(format!(
                    "workspace policy route `{}` must declare one PATH",
                    section.raw_title
                ));
            }
            let route_path = normalize_policy_path(paths[0])?;
            let path_kind =
                match required_policy_property(&section.properties, "PATH_KIND")?.as_str() {
                    "file" => RoutePathKind::File,
                    "directory" => RoutePathKind::Directory,
                    value => {
                        return Err(format!(
                            "workspace policy route `{}` has unsupported PATH_KIND `{value}`",
                            section.raw_title
                        ));
                    }
                };
            let role = match required_policy_property(&section.properties, "ROLE")?.as_str() {
                "maintained" => RouteRole::Maintained,
                "support" => RouteRole::Support,
                value => {
                    return Err(format!(
                        "workspace policy route `{}` has unsupported ROLE `{value}`",
                        section.raw_title
                    ));
                }
            };
            let exact_contract_values = property_values(&section.properties, "CONTRACT_ORG_EXACT");
            if exact_contract_values.len() > 1 {
                return Err(format!(
                    "workspace policy route `{}` must declare CONTRACT_ORG_EXACT exactly once",
                    section.raw_title
                ));
            }
            let contracts = exact_contract_values
                .first()
                .map(|value| {
                    parse_contract_references(value)
                        .into_iter()
                        .map(|reference| reference.contract_id.unwrap_or(reference.raw))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if role == RouteRole::Maintained && contracts.is_empty() {
                return Err(format!(
                    "workspace policy route `{}` must declare CONTRACT_ORG_EXACT",
                    section.raw_title
                ));
            }
            let workspace_rules = section_workspace_rules(section)?;
            let pair_group_values = property_values(&section.properties, "PAIR_GROUP");
            if pair_group_values.len() > 1 {
                return Err(format!(
                    "workspace policy route `{}` must declare PAIR_GROUP at most once",
                    section.raw_title
                ));
            }
            let pair_group = pair_group_values.first().map(|value| (*value).to_string());
            let pair = if let Some(group) = pair_group {
                let node_equality = workspace_rules.node_equality.clone();
                let document_equality = workspace_rules.document_equality.clone();
                let declared_identity =
                    optional_policy_property(&section.properties, "PAIR_NODE_ID_PROPERTY");
                if let (Some(declared), Some(rule)) = (&declared_identity, &node_equality)
                    && declared != &rule.identity_property
                {
                    return Err(format!(
                        "workspace policy route `{}` declares conflicting paired node identity properties",
                        section.raw_title
                    ));
                }
                Some(PairRoute {
                    group,
                    language_root: required_policy_property(&section.properties, "LANGUAGE_ROOT")?,
                    counterpart_root: required_policy_property(
                        &section.properties,
                        "COUNTERPART_ROOT",
                    )?,
                    language_value: required_policy_property(
                        &section.properties,
                        "LANGUAGE_VALUE",
                    )?,
                    node_identity_property: declared_identity.or_else(|| {
                        node_equality
                            .as_ref()
                            .map(|rule| rule.identity_property.clone())
                    }),
                    node_equality,
                    document_equality,
                })
            } else {
                None
            };
            let route_index = routes.len();
            let route_index_by_path = match path_kind {
                RoutePathKind::File => &mut file_routes,
                RoutePathKind::Directory => &mut directory_routes,
            };
            if route_index_by_path
                .insert(route_path.clone(), route_index)
                .is_some()
            {
                return Err(format!(
                    "workspace policy declares duplicate {path_kind:?} PATH `{route_path}`"
                ));
            }
            routes.push(WorkspaceRoute {
                name: section.raw_title.trim().to_string(),
                role,
                contracts,
                pair,
                references: workspace_rules.references,
            });
        }
        if routes.is_empty() {
            return Err("workspace policy contains no PATH routes".to_string());
        }
        Ok(Self {
            id,
            ignored_dirs,
            routes,
            file_routes,
            directory_routes,
        })
    }

    fn matching_routes(&self, relative: &str) -> Vec<usize> {
        let mut matches = Vec::with_capacity(2);
        if let Some(route) = self.file_routes.get(relative) {
            matches.push(*route);
        }
        let parent = Path::new(relative)
            .parent()
            .map(path_to_policy_string)
            .unwrap_or_default();
        if let Some(route) = self.directory_routes.get(&parent) {
            matches.push(*route);
        }
        matches
    }
}

#[derive(Debug)]
pub(super) struct WorkspaceRoute {
    name: String,
    role: RouteRole,
    contracts: Vec<String>,
    pair: Option<PairRoute>,
    pub(super) references: Vec<OrgContractWorkspaceReference>,
}

#[derive(Default)]
struct WorkspaceRules {
    node_equality: Option<OrgContractPairNodeEquality>,
    document_equality: Option<OrgContractPairDocumentEquality>,
    references: Vec<OrgContractWorkspaceReference>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RouteRole {
    Maintained,
    Support,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RoutePathKind {
    File,
    Directory,
}

#[derive(Clone, Debug)]
pub(super) struct PairRoute {
    pub(super) group: String,
    pub(super) language_root: String,
    pub(super) counterpart_root: String,
    language_value: String,
    pub(super) node_identity_property: Option<String>,
    pub(super) node_equality: Option<OrgContractPairNodeEquality>,
    pub(super) document_equality: Option<OrgContractPairDocumentEquality>,
}

pub(super) struct MaintainedDocument {
    path: PathBuf,
    pub(super) relative: String,
    pub(super) route_index: usize,
    pub(super) document: ParsedAst,
    source: String,
    diagnostic_count: usize,
}

struct MaintainedAdmission {
    path: PathBuf,
    relative: String,
    route_index: usize,
    source: String,
}

fn load_maintained_documents(
    admissions: &[MaintainedAdmission],
) -> Result<Vec<MaintainedDocument>, String> {
    if admissions.is_empty() {
        return Ok(Vec::new());
    }
    let worker_count = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(4)
        .min(admissions.len());
    let chunk_size = admissions.len().div_ceil(worker_count);
    std::thread::scope(|scope| {
        admissions
            .chunks(chunk_size)
            .map(|chunk| {
                scope.spawn(move || {
                    chunk
                        .iter()
                        .map(load_maintained_document)
                        .collect::<Result<Vec<_>, _>>()
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .try_fold(
                Vec::with_capacity(admissions.len()),
                |mut documents, task| {
                    documents.extend(
                        task.join()
                            .map_err(|_| "workspace parser worker panicked".to_string())??,
                    );
                    Ok(documents)
                },
            )
    })
}

fn load_maintained_document(admission: &MaintainedAdmission) -> Result<MaintainedDocument, String> {
    let source = admission.source.clone();
    let document = Org::parse(&source).document();
    let diagnostic_count = document.diagnostics.len();
    Ok(MaintainedDocument {
        path: admission.path.clone(),
        relative: admission.relative.clone(),
        route_index: admission.route_index,
        document,
        source,
        diagnostic_count,
    })
}

#[derive(Clone)]
pub(super) struct PairedDocument {
    pub(super) relative: String,
    pub(super) path: PathBuf,
    pub(super) semantic_id: String,
    pub(super) counterpart: String,
    pub(super) pair: PairRoute,
    pub(super) node_identities: Vec<String>,
    pub(super) node_metadata: BTreeMap<String, BTreeMap<String, String>>,
    pub(super) document_metadata: BTreeMap<String, String>,
}

fn collect_org_files(
    root: &Path,
    directory: &Path,
    ignored_dirs: &BTreeSet<String>,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| format!("{}: {error}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("{}: {error}", directory.display()))?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if file_type.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !ignored_dirs.contains(&name) && path != root {
                collect_org_files(root, &path, ignored_dirs, files)?;
            }
        } else if file_type.is_file()
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("org"))
        {
            files.push(path);
        }
    }
    Ok(())
}

fn relative_path(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| format!("{} is outside {}", path.display(), root.display()))?;
    Ok(relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

fn normalize_policy_path(value: &str) -> Result<String, String> {
    if value.contains(['*', '?', '[', ']', '{', '}']) {
        return Err(format!(
            "workspace policy PATH does not accept wildcard syntax: `{value}`"
        ));
    }
    let path = Path::new(value.trim());
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return Err(format!(
            "workspace policy PATH must be a non-empty repository-relative canonical path: `{value}`"
        ));
    }
    Ok(path_to_policy_string(path))
}

fn path_to_policy_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

pub(super) fn property_values<'a, A>(properties: &'a [Property<A>], key: &str) -> Vec<&'a str> {
    properties
        .iter()
        .filter(|property| property.key.eq_ignore_ascii_case(key))
        .map(|property| property.value.trim())
        .collect()
}

pub(super) fn collect_section_property_values<A>(
    sections: &[Section<A>],
    key: &str,
    values: &mut Vec<String>,
) {
    for section in sections {
        values.extend(
            property_values(&section.properties, key)
                .into_iter()
                .filter(|value| !value.is_empty())
                .map(str::to_string),
        );
        collect_section_property_values(&section.subsections, key, values);
    }
}

fn collect_node_metadata<A>(
    sections: &[Section<A>],
    rule: &OrgContractPairNodeEquality,
    path: &str,
    metadata: &mut BTreeMap<String, BTreeMap<String, String>>,
    findings: &mut Vec<String>,
) {
    for section in sections {
        let identities = property_values(&section.properties, &rule.identity_property);
        if identities.len() == 1 && !identities[0].is_empty() {
            let identity = identities[0].to_string();
            let mut values = BTreeMap::new();
            for key in &rule.properties {
                let properties = property_values(&section.properties, key);
                if properties.len() != 1 || properties[0].is_empty() {
                    findings.push(format!(
                        "{path}: paired node `{identity}` must declare one non-empty {key} property; found {}",
                        properties.len()
                    ));
                    continue;
                }
                values.insert(key.clone(), properties[0].to_string());
            }
            metadata.insert(identity, values);
        }
        collect_node_metadata(&section.subsections, rule, path, metadata, findings);
    }
}

fn section_workspace_rules<A>(section: &Section<A>) -> Result<WorkspaceRules, String> {
    let mut node_rules = Vec::new();
    let mut document_rules = Vec::new();
    let mut references = Vec::new();
    for element in &section.children {
        match &element.data {
            ElementData::Block(block)
                if block.kind == BlockKind::Source
                    && block
                        .language
                        .as_deref()
                        .is_some_and(|language| language.eq_ignore_ascii_case("org-contract")) =>
            {
                if let Some(rule) = parse_org_contract_pair_node_equality_block(&block.value) {
                    node_rules.push(rule);
                } else if let Some(rule) =
                    parse_org_contract_pair_document_equality_block(&block.value)
                {
                    document_rules.push(rule);
                } else if let Some(rule) =
                    parse_org_contract_workspace_reference_block(&block.value)
                {
                    references.push(rule);
                } else {
                    return Err(format!(
                        "workspace policy route `{}` contains an unsupported org-contract expression",
                        section.raw_title
                    ));
                }
            }
            _ => {}
        }
    }
    if node_rules.len() > 1 {
        return Err(format!(
            "workspace policy route `{}` declares more than one pair-node equality contract",
            section.raw_title
        ));
    }
    if document_rules.len() > 1 {
        return Err(format!(
            "workspace policy route `{}` declares more than one pair-document equality contract",
            section.raw_title
        ));
    }
    let node_rule = node_rules.into_iter().next();
    if let Some(rule) = &node_rule
        && rule.properties.iter().collect::<BTreeSet<_>>().len() != rule.properties.len()
    {
        return Err(format!(
            "workspace policy route `{}` pair-node equality properties must be unique",
            section.raw_title
        ));
    }
    let document_rule = document_rules.into_iter().next();
    if let Some(rule) = &document_rule
        && rule.properties.iter().collect::<BTreeSet<_>>().len() != rule.properties.len()
    {
        return Err(format!(
            "workspace policy route `{}` pair-document equality properties must be unique",
            section.raw_title
        ));
    }
    Ok(WorkspaceRules {
        node_equality: node_rule,
        document_equality: document_rule,
        references,
    })
}

fn single_property<A>(
    properties: &[Property<A>],
    key: &str,
    path: &str,
    findings: &mut Vec<String>,
) -> Option<String> {
    let values = property_values(properties, key);
    if values.len() != 1 || values[0].is_empty() {
        findings.push(format!(
            "{path}: expected one non-empty {key} property, found {}",
            values.len()
        ));
        return None;
    }
    Some(values[0].to_string())
}

fn required_policy_property<A>(properties: &[Property<A>], key: &str) -> Result<String, String> {
    optional_policy_property(properties, key)
        .ok_or_else(|| format!("workspace policy requires one non-empty {key} property"))
}

fn optional_policy_property<A>(properties: &[Property<A>], key: &str) -> Option<String> {
    let values = property_values(properties, key);
    (values.len() == 1 && !values[0].is_empty()).then(|| values[0].to_string())
}

fn print_usage() {
    eprintln!(
        "Usage: orgize contract workspace --root DIR --policy POLICY.org --org-contract-registry REGISTRY.org [--require-maintained PATH.org] [--json|--summary-json]"
    );
}
