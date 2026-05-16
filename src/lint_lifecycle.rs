//! Lifecycle and archive lint checks.

use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use crate::{
    ast::{
        ArchiveLocation, AstRef, LifecycleRecordKind, ParsedAnnotation, ParsedAst, Property,
        Section,
    },
    Org,
};

use super::lint_model::{location_for_range, LintFinding, LintOptions, LintSeverity};

pub(crate) fn lifecycle_findings(
    document: &ParsedAst,
    source: &str,
    options: &LintOptions,
) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    push_malformed_logbook_findings(document, source, &mut findings);
    push_archive_location_findings(document, source, &mut findings);
    push_archive_destination_findings(document, source, options, &mut findings);
    push_refile_destination_findings(document, source, options, &mut findings);
    findings
}

fn push_malformed_logbook_findings(
    document: &ParsedAst,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    for record in document.lifecycle_records() {
        let LifecycleRecordKind::MalformedLogbook { reason } = record.kind else {
            continue;
        };
        findings.push(LintFinding {
            code: "ORG014",
            severity: LintSeverity::Warning,
            message: format!("malformed LOGBOOK lifecycle line: {reason}"),
            location: location_for_range(source, record.ann.range),
        });
    }
}

fn push_archive_destination_findings(
    document: &ParsedAst,
    source: &str,
    options: &LintOptions,
    findings: &mut Vec<LintFinding>,
) {
    let Some(base_dir) = &options.file_base_dir else {
        return;
    };

    for location in &document.archive_locations {
        push_archive_destination_finding(document, source, base_dir, location, findings);
    }
    document.visit(|node| match node {
        AstRef::Property(property) if property.key.eq_ignore_ascii_case("ARCHIVE") => {
            let location =
                ArchiveLocation::from_value(property.ann.clone(), property.value.clone());
            push_archive_destination_finding(document, source, base_dir, &location, findings);
        }
        _ => {}
    });
}

fn push_archive_destination_finding(
    document: &ParsedAst,
    source: &str,
    base_dir: &Path,
    location: &ArchiveLocation<ParsedAnnotation>,
    findings: &mut Vec<LintFinding>,
) {
    if location.is_empty() {
        return;
    }
    if let Some(file) = &location.file {
        if is_remote_file_path(file) {
            return;
        }
        let path = resolve_lifecycle_path(base_dir, file);
        if let Some(message) = destination_file_message(path.as_path(), "archive", file) {
            findings.push(LintFinding {
                code: "ORG018",
                severity: LintSeverity::Warning,
                message,
                location: location_for_range(source, location.ann.range),
            });
            return;
        }
        if let Some(heading) = &location.heading {
            if !heading_exists_in_file(path.as_path(), heading) {
                findings.push(LintFinding {
                    code: "ORG018",
                    severity: LintSeverity::Warning,
                    message: format!("archive destination heading `{heading}` was not found"),
                    location: location_for_range(source, location.ann.range),
                });
            }
        }
    } else if let Some(heading) = &location.heading {
        if !heading_exists_in_document(document, heading) {
            findings.push(LintFinding {
                code: "ORG018",
                severity: LintSeverity::Warning,
                message: format!("archive destination heading `{heading}` was not found"),
                location: location_for_range(source, location.ann.range),
            });
        }
    }
}

fn push_refile_destination_findings(
    document: &ParsedAst,
    source: &str,
    options: &LintOptions,
    findings: &mut Vec<LintFinding>,
) {
    let Some(base_dir) = &options.file_base_dir else {
        return;
    };

    for record in document.lifecycle_records() {
        let LifecycleRecordKind::Refile {
            target: Some(target),
            ..
        } = &record.kind
        else {
            continue;
        };
        let Some((file, heading)) = refile_file_target(target) else {
            continue;
        };
        if is_remote_file_path(file) {
            continue;
        }
        let path = resolve_lifecycle_path(base_dir, file);
        if let Some(message) = destination_file_message(path.as_path(), "refile", file) {
            findings.push(LintFinding {
                code: "ORG019",
                severity: LintSeverity::Warning,
                message,
                location: location_for_range(source, record.ann.range),
            });
            continue;
        }
        if let Some(heading) = heading {
            if !heading_exists_in_file(path.as_path(), heading) {
                findings.push(LintFinding {
                    code: "ORG019",
                    severity: LintSeverity::Warning,
                    message: format!("refile destination heading `{heading}` was not found"),
                    location: location_for_range(source, record.ann.range),
                });
            }
        }
    }
}

