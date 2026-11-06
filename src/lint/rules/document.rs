//! Document-level metadata, include, target and declaration rules.

use std::{collections::BTreeMap, fs, io::ErrorKind, path::Path};

use super::model::location_for_range_bounds;
use super::{LintFinding, LintOptions, LintSeverity, location_for_range};
use crate::ast::{
    Diagnostic, IncludeDirective, Keyword, MacroDefinition, MacroExpansionStatus, ParsedAnnotation,
    ParsedAst, TargetDefinition, TargetKind,
};

pub(super) fn finding_from_diagnostic(diagnostic: &Diagnostic, source: &str) -> LintFinding {
    LintFinding {
        code: "ORG001",
        severity: LintSeverity::Error,
        message: diagnostic.message.clone(),
        location: location_for_range(source, diagnostic.range),
    }
}

pub(super) fn duplicate_target_findings(
    targets: &[TargetDefinition<crate::ast::ParsedAnnotation>],
    source: &str,
) -> Vec<LintFinding> {
    let mut by_key = BTreeMap::<&str, Vec<&TargetDefinition<_>>>::new();
    for target in targets {
        by_key.entry(&target.key).or_default().push(target);
    }

    let mut findings = Vec::new();
    for (key, definitions) in by_key {
        if definitions.len() < 2 {
            continue;
        }
        let first = definitions[0];
        findings.push(LintFinding {
            code: "ORG002",
            severity: duplicate_target_severity(key, &definitions),
            message: format!("target `{key}` is defined {} times", definitions.len()),
            location: location_for_range(source, first.ann.range),
        });
    }
    findings
}

fn duplicate_target_severity(
    key: &str,
    definitions: &[&TargetDefinition<crate::ast::ParsedAnnotation>],
) -> LintSeverity {
    if key.starts_with("id:")
        || key.starts_with('#')
        || definitions
            .iter()
            .any(|target| matches!(target.kind, TargetKind::Id | TargetKind::CustomId))
    {
        LintSeverity::Error
    } else {
        LintSeverity::Warning
    }
}

pub(super) fn missing_macro_findings(document: &ParsedAst, source: &str) -> Vec<LintFinding> {
    document
        .macro_expansions()
        .into_iter()
        .filter(|expansion| expansion.status == MacroExpansionStatus::MissingDefinition)
        .map(|expansion| LintFinding {
            code: "ORG004",
            severity: LintSeverity::Warning,
            message: format!("macro `{}` has no local definition", expansion.name),
            location: location_for_range(source, expansion.ann.range),
        })
        .collect()
}

pub(super) fn duplicate_macro_definition_findings(
    definitions: &[MacroDefinition<ParsedAnnotation>],
    source: &str,
) -> Vec<LintFinding> {
    let mut by_name = BTreeMap::<&str, Vec<&MacroDefinition<ParsedAnnotation>>>::new();
    for definition in definitions {
        by_name
            .entry(&definition.name)
            .or_default()
            .push(definition);
    }

    let mut findings = Vec::new();
    for (name, definitions) in by_name {
        if definitions.len() < 2 {
            continue;
        }
        let duplicate = definitions[1];
        findings.push(LintFinding {
            code: "ORG008",
            severity: LintSeverity::Warning,
            message: format!("macro `{name}` is defined {} times", definitions.len()),
            location: location_for_range(source, duplicate.ann.range),
        });
    }
    findings
}

