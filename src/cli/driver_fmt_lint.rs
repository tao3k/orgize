//! Format, lint, and property-schema command handlers.

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use crate::{
    ast::{
        PriorityProfile, PriorityValue, PropertySchemaContract, PropertySchemaField,
        PropertySchemaRegistry, PropertySchemaValueRule,
    },
    fmt::{FormatOptions, format_org},
    lint::{LintOptions, lint_org_with_options},
};

use super::driver_paths::{collect_org_paths, display_path, format_path_error, read_stdin};
use super::driver_usage::{print_fmt_usage, print_lint_usage};

pub(crate) fn run_fmt(args: Vec<String>) -> Result<ExitCode, String> {
    let mut check = false;
    let mut paths = Vec::new();

    for arg in args {
        match arg.as_str() {
            "--check" => check = true,
            "--write" | "-w" => {}
            "-h" | "--help" => {
                print_fmt_usage();
                return Ok(ExitCode::SUCCESS);
            }
            _ if arg.starts_with('-') => return Err(format!("unknown fmt flag `{arg}`")),
            _ => paths.push(arg),
        }
    }

    let options = FormatOptions::default();
    let mut changed = false;

    if paths.is_empty() {
        let source = read_stdin()?;
        let formatted = format_org(&source, &options);
        changed |= formatted.changed;
        if check {
            if formatted.changed {
                eprintln!("<stdin>: needs formatting");
            }
        } else {
            print!("{}", formatted.output);
        }
    } else {
        for path in collect_org_paths(&paths)? {
            let display_path = display_path(&path);
            let source =
                fs::read_to_string(&path).map_err(|error| format_path_error(&path, error))?;
            let formatted = format_org(&source, &options);
            changed |= formatted.changed;
            if check {
                if formatted.changed {
                    eprintln!("{display_path}: needs formatting");
                }
            } else {
                if formatted.changed {
                    fs::write(&path, formatted.output)
                        .map_err(|error| format_path_error(&path, error))?;
                }
            }
        }
    }

    Ok(if check && changed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

pub(crate) fn run_lint(args: Vec<String>) -> Result<ExitCode, String> {
    let mut output_format = LintOutputFormat::Compact;
    let mut priority_highest = None;
    let mut priority_lowest = None;
    let mut priority_default = None;
    let mut property_schema_registry_paths = Vec::new();
    let mut org_contract_registry_paths = Vec::new();
    let mut fix = false;
    let mut paths = Vec::new();
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--format" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("lint --format requires `compact`, `text`, or `json`".to_string());
                };
                output_format = LintOutputFormat::parse(value)?;
            }
            "--json" => output_format = LintOutputFormat::Json,
            "--priority-highest" => {
                index += 1;
                priority_highest = Some(parse_priority_flag(&args, index, "--priority-highest")?);
            }
            "--priority-lowest" => {
                index += 1;
                priority_lowest = Some(parse_priority_flag(&args, index, "--priority-lowest")?);
            }
            "--priority-default" => {
                index += 1;
                priority_default = Some(parse_priority_flag(&args, index, "--priority-default")?);
            }
            "--property-schema-registry" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("lint --property-schema-registry requires a JSON path".to_string());
                };
                property_schema_registry_paths.push(PathBuf::from(value));
            }
            "--org-contract-registry" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("lint --org-contract-registry requires an Org path".to_string());
                };
                org_contract_registry_paths.push(PathBuf::from(value));
            }
            "--fix" => fix = true,
            "-h" | "--help" => {
                print_lint_usage();
                return Ok(ExitCode::SUCCESS);
            }
            _ if arg.starts_with('-') => return Err(format!("unknown lint flag `{arg}`")),
            _ => paths.push(arg.clone()),
        }
        index += 1;
    }

    let priority_profile =
        priority_profile_from_flags(priority_highest, priority_lowest, priority_default)?;
    let property_schema_registry =
        load_property_schema_registries(&property_schema_registry_paths)?;
    let lint_paths = if paths.is_empty() {
        Vec::new()
    } else {
        collect_org_paths(&paths)?
    };
    let org_contract_registry = super::org_contract_registry::load_org_contract_registry_for_lint(
        &org_contract_registry_paths,
        &lint_paths,
    )?;
    let base_lint_options = LintOptions {
        priority_profile,
        property_schema_registry,
        org_contract_registry,
        ..LintOptions::default()
    };

    let mut reports = Vec::new();
    if paths.is_empty() {
        if fix {
            return Err("lint --fix requires at least one Org file or directory path".to_string());
        }
        let source = read_stdin()?;
        let report = lint_org_with_options(&source, &base_lint_options);
        reports.push(LintFileReport {
            path: "<stdin>".to_string(),
            source,
            report,
        });
    } else {
        for path in lint_paths {
            let display_path = display_path(&path);
            let mut source =
                fs::read_to_string(&path).map_err(|error| format_path_error(&path, error))?;
            if fix {
                let formatted = format_org(&source, &FormatOptions::default());
                if formatted.changed {
                    fs::write(&path, &formatted.output)
                        .map_err(|error| format_path_error(&path, error))?;
                    source = formatted.output;
                }
            }
            let lint_options = LintOptions {
                source_path: Some(path.clone()),
                include_base_dir: path.parent().map(Path::to_path_buf),
                attachment_base_dir: path.parent().map(Path::to_path_buf),
                file_base_dir: path.parent().map(Path::to_path_buf),
                ..base_lint_options.clone()
            };
            let report = lint_org_with_options(&source, &lint_options);
            reports.push(LintFileReport {
                path: display_path,
                source,
                report,
            });
        }
    }

    let has_findings = reports.iter().any(|file| !file.report.is_clean());
    match output_format {
        LintOutputFormat::Compact => {
            print!("{}", render_lint_compact(&reports));
        }
        LintOutputFormat::Text => {
            for file in &reports {
                print!("{}", file.report.to_text(&file.path));
            }
        }
        LintOutputFormat::Json => {
            print!("{{\"files\":[");
            for (index, file) in reports.iter().enumerate() {
                if index > 0 {
                    print!(",");
                }
                print!("{}", file.report.to_json_file(&file.path));
            }
            println!("]}}");
        }
    }

    Ok(if has_findings {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

struct LintFileReport {
    path: String,
    source: String,
    report: crate::lint::LintReport,
}

fn render_lint_compact(reports: &[LintFileReport]) -> String {
    let rendered = reports
        .iter()
        .filter(|file| !file.report.is_clean())
        .map(|file| file.report.to_compact_text(&file.path, &file.source))
        .collect::<Vec<_>>();

    if rendered.is_empty() {
        "[ok] orgize lint\n".to_string()
    } else {
        rendered.join("\n")
    }
}

#[derive(Clone, Copy)]
enum LintOutputFormat {
    Compact,
    Text,
    Json,
}

impl LintOutputFormat {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "compact" => Ok(Self::Compact),
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            _ => Err(format!("unsupported lint output format `{value}`")),
        }
    }
}

