//! Ordinary file-link lint checks.

use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use crate::ast::{AstRef, FileLinkPathKind, ObjectData, ParsedAst};

use super::lint_model::{LintFinding, LintOptions, LintSeverity, location_for_range};

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
    findings
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
