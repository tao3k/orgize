//! Opt-in filesystem-aware attachment inventory.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use super::{
    AttachmentAnnexEvidence, AttachmentAnnexStatus, AttachmentArchiveAdvice,
    AttachmentArchiveDeletePolicy, AttachmentInventory, AttachmentInventoryEntry,
    AttachmentInventoryEntryKind, AttachmentInventoryOptions, AttachmentInventoryWarning,
    AttachmentInventoryWarningKind, AttachmentVcsEvidence, AttachmentVcsStatus, Document,
    ParsedAnnotation, SectionIndexRecord,
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
            let directory_path =
                section.attachment.directory.as_ref().map(|directory| {
                    absolute_path(options.base_dir.as_str(), directory.path.as_str())
                });
            if let Some(directory) = section.attachment.directory.as_ref() {
                let path = directory.path.clone();
                let absolute_path = absolute_path(options.base_dir.as_str(), path.as_str());
                let exists = absolute_path.exists();
                let vcs = vcs_evidence(options, &absolute_path);
                push_missing_warning(&mut inventory, exists, path.as_str());
                push_archive_delete_advice(&mut inventory, options, &section, path.as_str());
                inventory.entries.push(AttachmentInventoryEntry {
                    source: section.source.clone(),
                    section_title: section.title.clone(),
                    kind: AttachmentInventoryEntryKind::Directory {
                        source: directory.source.clone(),
                    },
                    path,
                    absolute_path: absolute_path.display().to_string(),
                    exists,
                    vcs,
                });
            }
            for link in section.links.into_iter().filter_map(|link| link.attachment) {
                let path = link.path.clone();
                let (absolute_path, exists, vcs) = if let Some(directory_path) = &directory_path {
                    let absolute_path = directory_path.join(path.as_str());
                    let exists = absolute_path.exists();
                    let vcs = vcs_evidence(options, &absolute_path);
                    push_missing_warning(&mut inventory, exists, path.as_str());
                    (absolute_path, exists, vcs)
                } else {
                    push_missing_directory_warning(&mut inventory, path.as_str());
                    (
                        absolute_path(options.base_dir.as_str(), path.as_str()),
                        false,
                        AttachmentVcsEvidence::default(),
                    )
                };
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

fn push_archive_delete_advice(
    inventory: &mut AttachmentInventory,
    options: &AttachmentInventoryOptions,
    section: &SectionIndexRecord,
    path: &str,
) {
    if !section.archive.archived {
        return;
    }
    let policy = options.archive_delete_policy;
    if matches!(
        policy,
        AttachmentArchiveDeletePolicy::NotConfigured | AttachmentArchiveDeletePolicy::Never
    ) {
        return;
    }
    let action = match policy {
        AttachmentArchiveDeletePolicy::Query => "may ask before deleting",
        AttachmentArchiveDeletePolicy::Always => "may delete",
        AttachmentArchiveDeletePolicy::NotConfigured | AttachmentArchiveDeletePolicy::Never => {
            return;
        }
    };
    inventory.archive_advice.push(AttachmentArchiveAdvice {
        source: section.source.clone(),
        section_title: section.title.clone(),
        policy,
        path: path.to_string(),
        message: format!(
            "archived section `{}` {action} attachment directory `{path}`",
            section.title
        ),
    });
}

fn push_missing_directory_warning(inventory: &mut AttachmentInventory, path: &str) {
    inventory.warnings.push(AttachmentInventoryWarning {
        kind: AttachmentInventoryWarningKind::MissingDirectory,
        message: format!("attachment link `{path}` has no resolved attachment directory"),
    });
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
            annex: AttachmentAnnexEvidence::default(),
            raw: None,
        };
    };
    let Some(relative) = git_relative_path(path, root.as_path()) else {
        return AttachmentVcsEvidence {
            status: AttachmentVcsStatus::NotInGitWorktree,
            annex: AttachmentAnnexEvidence::default(),
            raw: None,
        };
    };
    let output = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("status")
        .arg("--porcelain")
        .arg("--")
        .arg(&relative)
        .output();
    let Ok(output) = output else {
        return AttachmentVcsEvidence {
            status: AttachmentVcsStatus::GitUnavailable,
            annex: AttachmentAnnexEvidence::default(),
            raw: None,
        };
    };
    if !output.status.success() {
        return AttachmentVcsEvidence {
            status: AttachmentVcsStatus::GitUnavailable,
            annex: AttachmentAnnexEvidence::default(),
            raw: Some(String::from_utf8_lossy(&output.stderr).trim().to_string()),
        };
    }
    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    AttachmentVcsEvidence {
        status: git_status(raw.as_str(), path.exists()),
        annex: annex_evidence(options, root.as_path(), &relative),
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

fn git_relative_path(path: &Path, root: &Path) -> Option<PathBuf> {
    if let Ok(relative) = path.strip_prefix(root) {
        return Some(relative.to_path_buf());
    }
    let root = root.canonicalize().ok()?;
    let path = canonical_path_for_git(path)?;
    path.strip_prefix(root).ok().map(Path::to_path_buf)
}

fn canonical_path_for_git(path: &Path) -> Option<PathBuf> {
    if let Ok(path) = path.canonicalize() {
        return Some(path);
    }
    let parent = path.parent()?.canonicalize().ok()?;
    Some(parent.join(path.file_name()?))
}

fn annex_evidence(
    options: &AttachmentInventoryOptions,
    root: &Path,
    relative: &Path,
) -> AttachmentAnnexEvidence {
    if !options.check_annex {
        return AttachmentAnnexEvidence::default();
    }
    if !is_annex_repository(root) {
        return AttachmentAnnexEvidence {
            status: AttachmentAnnexStatus::NotAnnexRepository,
            raw: None,
        };
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("annex")
        .arg("find")
        .arg("--format=found")
        .arg("--in=here")
        .arg("--")
        .arg(relative)
        .output();
    let Ok(output) = output else {
        return AttachmentAnnexEvidence {
            status: AttachmentAnnexStatus::GitAnnexUnavailable,
            raw: None,
        };
    };
    if !output.status.success() {
        return AttachmentAnnexEvidence {
            status: AttachmentAnnexStatus::GitAnnexUnavailable,
            raw: Some(String::from_utf8_lossy(&output.stderr).trim().to_string()),
        };
    }
    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let status = match raw.as_str() {
        "found" => AttachmentAnnexStatus::Present,
        "" => AttachmentAnnexStatus::Missing,
        _ => AttachmentAnnexStatus::Unknown,
    };
    AttachmentAnnexEvidence {
        status,
        raw: (!raw.is_empty()).then_some(raw),
    }
}

fn is_annex_repository(root: &Path) -> bool {
    root.join("annex").exists()
        || git_dir(root)
            .map(|git_dir| git_dir.join("annex").exists())
            .unwrap_or(false)
}

fn git_dir(root: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("rev-parse")
        .arg("--git-dir")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if raw.is_empty() {
        return None;
    }
    let path = PathBuf::from(raw);
    Some(if path.is_absolute() {
        path
    } else {
        root.join(path)
    })
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