fn parse_priority_flag(
    args: &[String],
    index: usize,
    flag: &'static str,
) -> Result<PriorityValue, String> {
    let Some(value) = args.get(index) else {
        return Err(format!("lint {flag} requires a priority value"));
    };
    PriorityValue::parse(value).ok_or_else(|| format!("unsupported priority value `{value}`"))
}

fn priority_profile_from_flags(
    highest: Option<PriorityValue>,
    lowest: Option<PriorityValue>,
    default: Option<PriorityValue>,
) -> Result<PriorityProfile, String> {
    if highest.is_none() && lowest.is_none() && default.is_none() {
        return Ok(PriorityProfile::org_default());
    }
    let profile = PriorityProfile::org_default();
    let highest = highest.unwrap_or_else(|| profile.highest().clone());
    let lowest = lowest.unwrap_or_else(|| profile.lowest().clone());
    let default = default.unwrap_or_else(|| profile.default_priority().clone());
    PriorityProfile::new(highest, lowest, default).ok_or_else(|| {
        "priority profile must use one priority family and satisfy highest <= default <= lowest"
            .to_string()
    })
}

fn load_property_schema_registries(paths: &[PathBuf]) -> Result<PropertySchemaRegistry, String> {
    let mut registry = PropertySchemaRegistry::default();
    for path in paths {
        let loaded = load_property_schema_registry(path)?;
        registry.contracts.extend(loaded.contracts);
    }
    Ok(registry)
}

