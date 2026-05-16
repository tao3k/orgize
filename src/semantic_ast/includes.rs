//! Safe include expansion planning.

use super::{
    Document, IncludeDirective, IncludeExpansionEntry, IncludeExpansionMode,
    IncludeExpansionOptions, IncludeExpansionPlan, IncludeLineSelection, IncludeOption,
};

impl<A: Clone> Document<A> {
    /// Builds a non-executing plan for explicit include expansion.
    ///
    /// The parser records source intent only. Callers decide whether and how to
    /// read files, enforce roots, or rewrite the document.
    pub fn include_expansion_plan(
        &self,
        options: &IncludeExpansionOptions,
    ) -> IncludeExpansionPlan<A> {
        IncludeExpansionPlan {
            entries: self
                .includes
                .iter()
                .map(|directive| include_expansion_entry(directive, options))
                .collect(),
        }
    }
}

fn include_expansion_entry<A: Clone>(
    directive: &IncludeDirective<A>,
    options: &IncludeExpansionOptions,
) -> IncludeExpansionEntry<A> {
    IncludeExpansionEntry {
        directive: directive.clone(),
        resolved_path: resolved_include_path(directive.path.as_str(), options),
        line_selection: include_line_selection(&directive.options),
        min_level: include_min_level(&directive.options),
        mode: include_mode(&directive.arguments),
        options: directive.options.clone(),
    }
}

fn resolved_include_path(path: &str, options: &IncludeExpansionOptions) -> Option<String> {
    if is_absolute_or_special_path(path) {
        return Some(path.to_string());
    }
    let base = options.base_dir.as_deref()?;
    Some(format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches("./")
    ))
}

fn is_absolute_or_special_path(path: &str) -> bool {
    path.starts_with('/') || path.starts_with("~/") || path.contains("://")
}

fn include_line_selection(options: &[IncludeOption]) -> IncludeLineSelection {
    options
        .iter()
        .find(|option| option.key.eq_ignore_ascii_case("lines"))
        .and_then(|option| option.value.as_deref())
        .map(parse_line_selection)
        .unwrap_or(IncludeLineSelection::All)
}

fn parse_line_selection(raw: &str) -> IncludeLineSelection {
    let value = raw.trim();
    let Some((start, end)) = value.split_once('-') else {
        return IncludeLineSelection::Invalid {
            raw: raw.to_string(),
        };
    };
    let start = parse_optional_line_bound(start);
    let end = parse_optional_line_bound(end);
    if start.is_none() && end.is_none() {
        IncludeLineSelection::Invalid {
            raw: raw.to_string(),
        }
    } else {
        IncludeLineSelection::Range {
            start,
            end,
            raw: raw.to_string(),
        }
    }
}

fn parse_optional_line_bound(value: &str) -> Option<usize> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        value.parse::<usize>().ok().filter(|line| *line > 0)
    }
}

fn include_min_level(options: &[IncludeOption]) -> Option<usize> {
    options
        .iter()
        .find(|option| option.key.eq_ignore_ascii_case("minlevel"))
        .and_then(|option| option.value.as_deref())
        .and_then(|value| value.trim().parse::<usize>().ok())
}

fn include_mode(arguments: &[String]) -> IncludeExpansionMode {
    let Some(first) = arguments.first() else {
        return IncludeExpansionMode::Org;
    };
    match first.to_ascii_lowercase().as_str() {
        "example" => IncludeExpansionMode::Example,
        "src" => IncludeExpansionMode::Source {
            language: arguments.get(1).cloned(),
        },
        "export" => IncludeExpansionMode::Export {
            backend: arguments.get(1).cloned(),
        },
        _ => IncludeExpansionMode::Other {
            arguments: arguments.to_vec(),
        },
    }
}
