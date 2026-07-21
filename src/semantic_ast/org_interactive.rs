//! Org-owned interactive choice projection.

use std::collections::HashSet;

use super::{
    Document, OrgInteractiveCategory, OrgInteractiveChoice, OrgInteractiveChoiceEntry,
    OrgInteractiveParseError, ParsedAnnotation, SourceBlockRecord,
};

impl Document<ParsedAnnotation> {
    /// Projects validated interactive choice windows from Org source blocks.
    ///
    /// The formal surface is `org-contract :type agent-interactive`; consumers
    /// share one parser and one DTO shape instead of recognizing aliases.
    pub fn org_interactive_choices(
        &self,
    ) -> Result<Vec<OrgInteractiveChoice>, OrgInteractiveParseError> {
        self.source_block_records()
            .iter()
            .filter(|record| {
                record.language.as_deref() == Some("org-contract")
                    && record.header_args.iter().any(|arg| {
                        arg.key == "type" && arg.value.as_deref() == Some("agent-interactive")
                    })
            })
            .map(parse_choice)
            .collect()
    }
}

fn parse_choice(
    record: &SourceBlockRecord,
) -> Result<OrgInteractiveChoice, OrgInteractiveParseError> {
    let mut id = None;
    let mut method = None;
    let mut stage = None;
    let mut group = None;
    let mut target = None;
    let mut create = None;
    let mut info = None;
    let mut categories = None;
    let mut in_details = false;
    let mut entries = Vec::new();

    for line in record.value.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if in_details && line.starts_with('|') {
            if let Some(entry) = parse_table_row(line)? {
                entries.push(entry);
            }
            continue;
        }
        if line == "details:" {
            in_details = true;
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let value = value.trim().to_string();
            match key.trim() {
                "id" => id = Some(value),
                "method" => method = Some(value),
                "stage" => stage = Some(value),
                "group" => group = optional(value),
                "target" => target = optional(value),
                "create" => create = optional(value),
                "info" => info = Some(value),
                "categories" => categories = Some(value),
                _ => {}
            }
        }
    }

    let id = required(id, "id")?;
    let method = required(method, "method")?;
    if method != "choice" {
        return Err(error(format!(
            "agent-interactive `{id}` must use `method: choice`, got `{method}`"
        )));
    }
    let stage = required(stage, "stage")?;
    let info = required(info, "info")?;
    if entries.is_empty() {
        return Err(error(format!(
            "agent-interactive `{id}` details table must contain at least one row"
        )));
    }
    let categories = parse_categories(&id, &required(categories, "categories")?, &entries)?;

    Ok(OrgInteractiveChoice {
        source: record.source.clone(),
        id,
        method,
        stage,
        group,
        target,
        create,
        info,
        categories,
        entries,
    })
}

fn parse_categories(
    id: &str,
    value: &str,
    entries: &[OrgInteractiveChoiceEntry],
) -> Result<Vec<OrgInteractiveCategory>, OrgInteractiveParseError> {
    let mut categories = Vec::new();
    let mut keys = HashSet::new();
    let mut has_detail = false;
    for part in value.split(',') {
        let (key, value) = part.split_once('=').ok_or_else(|| {
            error(format!(
                "agent-interactive `{id}` category `{part}` must use key=value"
            ))
        })?;
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() || !keys.insert(key.to_string()) {
            return Err(error(format!(
                "agent-interactive `{id}` category keys and values must be non-empty and unique"
            )));
        }
        let detail = key == "?" && value == "detail";
        has_detail |= detail;
        if !detail
            && !entries
                .iter()
                .any(|entry| entry.number == key && entry.id == value)
        {
            return Err(error(format!(
                "agent-interactive `{id}` category `{key}={value}` must match a detail row"
            )));
        }
        categories.push(OrgInteractiveCategory {
            key: key.to_string(),
            value: value.to_string(),
            detail,
        });
    }
    if !has_detail {
        return Err(error(format!(
            "agent-interactive `{id}` categories must include `?=detail`"
        )));
    }
    Ok(categories)
}

fn parse_table_row(
    line: &str,
) -> Result<Option<OrgInteractiveChoiceEntry>, OrgInteractiveParseError> {
    let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
    if cells == ["n", "id", "contract", "full", "use-if"] {
        return Ok(None);
    }
    if cells.len() != 5 {
        return Err(error(format!(
            "agent-interactive detail row must use `n|id|contract|full|use-if`: {line}"
        )));
    }
    Ok(Some(OrgInteractiveChoiceEntry {
        number: required_cell(cells[0], "n")?,
        id: required_cell(cells[1], "id")?,
        contract: optional(cells[2].to_string()),
        full: required_cell(cells[3], "full")?,
        use_if: required_cell(cells[4], "use-if")?,
    }))
}

fn required(value: Option<String>, field: &str) -> Result<String, OrgInteractiveParseError> {
    value
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| error(format!("agent-interactive choice requires `{field}`")))
}

fn required_cell(value: &str, field: &str) -> Result<String, OrgInteractiveParseError> {
    (!value.is_empty())
        .then(|| value.to_string())
        .ok_or_else(|| error(format!("agent-interactive detail row requires `{field}`")))
}

fn optional(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty() && value != "-").then(|| value.to_string())
}

fn error(message: impl Into<String>) -> OrgInteractiveParseError {
    OrgInteractiveParseError::new(message)
}
