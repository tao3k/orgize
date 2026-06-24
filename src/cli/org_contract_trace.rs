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
        CONTRACT_ORG_PROPERTY, Keyword, ORG_ELEMENTS_QUERY_EXPRESSION_EXAMPLES,
        ORG_ELEMENTS_QUERY_EXPRESSION_SURFACE_GUIDE, OrgContract, OrgContractEvaluation,
        OrgContractEvaluationContext, OrgContractEvaluationScope, OrgContractRegistry,
        OrgContractScope, ParsedAnnotation, ParsedAst, Property, Section,
        evaluate_org_contract_with_context, org_contract_evaluations_to_json_value,
        parse_contract_reference,
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
        "query-surface" | "surface" | "guide" => run_query_surface(args.collect()),
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

fn run_query_surface(args: Vec<String>) -> Result<ExitCode, String> {
    let mut json_output = false;

    for arg in args {
        match arg.as_str() {
            "--json" => json_output = true,
            "-h" | "--help" => {
                print_query_surface_usage();
                return Ok(ExitCode::SUCCESS);
            }
            _ if arg.starts_with('-') => {
                return Err(format!("unknown contract query-surface flag `{arg}`"));
            }
            _ => {
                return Err(format!(
                    "contract query-surface does not accept path `{arg}`"
                ));
            }
        }
    }

    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "schemaVersion": 1,
                "kind": "org-elements-query-expression-surface",
                "guide": ORG_ELEMENTS_QUERY_EXPRESSION_SURFACE_GUIDE,
                "examples": ORG_ELEMENTS_QUERY_EXPRESSION_EXAMPLES,
            }))
            .expect("contract query-surface JSON should serialize")
        );
    } else {
        println!("orgize contract query-surface");
        println!("guide:");
        for entry in ORG_ELEMENTS_QUERY_EXPRESSION_SURFACE_GUIDE {
            println!("- {entry}");
        }
        println!("examples:");
        for example in ORG_ELEMENTS_QUERY_EXPRESSION_EXAMPLES {
            println!("- {example}");
        }
    }

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
    let context = OrgContractEvaluationContext::with_source_path(path);
    let document_contracts =
        resolve_bindings(document_contract_bindings(document), registry, path)?;
    let mut document_default_contracts = Vec::new();
    for contract in document_contracts {
        if contract.scope == OrgContractScope::Document {
            evaluations.push(evaluate_org_contract_with_context(
                document,
                contract,
                OrgContractEvaluationScope::document(),
                &context,
            ));
        } else if contract.scope == OrgContractScope::Subtree {
            document_default_contracts.push(contract);
        }
    }

    {
        let mut collector = SectionContractEvaluationCollector {
            document,
            registry,
            path,
            context: &context,
            evaluations: &mut evaluations,
        };
        for section in &document.sections {
            collector.collect(section, Vec::new(), &document_default_contracts)?;
        }
    }
    Ok(evaluations)
}

struct SectionContractEvaluationCollector<'a> {
    document: &'a ParsedAst,
    registry: &'a OrgContractRegistry,
    path: &'a str,
    context: &'a OrgContractEvaluationContext,
    evaluations: &'a mut Vec<OrgContractEvaluation>,
}

impl<'a> SectionContractEvaluationCollector<'a> {
    fn collect(
        &mut self,
        section: &Section<ParsedAnnotation>,
        mut outline_path: Vec<String>,
        inherited_contracts: &[&'a OrgContract],
    ) -> Result<(), String> {
        outline_path.push(section.raw_title.trim_end().to_string());
        let section_bindings = section_contract_bindings(section);
        let section_contracts = if section_bindings.is_empty() {
            inherited_contracts.to_vec()
        } else {
            resolve_bindings(section_bindings, self.registry, self.path)?
                .into_iter()
                .filter(|contract| contract.scope == OrgContractScope::Subtree)
                .collect()
        };

        for contract in section_contracts {
            self.evaluations.push(evaluate_org_contract_with_context(
                self.document,
                contract,
                OrgContractEvaluationScope::section(
                    section.raw_title.trim_end(),
                    outline_path.clone(),
                    section.ann.range,
                ),
                self.context,
            ));
        }

        for child in &section.subsections {
            self.collect(child, outline_path.clone(), &[])?;
        }
        Ok(())
    }
}

fn resolve_binding<'a>(
    binding: ContractBinding,
    registry: &'a OrgContractRegistry,
    path: &str,
) -> Result<Option<&'a OrgContract>, String> {
    if binding.reference.raw.trim().is_empty() {
        return Err(format!("{path}: CONTRACT_ORG is empty"));
    }
    let reference = binding
        .reference
        .with_source_relative_path(Some(Path::new(path)));
    registry
        .resolve(&reference)
        .or_else(|| registry.resolve(&binding.reference))
        .map(Some)
        .ok_or_else(|| {
            format!(
                "{path}: CONTRACT_ORG `{}` was not found in the loaded Org contract registry",
                binding.reference.raw
            )
        })
}

fn resolve_bindings<'a>(
    bindings: Vec<ContractBinding>,
    registry: &'a OrgContractRegistry,
    path: &str,
) -> Result<Vec<&'a OrgContract>, String> {
    bindings
        .into_iter()
        .map(|binding| resolve_binding(binding, registry, path))
        .collect::<Result<Vec<_>, _>>()
        .map(|contracts| contracts.into_iter().flatten().collect())
}

fn document_contract_bindings(document: &ParsedAst) -> Vec<ContractBinding> {
    let properties = property_contract_bindings(&document.properties);
    if properties.is_empty() {
        keyword_contract_bindings(&document.metadata)
    } else {
        properties
    }
}

fn section_contract_bindings(section: &Section<ParsedAnnotation>) -> Vec<ContractBinding> {
    property_contract_bindings(&section.properties)
}

fn property_contract_bindings(properties: &[Property<ParsedAnnotation>]) -> Vec<ContractBinding> {
    properties
        .iter()
        .filter(|property| property.key.eq_ignore_ascii_case(CONTRACT_ORG_PROPERTY))
        .map(|property| ContractBinding {
            reference: parse_contract_reference(property.value.as_str()),
            range: property.ann.range,
        })
        .collect()
}

fn keyword_contract_bindings(keywords: &[Keyword<ParsedAnnotation>]) -> Vec<ContractBinding> {
    keywords
        .iter()
        .filter(|keyword| keyword.key.eq_ignore_ascii_case(CONTRACT_ORG_PROPERTY))
        .map(|keyword| ContractBinding {
            reference: parse_contract_reference(keyword.value.as_str()),
            range: keyword.ann.range,
        })
        .collect()
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
    eprintln!("Usage: orgize contract <trace|query-surface> [options] [PATH ...]");
}

fn print_trace_usage() {
    eprintln!(
        "Usage: orgize contract trace [--json] --org-contract-registry PATH.org [--org-contract-registry PATH.org ...] [PATH ...]"
    );
}

fn print_query_surface_usage() {
    eprintln!("Usage: orgize contract query-surface [--json]");
}