fn load_property_schema_registry(path: &Path) -> Result<PropertySchemaRegistry, String> {
    let source = fs::read_to_string(path).map_err(|error| format_path_error(path, error))?;
    let value = serde_json::from_str::<serde_json::Value>(&source)
        .map_err(|error| format!("{}: invalid JSON: {error}", display_path(path)))?;
    let mut registry = property_schema_registry_from_json(&value)
        .map_err(|error| format!("{}: {error}", display_path(path)))?;
    add_property_schema_file_aliases(&mut registry, path);
    Ok(registry)
}

fn property_schema_registry_from_json(
    value: &serde_json::Value,
) -> Result<PropertySchemaRegistry, String> {
    if let Some(contracts) = value.get("contracts") {
        return Ok(PropertySchemaRegistry::new(parse_contracts(contracts)?));
    }
    if value.get("id").is_some() {
        return Ok(PropertySchemaRegistry::new([parse_schema_contract(value)?]));
    }
    if value.is_array() {
        return Ok(PropertySchemaRegistry::new(parse_contracts(value)?));
    }
    Err(
        "expected a registry object with `contracts`, a contract object, or a contract array"
            .to_string(),
    )
}

fn parse_contracts(value: &serde_json::Value) -> Result<Vec<PropertySchemaContract>, String> {
    value
        .as_array()
        .ok_or_else(|| "`contracts` must be an array".to_string())?
        .iter()
        .enumerate()
        .map(|(index, contract)| {
            parse_schema_contract(contract).map_err(|error| format!("contracts[{index}]: {error}"))
        })
        .collect()
}

fn parse_schema_contract(value: &serde_json::Value) -> Result<PropertySchemaContract, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "contract must be an object".to_string())?;
    let id = json_string(object, "id")
        .ok_or_else(|| "contract requires string `id`".to_string())?
        .to_string();
    let allow_unknown_properties =
        json_optional_bool(object, "allowUnknownProperties", "allow_unknown_properties")?
            .unwrap_or(true);
    let mut contract =
        PropertySchemaContract::new(id).allow_unknown_properties(allow_unknown_properties);

    if let Some(aliases) = object.get("aliases") {
        for alias in json_string_array(aliases, "aliases")? {
            contract = contract.alias(alias);
        }
    }

    if let Some(fields) = object.get("fields") {
        for (index, field) in fields
            .as_array()
            .ok_or_else(|| "`fields` must be an array".to_string())?
            .iter()
            .enumerate()
        {
            contract = contract.field(
                parse_schema_field(field).map_err(|error| format!("fields[{index}]: {error}"))?,
            );
        }
    }

    Ok(contract)
}

fn parse_schema_field(value: &serde_json::Value) -> Result<PropertySchemaField, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "field must be an object".to_string())?;
    let key = json_string(object, "key")
        .ok_or_else(|| "field requires string `key`".to_string())?
        .to_string();
    let required = json_optional_bool(object, "required", "required")?.unwrap_or(false);
    let value_rule = json_field(object, "valueRule", "value_rule")
        .map(parse_schema_value_rule)
        .transpose()?
        .unwrap_or(PropertySchemaValueRule::Any);

    Ok(if required {
        PropertySchemaField::required(key, value_rule)
    } else {
        PropertySchemaField::optional(key, value_rule)
    })
}