pub(super) fn link_abbreviation_definition_findings(
    metadata: &[Keyword<ParsedAnnotation>],
    source: &str,
) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    let mut by_name = BTreeMap::<String, Vec<&Keyword<ParsedAnnotation>>>::new();

    for keyword in metadata {
        if !keyword.key.eq_ignore_ascii_case("LINK") {
            continue;
        }

        let value = keyword.value.trim();
        let Some((name, replacement)) = value.split_once(char::is_whitespace) else {
            findings.push(malformed_link_abbreviation_finding(keyword, source));
            continue;
        };
        let name = name.trim();
        if name.is_empty() || replacement.trim().is_empty() {
            findings.push(malformed_link_abbreviation_finding(keyword, source));
            continue;
        }

        by_name
            .entry(name.to_ascii_lowercase())
            .or_default()
            .push(keyword);
    }

    for (name, definitions) in by_name {
        if definitions.len() < 2 {
            continue;
        }
        let duplicate = definitions[1];
        findings.push(LintFinding {
            code: "ORG006",
            severity: LintSeverity::Warning,
            message: format!(
                "link abbreviation `{name}` is defined {} times",
                definitions.len()
            ),
            location: location_for_range(source, duplicate.ann.range),
        });
    }

    findings
}

fn malformed_link_abbreviation_finding(
    keyword: &Keyword<ParsedAnnotation>,
    source: &str,
) -> LintFinding {
    LintFinding {
        code: "ORG005",
        severity: LintSeverity::Warning,
        message: "LINK keyword is missing an abbreviation name or replacement".into(),
        location: location_for_range(source, keyword.ann.range),
    }
}

pub(super) fn options_keyword_findings(
    metadata: &[Keyword<ParsedAnnotation>],
    source: &str,
) -> Vec<LintFinding> {
    let mut findings = Vec::new();

    for keyword in metadata {
        if !keyword.key.eq_ignore_ascii_case("OPTIONS") {
            continue;
        }

        for token in keyword.value.split_whitespace() {
            let Some((key, value)) = token.split_once(':') else {
                continue;
            };
            let message = match key {
                "H" if value.parse::<usize>().is_err() => Some(format!(
                    "OPTIONS `H` expects a non-negative integer, got `{value}`"
                )),
                "-" | "e" if !is_bool_option(value) => Some(format!(
                    "OPTIONS `{key}` expects t/nil or true/false, got `{value}`"
                )),
                _ => None,
            };

            if let Some(message) = message {
                findings.push(LintFinding {
                    code: "ORG007",
                    severity: LintSeverity::Warning,
                    message,
                    location: location_for_range(source, keyword.ann.range),
                });
            }
        }
    }

    findings
}

fn is_bool_option(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "t" | "true" | "yes" | "nil" | "false" | "no"
    )
}

pub(super) fn todo_declaration_findings(document: &ParsedAst, source: &str) -> Vec<LintFinding> {
    duplicate_todo_declaration_findings(source, &todo_declaration_lines(document, source))
}

fn duplicate_todo_declaration_findings(
    source: &str,
    lines: &[TodoDeclarationLine],
) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    let mut seen = BTreeMap::<String, SeenTodoDeclaration>::new();
    for line in lines {
        push_todo_declaration_line_findings(source, line, &mut seen, &mut findings);
    }
    findings
}

fn push_todo_declaration_line_findings(
    source: &str,
    line: &TodoDeclarationLine,
    seen: &mut BTreeMap<String, SeenTodoDeclaration>,
    findings: &mut Vec<LintFinding>,
) {
    for declaration in todo_declarations(&line.value) {
        if let Some(finding) = todo_declaration_duplicate_finding(source, line, declaration, seen) {
            findings.push(finding);
        }
    }
}

fn todo_declaration_duplicate_finding(
    source: &str,
    line: &TodoDeclarationLine,
    declaration: TodoDeclaration,
    seen: &mut BTreeMap<String, SeenTodoDeclaration>,
) -> Option<LintFinding> {
    let Some(previous) = seen.get_mut(&declaration.name) else {
        seen.insert(
            declaration.name,
            SeenTodoDeclaration {
                state: declaration.state,
                count: 1,
            },
        );
        return None;
    };

    previous.count += 1;
    Some(LintFinding {
        code: "ORG009",
        severity: LintSeverity::Warning,
        message: todo_declaration_duplicate_message(&declaration, previous),
        location: location_for_range_bounds(source, line.range_start, line.range_end),
    })
}

