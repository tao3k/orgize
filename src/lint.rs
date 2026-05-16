//! Org document linting built on the semantic parser projection.

#[path = "lint_attachments.rs"]
mod lint_attachments;
#[path = "lint_babel.rs"]
mod lint_babel;
#[path = "lint_file_links.rs"]
mod lint_file_links;
#[path = "lint_lifecycle.rs"]
mod lint_lifecycle;
#[path = "lint_model.rs"]
mod lint_model;
#[path = "lint_priority.rs"]
mod lint_priority;
#[path = "lint_progress.rs"]
mod lint_progress;
#[path = "lint_properties.rs"]
mod lint_properties;
#[path = "lint_render.rs"]
mod lint_render;
#[path = "lint_table_formulas.rs"]
mod lint_table_formulas;
#[path = "lint_task_blockers.rs"]
mod lint_task_blockers;

use std::{collections::BTreeMap, fs, io::ErrorKind, path::Path};

use self::{
    lint_attachments::attachment_findings,
    lint_babel::babel_findings,
    lint_file_links::file_link_findings,
    lint_lifecycle::lifecycle_findings,
    lint_model::{location_for_offsets, location_for_range},
    lint_priority::priority_cookie_findings,
    lint_progress::progress_findings,
    lint_properties::property_drawer_findings,
    lint_table_formulas::table_formula_findings,
    lint_task_blockers::task_blocker_findings,
};

use crate::{
    ast::{
        Diagnostic, IncludeDirective, Keyword, MacroDefinition, MacroExpansionStatus,
        ParsedAnnotation, ParsedAst, TargetDefinition, TargetKind,
    },
    Org,
};

pub use self::lint_model::{LintFinding, LintLocation, LintOptions, LintReport, LintSeverity};

/// Lints Org source with the default parser configuration.
pub fn lint_org(source: &str) -> LintReport {
    lint_org_with_options(source, &LintOptions::default())
}

/// Lints Org source with explicit lint options.
pub fn lint_org_with_options(source: &str, options: &LintOptions) -> LintReport {
    let org = Org::parse(source);
    lint_document_with_options(&org.document(), source, options)
}

/// Lints an already projected semantic document.
pub fn lint_document(document: &ParsedAst, source: &str) -> LintReport {
    lint_document_with_options(document, source, &LintOptions::default())
}

/// Lints an already projected semantic document with explicit lint options.
pub fn lint_document_with_options(
    document: &ParsedAst,
    source: &str,
    options: &LintOptions,
) -> LintReport {
    let mut findings = collect_lint_findings(document, source, options);
    sort_lint_findings(&mut findings);
    LintReport { findings }
}

fn collect_lint_findings(
    document: &ParsedAst,
    source: &str,
    options: &LintOptions,
) -> Vec<LintFinding> {
    let mut findings = Vec::new();

    findings.extend(
        document
            .diagnostics
            .iter()
            .map(|diagnostic| finding_from_diagnostic(diagnostic, source)),
    );
    findings.extend(duplicate_target_findings(&document.targets, source));
    findings.extend(include_path_findings(&document.includes, source, options));
    findings.extend(duplicate_macro_definition_findings(
        &document.macro_definitions,
        source,
    ));
    findings.extend(missing_macro_findings(document, source));
    findings.extend(link_abbreviation_definition_findings(
        &document.metadata,
        source,
    ));
    findings.extend(options_keyword_findings(&document.metadata, source));
    findings.extend(priority_cookie_findings(source, &options.priority_profile));
    findings.extend(property_drawer_findings(document, source));
    findings.extend(progress_findings(document, source));
    findings.extend(attachment_findings(document, source, options));
    findings.extend(babel_findings(document, source));
    findings.extend(file_link_findings(document, source, options));
    findings.extend(lifecycle_findings(document, source, options));
    findings.extend(table_formula_findings(document, source));
    findings.extend(task_blocker_findings(document, source));
    findings.extend(todo_declaration_findings(source));

    findings
}

fn sort_lint_findings(findings: &mut [LintFinding]) {
    findings.sort_by(|left, right| {
        left.location
            .range_start
            .cmp(&right.location.range_start)
            .then_with(|| left.code.cmp(right.code))
            .then_with(|| left.message.cmp(&right.message))
    });
}

fn finding_from_diagnostic(diagnostic: &Diagnostic, source: &str) -> LintFinding {
    LintFinding {
        code: "ORG001",
        severity: LintSeverity::Error,
        message: diagnostic.message.clone(),
        location: location_for_range(source, diagnostic.range),
    }
}

fn duplicate_target_findings(
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

fn missing_macro_findings(document: &ParsedAst, source: &str) -> Vec<LintFinding> {
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

fn duplicate_macro_definition_findings(
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

fn link_abbreviation_definition_findings(
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

fn options_keyword_findings(
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

fn todo_declaration_findings(source: &str) -> Vec<LintFinding> {
    duplicate_todo_declaration_findings(source, &todo_declaration_lines(source))
}

fn duplicate_todo_declaration_findings(
    source: &str,
    lines: &[TodoDeclarationLine<'_>],
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
    line: &TodoDeclarationLine<'_>,
    seen: &mut BTreeMap<String, SeenTodoDeclaration>,
    findings: &mut Vec<LintFinding>,
) {
    for declaration in todo_declarations(line.value) {
        if let Some(finding) = todo_declaration_duplicate_finding(source, line, declaration, seen) {
            findings.push(finding);
        }
    }
}

fn todo_declaration_duplicate_finding(
    source: &str,
    line: &TodoDeclarationLine<'_>,
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
        location: location_for_offsets(source, line.range_start, line.range_end),
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

fn todo_declaration_lines(source: &str) -> Vec<TodoDeclarationLine<'_>> {
    let mut lines = Vec::new();
    let mut in_block = false;
    let mut offset = 0;

    for segment in source.split_inclusive('\n') {
        let line = segment.trim_end_matches('\n').trim_end_matches('\r');
        let trimmed = line.trim_start_matches([' ', '\t']);
        if in_block {
            if is_lint_keyword_line_with_prefix(trimmed, "end_") {
                in_block = false;
            }
            offset += segment.len();
            continue;
        }

        if is_lint_keyword_line_with_prefix(trimmed, "begin_") {
            in_block = true;
            offset += segment.len();
            continue;
        }

        let Some(value) = todo_declaration_line_value(trimmed) else {
            offset += segment.len();
            continue;
        };

        lines.push(TodoDeclarationLine {
            value,
            range_start: offset + line.len() - trimmed.len(),
            range_end: offset + line.len(),
        });

        offset += segment.len();
    }

    lines
}

fn todo_declaration_line_value(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("#+")?;
    let (key, value) = rest.split_once(':')?;
    is_todo_declaration_key(key).then_some(value)
}

fn is_todo_declaration_key(key: &str) -> bool {
    matches!(
        key.to_ascii_uppercase().as_str(),
        "TODO" | "SEQ_TODO" | "TYP_TODO"
    )
}

fn is_lint_keyword_line_with_prefix(line: &str, prefix: &str) -> bool {
    let Some(rest) = line.strip_prefix("#+") else {
        return false;
    };
    rest.get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
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
struct TodoDeclarationLine<'a> {
    value: &'a str,
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

fn include_path_findings(
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
