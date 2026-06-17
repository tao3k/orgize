//! Ordinary file-link lint checks.

use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use crate::ast::{AstRef, FileLinkPathKind, ObjectData, ParsedAst};

use super::lint_model::{
    LintFinding, LintOptions, LintSeverity, location_for_range, location_for_range_bounds,
};

pub(crate) fn file_link_findings(
    document: &ParsedAst,
    source: &str,
    options: &LintOptions,
) -> Vec<LintFinding> {
    let Some(base_dir) = &options.file_base_dir else {
        return Vec::new();
    };

    let mut findings = Vec::new();
    document.visit(|node| {
        let AstRef::Object(object) = node else {
            return;
        };
        let ObjectData::Link(link) = &object.data else {
            return;
        };
        let Some(file) = link.file.as_ref() else {
            return;
        };
        if file.path_kind == FileLinkPathKind::Remote {
            return;
        }
        if file.path.trim().is_empty() {
            findings.push(LintFinding {
                code: "ORG017",
                severity: LintSeverity::Warning,
                message: "file link has no file path".to_string(),
                location: location_for_range(source, object.ann.range),
            });
            return;
        }

        let Some(path) = resolve_file_link_path(base_dir, file.path.as_str(), file.path_kind)
        else {
            return;
        };
        let message = match fs::metadata(&path) {
            Ok(metadata) if metadata.is_file() => None,
            Ok(_) => Some(format!(
                "file target `{}` points at a directory",
                link.path()
            )),
            Err(error) if error.kind() == ErrorKind::NotFound => {
                Some(format!("file target `{}` was not found", link.path()))
            }
            Err(error) => Some(format!(
                "file target `{}` could not be read: {error}",
                link.path()
            )),
        };
        if let Some(message) = message {
            findings.push(LintFinding {
                code: "ORG017",
                severity: LintSeverity::Warning,
                message,
                location: location_for_range(source, object.ann.range),
            });
        }
    });
    findings.extend(org_package_relative_path_findings(source, options));
    findings
}

fn org_package_relative_path_findings(source: &str, options: &LintOptions) -> Vec<LintFinding> {
    let Some(base_dir) = &options.file_base_dir else {
        return Vec::new();
    };
    let Some(package_dir) = base_dir.file_name().and_then(|name| name.to_str()) else {
        return Vec::new();
    };

    org_package_path_rules(package_dir)
        .flat_map(|directory| {
            source
                .match_indices(directory.path_segment)
                .filter_map(move |(start, _)| org_package_path_finding(source, start, directory))
        })
        .collect()
}

#[derive(Clone, Copy)]
struct OrgPackagePathRule {
    path_segment: &'static str,
    recommendation: &'static str,
}

fn org_package_path_rules(package_dir: &str) -> impl Iterator<Item = OrgPackagePathRule> {
    let rules: &[OrgPackagePathRule] = match package_dir {
        "skills" => &[
            OrgPackagePathRule {
                path_segment: "contracts/",
                recommendation: "../contracts/...",
            },
            OrgPackagePathRule {
                path_segment: "templates/",
                recommendation: "../templates/...",
            },
        ],
        "templates" => &[
            OrgPackagePathRule {
                path_segment: "contracts/",
                recommendation: "../contracts/...",
            },
            OrgPackagePathRule {
                path_segment: "templates/",
                recommendation: "same-directory template links such as agent.execplan.v1.org",
            },
        ],
        _ => &[],
    };
    rules.iter().copied()
}

fn org_package_path_finding(
    source: &str,
    directory_start: usize,
    rule: OrgPackagePathRule,
) -> Option<LintFinding> {
    let path_start = org_package_path_start(source, directory_start);
    let path = org_package_path_token(source, path_start);
    if !path.contains(".org") {
        return None;
    }
    if path.starts_with("../") {
        return None;
    }
    if !(path.starts_with(rule.path_segment)
        || path.starts_with("<ASP_ORG_ROOT>/")
        || path.starts_with("languages/org/"))
    {
        return None;
    }

    Some(LintFinding {
        code: "ORG018",
        severity: LintSeverity::Warning,
        message: format!(
            "Org package path `{path}` should use sibling-relative style `{}`",
            rule.recommendation
        ),
        location: location_for_range_bounds(source, path_start, path_start + path.len()),
    })
}

fn org_package_path_start(source: &str, directory_start: usize) -> usize {
    source[..directory_start]
        .rfind(|ch: char| ch.is_whitespace() || matches!(ch, '[' | '(' | '"' | '\'' | '='))
        .map_or(0, |index| index + 1)
}

fn org_package_path_token(source: &str, start: usize) -> &str {
    let relative_end = source[start..]
        .find(|ch: char| ch.is_whitespace() || matches!(ch, ']' | ')' | '"' | '\'' | '='))
        .unwrap_or_else(|| source.len() - start);
    &source[start..start + relative_end]
}

fn resolve_file_link_path(base_dir: &Path, path: &str, kind: FileLinkPathKind) -> Option<PathBuf> {
    match kind {
        FileLinkPathKind::Empty | FileLinkPathKind::Remote => None,
        FileLinkPathKind::Absolute => Some(PathBuf::from(path)),
        FileLinkPathKind::HomeRelative => {
            let home = std::env::var_os("HOME")?;
            Some(PathBuf::from(home).join(path.trim_start_matches("~/")))
        }
        FileLinkPathKind::Relative => Some(base_dir.join(path)),
    }
}