fn refile_file_target(target: &str) -> Option<(&str, Option<&str>)> {
    let inner = target.strip_prefix("[[")?.strip_suffix("]]")?;
    let path = inner
        .split_once("][")
        .map(|(path, _)| path)
        .unwrap_or(inner);
    let (protocol, rest) = path.split_once(':')?;
    if !matches!(
        protocol.to_ascii_lowercase().as_str(),
        "file" | "file+sys" | "file+emacs" | "file+shell"
    ) {
        return None;
    }
    let (file, heading) = rest
        .split_once("::")
        .map(|(file, heading)| (file, Some(heading)))
        .unwrap_or((rest, None));
    Some((file, heading))
}

fn destination_file_message(path: &Path, kind: &str, display: &str) -> Option<String> {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => None,
        Ok(_) => Some(format!(
            "{kind} destination `{display}` points at a directory"
        )),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            Some(format!("{kind} destination `{display}` was not found"))
        }
        Err(error) => Some(format!(
            "{kind} destination `{display}` could not be read: {error}"
        )),
    }
}

fn resolve_lifecycle_path(base_dir: &Path, file: &str) -> PathBuf {
    if file.starts_with('/') {
        PathBuf::from(file)
    } else if let Some(rest) = file.strip_prefix("~/") {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| base_dir.to_path_buf())
            .join(rest)
    } else {
        base_dir.join(file)
    }
}

fn is_remote_file_path(file: &str) -> bool {
    file.starts_with("/ssh:") || file.starts_with("/scp:")
}

fn heading_exists_in_file(path: &Path, heading: &str) -> bool {
    let Ok(source) = fs::read_to_string(path) else {
        return false;
    };
    let doc = Org::parse(&source).document();
    heading_exists_in_document(&doc, heading)
}

fn heading_exists_in_document(document: &ParsedAst, heading: &str) -> bool {
    let needle = normalize_heading_target(heading);
    document
        .sections
        .iter()
        .any(|section| section_or_subsection_matches(section, needle.as_str()))
}

fn section_or_subsection_matches(section: &Section<ParsedAnnotation>, needle: &str) -> bool {
    section.raw_title.trim() == needle
        || section
            .subsections
            .iter()
            .any(|subsection| section_or_subsection_matches(subsection, needle))
}

fn normalize_heading_target(heading: &str) -> String {
    heading.trim().trim_start_matches('*').trim().to_string()
}

fn push_archive_location_findings(
    document: &ParsedAst,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    for location in &document.archive_locations {
        if location.is_empty() {
            findings.push(LintFinding {
                code: "ORG015",
                severity: LintSeverity::Warning,
                message: "#+ARCHIVE keyword has no archive destination".to_string(),
                location: location_for_range(source, location.ann.range),
            });
        }
    }
    push_archive_property_findings(&document.properties, source, findings);
    document.visit(|node| match node {
        AstRef::Section(section) => {
            push_archive_property_findings(&section.properties, source, findings);
        }
        AstRef::Inlinetask(task) => {
            push_archive_property_findings(&task.properties, source, findings);
        }
        _ => {}
    });
}

fn push_archive_property_findings(
    properties: &[Property<ParsedAnnotation>],
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    for property in properties {
        if property.key.eq_ignore_ascii_case("ARCHIVE") && property.value.trim().is_empty() {
            findings.push(LintFinding {
                code: "ORG015",
                severity: LintSeverity::Warning,
                message: "ARCHIVE property has no archive destination".to_string(),
                location: location_for_range(source, property.ann.range),
            });
        }
    }
}
