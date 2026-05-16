//! Opt-in filesystem-aware attachment inventory.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use super::{
    AttachmentInventory, AttachmentInventoryEntry, AttachmentInventoryEntryKind,
    AttachmentInventoryOptions, AttachmentInventoryWarning, AttachmentInventoryWarningKind,
    AttachmentVcsEvidence, AttachmentVcsStatus, Document, ParsedAnnotation,
};

impl Document<ParsedAnnotation> {
    /// Builds a filesystem-aware attachment inventory from semantic attachment
    /// records. This is opt-in and never mutates attachment directories.
    pub fn attachment_inventory(
        &self,
        options: &AttachmentInventoryOptions,
    ) -> AttachmentInventory {
        let sections = self.section_index_records();
        let mut inventory = AttachmentInventory::default();
        for section in sections {
            if let Some(directory) = section.attachment.directory {
                let path = directory.path;
                let absolute_path = absolute_path(options.base_dir.as_str(), path.as_str());
                let exists = absolute_path.exists();
                let vcs = vcs_evidence(options, &absolute_path);
                push_missing_warning(&mut inventory, exists, path.as_str());
                inventory.entries.push(AttachmentInventoryEntry {
                    source: section.source.clone(),
                    section_title: section.title.clone(),
                    kind: AttachmentInventoryEntryKind::Directory {
                        source: directory.source,
                    },
                    path,
                    absolute_path: absolute_path.display().to_string(),
                    exists,
                    vcs,
                });
            }
            for link in section.links.into_iter().filter_map(|link| link.attachment) {
                let path = link.path.clone();
                let absolute_path = absolute_path(options.base_dir.as_str(), path.as_str());
                let exists = absolute_path.exists();
                let vcs = vcs_evidence(options, &absolute_path);
                push_missing_warning(&mut inventory, exists, path.as_str());
                inventory.entries.push(AttachmentInventoryEntry {
                    source: section.source.clone(),
                    section_title: section.title.clone(),
                    kind: AttachmentInventoryEntryKind::Link { link },
                    path,
                    absolute_path: absolute_path.display().to_string(),
                    exists,
                    vcs,
                });
            }
        }
        inventory
    }
}

fn push_missing_warning(inventory: &mut AttachmentInventory, exists: bool, path: &str) {
    if exists {
        return;
    }
    inventory.warnings.push(AttachmentInventoryWarning {
        kind: AttachmentInventoryWarningKind::MissingPath,
        message: format!("attachment path `{path}` does not exist"),
    });
}

fn absolute_path(base_dir: &str, path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        Path::new(base_dir).join(path)
    }
}

fn vcs_evidence(options: &AttachmentInventoryOptions, path: &Path) -> AttachmentVcsEvidence {
    if !options.check_vcs {
        return AttachmentVcsEvidence::default();
    }
    let Some(root) = git_root(Path::new(options.base_dir.as_str())) else {
        return AttachmentVcsEvidence {
            status: AttachmentVcsStatus::NotInGitWorktree,
            raw: None,
        };
    };
    let Ok(relative) = path.strip_prefix(root.as_path()) else {
        return AttachmentVcsEvidence {
            status: AttachmentVcsStatus::NotInGitWorktree,
            raw: None,
        };
    };
    let output = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("status")
        .arg("--porcelain")
        .arg("--")
        .arg(relative)
        .output();
    let Ok(output) = output else {
        return AttachmentVcsEvidence {
            status: AttachmentVcsStatus::GitUnavailable,
            raw: None,
        };
    };
    if !output.status.success() {
        return AttachmentVcsEvidence {
            status: AttachmentVcsStatus::GitUnavailable,
            raw: Some(String::from_utf8_lossy(&output.stderr).trim().to_string()),
        };
    }
    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    AttachmentVcsEvidence {
        status: git_status(raw.as_str(), path.exists()),
        raw: (!raw.is_empty()).then_some(raw),
    }
}

fn git_root(base_dir: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(base_dir)
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!root.is_empty()).then(|| PathBuf::from(root))
}

fn git_status(raw: &str, exists: bool) -> AttachmentVcsStatus {
    if raw.is_empty() {
        return if exists {
            AttachmentVcsStatus::Clean
        } else {
            AttachmentVcsStatus::Missing
        };
    }
    if raw.starts_with("??") {
        AttachmentVcsStatus::Untracked
    } else if raw.starts_with(" D") || raw.starts_with("D ") {
        AttachmentVcsStatus::Missing
    } else {
        AttachmentVcsStatus::Modified
    }
}
