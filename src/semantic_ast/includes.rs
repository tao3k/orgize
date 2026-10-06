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
    let row = super::org_native_values::rows("include-lines", &[raw])
        .pop()
        .expect("native include selection");
    let [kind, start, end]: [String; 3] = row.try_into().expect("native include arity");
    match kind.as_str() {
        "invalid" => IncludeLineSelection::Invalid {
            raw: raw.to_owned(),
        },
        "range" => IncludeLineSelection::Range {
            start: (!start.is_empty()).then(|| start.parse().expect("native line start")),
            end: (!end.is_empty()).then(|| end.parse().expect("native line end")),
            raw: raw.to_owned(),
        },
        _ => panic!("native include kind"),
    }
}

fn include_min_level(options: &[IncludeOption]) -> Option<usize> {
    options
        .iter()
        .find(|option| option.key.eq_ignore_ascii_case("minlevel"))
        .and_then(|option| option.value.as_deref())
        .and_then(|value| super::org_native_values::optional("unsigned", value))
        .map(|row| row[0].parse().expect("native include minlevel"))
}

fn include_mode(arguments: &[String]) -> IncludeExpansionMode {
    let Some(first) = arguments.first() else {
        return IncludeExpansionMode::Org;
    };
    let fields: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let _ = first;
    match super::org_native_values::scalar("include-mode", &fields).as_str() {
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
