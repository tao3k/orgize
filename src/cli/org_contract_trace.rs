//! CLI trace export for source-backed `CONTRACT_ORG` evaluations.

use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::ExitCode,
};

use rowan::TextRange;
use serde_json::json;

use crate::{
    Org,
    ast::{
        CONTRACT_ORG_PROPERTY, Keyword, OrgContract, OrgContractEvaluation,
        OrgContractEvaluationScope, OrgContractRegistry, OrgContractScope, ParsedAnnotation,
        ParsedAst, Property, Section, evaluate_org_contract,
        org_contract_evaluations_to_json_value, parse_contract_reference,
    },
};

pub(crate) fn run(args: Vec<String>) -> Result<ExitCode, String> {
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        print_usage();
        return Ok(ExitCode::from(2));
    };

    match command.as_str() {
        "trace" => run_trace(args.collect()),
        "-h" | "--help" | "help" => {
            print_usage();
            Ok(ExitCode::SUCCESS)
        }
        command => Err(format!("contract: unsupported command `{command}`")),
    }
}

fn run_trace(args: Vec<String>) -> Result<ExitCode, String> {
    let mut registry_paths = Vec::new();
    let mut paths = Vec::new();
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--json" => {}
            "--org-contract-registry" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(
                        "contract trace --org-contract-registry requires an Org path".to_string(),
                    );
                };
                registry_paths.push(PathBuf::from(value));
            }
            "-h" | "--help" => {
                print_trace_usage();
                return Ok(ExitCode::SUCCESS);
            }
            _ if arg.starts_with('-') => {
                return Err(format!("unknown contract trace flag `{arg}`"));
            }
            _ => paths.push(arg.clone()),
        }
        index += 1;
    }

    if registry_paths.is_empty() {
        return Err("contract trace requires --org-contract-registry PATH.org".to_string());
    }

    let registry = super::org_contract_registry::load_org_contract_registries(&registry_paths)?;
    let mut files = Vec::new();
    if paths.is_empty() {
        let source = read_stdin()?;
        files.push(trace_file("<stdin>", &source, &registry)?);
    } else {
        for path in collect_org_paths(&paths)? {
            let display_path = path.display().to_string();
            let source =
                fs::read_to_string(&path).map_err(|error| format!("{display_path}: {error}"))?;
            files.push(trace_file(&display_path, &source, &registry)?);
        }
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schemaVersion": 1,
            "files": files,
        }))
        .expect("contract trace JSON should serialize")
    );
    Ok(ExitCode::SUCCESS)
}

fn trace_file(
    path: &str,
    source: &str,
    registry: &OrgContractRegistry,
) -> Result<serde_json::Value, String> {
    let document = Org::parse(source).document();
    let evaluations = collect_contract_evaluations(&document, registry, path)?;
    Ok(json!({
        "path": path,
        "evaluations": org_contract_evaluations_to_json_value(&evaluations),
    }))
}

fn collect_contract_evaluations(
    document: &ParsedAst,
    registry: &OrgContractRegistry,
    path: &str,
) -> Result<Vec<OrgContractEvaluation>, String> {
    let mut evaluations = Vec::new();
    let document_contract = document_contract_binding(document)
        .map(|binding| resolve_binding(binding, registry, path))
        .transpose()?
        .flatten();
    let document_default_contract = match document_contract {
        Some(contract) if contract.scope == OrgContractScope::Document => {
            evaluations.push(evaluate_org_contract(
                document,
                contract,
                OrgContractEvaluationScope::document(),
            ));
            None
        }
        Some(contract) if contract.scope == OrgContractScope::Subtree => Some(contract),
        _ => None,
    };

    for section in &document.sections {
        collect_section_contract_evaluations(
            document,
            registry,
            path,
            section,
            Vec::new(),
            document_default_contract,
            &mut evaluations,
        )?;
    }
    Ok(evaluations)
}