fn todo_declaration_duplicate_message(
    declaration: &TodoDeclaration,
    previous: &SeenTodoDeclaration,
) -> String {
    if previous.state == declaration.state {
        format!(
            "TODO keyword `{}` is declared {} times as {}",
            declaration.name,
            previous.count,
            declaration.state.as_str()
        )
    } else {
        format!(
            "TODO keyword `{}` is declared as both {} and {}",
            declaration.name,
            previous.state.as_str(),
            declaration.state.as_str()
        )
    }
}

fn todo_declaration_lines(document: &ParsedAst, source: &str) -> Vec<TodoDeclarationLine> {
    let mut lines = Vec::new();
    document.visit(|node| {
        if let crate::ast::AstRef::Keyword(keyword) = node {
            if !is_todo_declaration_key(&keyword.key) {
                return;
            }
            let range_start = u32::from(keyword.ann.range.start()) as usize;
            let range_end = u32::from(keyword.ann.range.end()) as usize;
            let range_end = source
                .get(range_start..range_end)
                .map(|text| range_start + text.trim_end_matches(['\n', '\r']).len())
                .unwrap_or(range_end);
            lines.push(TodoDeclarationLine {
                value: keyword.value.clone(),
                range_start,
                range_end,
            });
        }
    });
    lines.sort_by_key(|line| line.range_start);
    lines.dedup_by_key(|line| line.range_start);
    lines
}

fn is_todo_declaration_key(key: &str) -> bool {
    matches!(
        key.to_ascii_uppercase().as_str(),
        "TODO" | "SEQ_TODO" | "TYP_TODO"
    )
}

fn todo_declarations(value: &str) -> Vec<TodoDeclaration> {
    let mut declarations = Vec::new();
    let mut state = TodoDeclarationState::Todo;

    for token in value.split_whitespace() {
        if token == "|" {
            state = TodoDeclarationState::Done;
            continue;
        }

        if let Some(name) = todo_declaration_name(token) {
            declarations.push(TodoDeclaration { name, state });
        }
    }

    declarations
}

fn todo_declaration_name(token: &str) -> Option<String> {
    let token = token.trim();
    if token.is_empty() || token.starts_with('(') {
        return None;
    }

    let name = token
        .split_once('(')
        .map(|(name, _)| name)
        .unwrap_or(token)
        .trim();

    (!name.is_empty() && name != "|").then(|| name.to_string())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SeenTodoDeclaration {
    state: TodoDeclarationState,
    count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TodoDeclaration {
    name: String,
    state: TodoDeclarationState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TodoDeclarationLine {
    value: String,
    range_start: usize,
    range_end: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TodoDeclarationState {
    Todo,
    Done,
}

impl TodoDeclarationState {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Todo => "TODO",
            Self::Done => "DONE",
        }
    }
}

pub(super) fn include_path_findings(
    includes: &[IncludeDirective<ParsedAnnotation>],
    source: &str,
    options: &LintOptions,
) -> Vec<LintFinding> {
    let Some(base_dir) = &options.include_base_dir else {
        return Vec::new();
    };

    includes
        .iter()
        .filter_map(|include| include_path_finding(include, source, base_dir))
        .collect()
}

fn include_path_finding(
    include: &IncludeDirective<ParsedAnnotation>,
    source: &str,
    base_dir: &Path,
) -> Option<LintFinding> {
    if include.path.contains("://") || include.path.starts_with('~') {
        return None;
    }

    let file_path = include_file_path(&include.path);
    let path = Path::new(file_path);
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    };

    let message = match fs::metadata(&resolved) {
        Ok(metadata) if metadata.is_file() => return None,
        Ok(_) => format!("include path `{}` is not a file", include.path),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            format!("include path `{}` was not found", include.path)
        }
        Err(error) => format!("include path `{}` could not be read: {error}", include.path),
    };

    Some(LintFinding {
        code: "ORG003",
        severity: LintSeverity::Error,
        message,
        location: location_for_range(source, include.ann.range),
    })
}

fn include_file_path(path: &str) -> &str {
    path.split_once("::")
        .map(|(file_path, _)| file_path)
        .unwrap_or(path)
}