fn parse_schema_value_rule(value: &serde_json::Value) -> Result<PropertySchemaValueRule, String> {
    if let Some(kind) = value.as_str() {
        return match kind {
            "any" => Ok(PropertySchemaValueRule::Any),
            "nonEmpty" => Ok(PropertySchemaValueRule::NonEmpty),
            "oneOf" => Err("valueRule `oneOf` requires an object with `values`".to_string()),
            _ => Err(format!("unsupported valueRule kind `{kind}`")),
        };
    }
    let object = value
        .as_object()
        .ok_or_else(|| "valueRule must be an object or string".to_string())?;
    let kind = json_string(object, "kind")
        .ok_or_else(|| "valueRule requires string `kind`".to_string())?;
    match kind {
        "any" => Ok(PropertySchemaValueRule::Any),
        "nonEmpty" => Ok(PropertySchemaValueRule::NonEmpty),
        "oneOf" => {
            let values = object
                .get("values")
                .ok_or_else(|| "valueRule `oneOf` requires `values`".to_string())?;
            Ok(PropertySchemaValueRule::OneOf(json_string_array(
                values, "values",
            )?))
        }
        _ => Err(format!("unsupported valueRule kind `{kind}`")),
    }
}

fn json_field<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    camel: &str,
    snake: &str,
) -> Option<&'a serde_json::Value> {
    object.get(camel).or_else(|| object.get(snake))
}

fn json_string<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<&'a str> {
    object.get(key).and_then(serde_json::Value::as_str)
}

fn json_optional_bool(
    object: &serde_json::Map<String, serde_json::Value>,
    camel: &str,
    snake: &str,
) -> Result<Option<bool>, String> {
    json_field(object, camel, snake)
        .map(|value| {
            value
                .as_bool()
                .ok_or_else(|| format!("`{camel}` must be a boolean"))
        })
        .transpose()
}

fn json_string_array(value: &serde_json::Value, label: &str) -> Result<Vec<String>, String> {
    value
        .as_array()
        .ok_or_else(|| format!("`{label}` must be an array"))?
        .iter()
        .enumerate()
        .map(|(index, item)| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("`{label}` item {index} must be a string"))
        })
        .collect()
}

fn add_property_schema_file_aliases(registry: &mut PropertySchemaRegistry, path: &Path) {
    let bases = property_schema_file_alias_bases(path);
    let single_contract_id =
        (registry.contracts.len() == 1).then(|| registry.contracts[0].id.clone());
    for contract in &mut registry.contracts {
        for base in &bases {
            push_property_schema_alias(contract, format!("{base}#{}", contract.id));
            push_property_schema_alias(contract, format!("file:{base}#{}", contract.id));
        }
        if single_contract_id.as_deref() == Some(contract.id.as_str()) {
            for base in &bases {
                push_property_schema_alias(contract, base.clone());
                push_property_schema_alias(contract, format!("file:{base}"));
            }
        }
    }
}

fn property_schema_file_alias_bases(path: &Path) -> Vec<String> {
    let mut bases = Vec::new();
    push_property_schema_alias_base(&mut bases, path);
    if let Ok(canonical) = path.canonicalize() {
        push_property_schema_alias_base(&mut bases, canonical.as_path());
    }
    bases
}

fn push_property_schema_alias_base(bases: &mut Vec<String>, path: &Path) {
    let value = normalize_property_schema_path(path);
    if !value.is_empty() && !bases.iter().any(|base| base == &value) {
        bases.push(value.clone());
    }
    match value.strip_prefix("./") {
        Some(stripped) if !stripped.is_empty() && !bases.iter().any(|base| base == stripped) => {
            bases.push(stripped.to_string());
        }
        _ => {}
    }
}

fn normalize_property_schema_path(path: &Path) -> String {
    display_path(path)
}

fn push_property_schema_alias(contract: &mut PropertySchemaContract, alias: String) {
    if alias.trim().is_empty()
        || alias == contract.id
        || contract.aliases.iter().any(|existing| existing == &alias)
    {
        return;
    }
    contract.aliases.push(alias);
}