fn collect_section_contract_evaluations<'a>(
    document: &ParsedAst,
    registry: &'a OrgContractRegistry,
    path: &str,
    section: &Section<ParsedAnnotation>,
    mut outline_path: Vec<String>,
    inherited_contract: Option<&'a OrgContract>,
    evaluations: &mut Vec<OrgContractEvaluation>,
) -> Result<(), String> {
    outline_path.push(section.raw_title.trim_end().to_string());
    let section_contract = match section_contract_binding(section) {
        Some(binding) => resolve_binding(binding, registry, path)?
            .filter(|contract| contract.scope == OrgContractScope::Subtree),
        None => inherited_contract,
    };

    if let Some(contract) = section_contract {
        evaluations.push(evaluate_org_contract(
            document,
            contract,
            OrgContractEvaluationScope::section(
                section.raw_title.trim_end(),
                outline_path.clone(),
                section.ann.range,
            ),
        ));
    }

    for child in &section.subsections {
        collect_section_contract_evaluations(
            document,
            registry,
            path,
            child,
            outline_path.clone(),
            None,
            evaluations,
        )?;
    }
    Ok(())
}

fn resolve_binding<'a>(
    binding: ContractBinding,
    registry: &'a OrgContractRegistry,
    path: &str,
) -> Result<Option<&'a OrgContract>, String> {
    if binding.reference.raw.trim().is_empty() {
        return Err(format!("{path}: CONTRACT_ORG is empty"));
    }
    registry
        .resolve(&binding.reference)
        .map(Some)
        .ok_or_else(|| {
            format!(
                "{path}: CONTRACT_ORG `{}` was not found in the loaded Org contract registry",
                binding.reference.raw
            )
        })
}

fn document_contract_binding(document: &ParsedAst) -> Option<ContractBinding> {
    property_contract_binding(&document.properties)
        .or_else(|| keyword_contract_binding(&document.metadata))
}

fn section_contract_binding(section: &Section<ParsedAnnotation>) -> Option<ContractBinding> {
    property_contract_binding(&section.properties)
}

fn property_contract_binding(properties: &[Property<ParsedAnnotation>]) -> Option<ContractBinding> {
    properties
        .iter()
        .rev()
        .find(|property| property.key.eq_ignore_ascii_case(CONTRACT_ORG_PROPERTY))
        .map(|property| ContractBinding {
            reference: parse_contract_reference(property.value.as_str()),
            range: property.ann.range,
        })
}

fn keyword_contract_binding(keywords: &[Keyword<ParsedAnnotation>]) -> Option<ContractBinding> {
    keywords
        .iter()
        .rev()
        .find(|keyword| keyword.key.eq_ignore_ascii_case(CONTRACT_ORG_PROPERTY))
        .map(|keyword| ContractBinding {
            reference: parse_contract_reference(keyword.value.as_str()),
            range: keyword.ann.range,
        })
}

#[derive(Clone, Debug)]
struct ContractBinding {
    reference: crate::ast::OrgContractReference,
    #[allow(dead_code)]
    range: TextRange,
}

fn read_stdin() -> Result<String, String> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| format!("failed to read stdin: {error}"))?;
    Ok(input)
}

fn collect_org_paths(paths: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for path in paths {
        collect_org_path(Path::new(path), &mut files)?;
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn collect_org_path(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if metadata.is_file() {
        if !is_org_file(path) {
            return Err(format!("{}: expected .org file", path.display()));
        }
        files.push(path.to_path_buf());
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(format!("{}: unsupported path type", path.display()));
    }

    let mut entries = fs::read_dir(path)
        .map_err(|error| format!("{}: {error}", path.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let entry_path = entry.path();
        let entry_type = entry
            .file_type()
            .map_err(|error| format!("{}: {error}", entry_path.display()))?;
        if entry_type.is_dir() {
            collect_org_path(&entry_path, files)?;
        } else if entry_type.is_file() && is_org_file(&entry_path) {
            files.push(entry_path);
        }
    }
    Ok(())
}

fn is_org_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("org"))
}

fn print_usage() {
    eprintln!("Usage: orgize contract <trace> [options] [PATH ...]");
}

fn print_trace_usage() {
    eprintln!("Usage: orgize contract trace [--json] --org-contract-registry PATH.org [PATH ...]");
}
