//! Opt-in filesystem-aware attachment inventory.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use super::{
    AttachmentAnnexEvidence, AttachmentAnnexStatus, AttachmentArchiveAdvice,
    AttachmentArchiveDeletePolicy, AttachmentDirectorySource, AttachmentDisplayAbsolutePath,
    AttachmentDisplayDirectoryPath, AttachmentDisplayId, AttachmentDisplayLinkPath,
    AttachmentDisplayMediaKind, AttachmentDisplayRecord, AttachmentIdPathLayout,
    AttachmentInventory, AttachmentInventoryEntry, AttachmentInventoryEntryKind,
    AttachmentInventoryOptions, AttachmentInventoryWarning, AttachmentInventoryWarningKind,
    AttachmentSyncAction, AttachmentSyncActionKind, AttachmentVcsEvidence, AttachmentVcsStatus,
    Document, ParsedAnnotation, SectionIndexRecord, SectionIndexSource,
};

impl Document<ParsedAnnotation> {
    /// Builds a filesystem-aware attachment inventory from semantic attachment
    /// records. This is opt-in and never mutates attachment directories.
    pub fn attachment_inventory(
        &self,
        options: &AttachmentInventoryOptions,
    ) -> AttachmentInventory {
        build_attachment_inventory(options, self.section_index_records())
    }
}

fn build_attachment_inventory(
    options: &AttachmentInventoryOptions,
    sections: Vec<SectionIndexRecord>,
) -> AttachmentInventory {
    let mut inventory = AttachmentInventory::default();
    let mut directory_links: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut directory_scan_records: BTreeMap<String, DirectoryScanRecord> = BTreeMap::new();
    for section in sections {
        let directory_path = section.attachment.directory.as_ref().map(|directory| {
            attachment_directory_path(options, &directory.source, &directory.path)
        });
        let directory_absolute_path = directory_path
            .as_ref()
            .map(|path| absolute_path(options.base_dir.as_str(), path.as_str()));
        if let Some(directory) = section.attachment.directory.as_ref() {
            let path = attachment_directory_path(options, &directory.source, &directory.path);
            let absolute_path = absolute_path(options.base_dir.as_str(), path.as_str());
            let exists = absolute_path.exists();
            let vcs = vcs_evidence(options, &absolute_path);
            push_missing_warning(&mut inventory, exists, path.as_str());
            push_missing_directory_action(
                &mut inventory,
                exists,
                &section,
                path.as_str(),
                Some(absolute_path.display().to_string()),
            );
            push_archive_delete_advice(&mut inventory, options, &section, path.as_str());
            directory_scan_records
                .entry(path.clone())
                .or_insert_with(|| DirectoryScanRecord {
                    source: section.source.clone(),
                    section_title: section.title.clone(),
                    path: path.clone(),
                    absolute_path: absolute_path.clone(),
                    has_attach_tag: section.attachment.has_attach_tag,
                });
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
        for link in section
            .links
            .iter()
            .filter_map(|link| link.attachment.as_ref())
        {
            let path = link.path.clone();
            let (absolute_path, exists, vcs) = if let Some(directory_path) = &directory_path {
                let absolute_path = directory_absolute_path
                    .as_ref()
                    .expect("directory absolute path")
                    .join(path.as_str());
                let exists = absolute_path.exists();
                let vcs = vcs_evidence(options, &absolute_path);
                push_missing_warning(&mut inventory, exists, path.as_str());
                push_missing_link_action(
                    &mut inventory,
                    exists,
                    &section,
                    path.as_str(),
                    absolute_path.display().to_string(),
                );
                directory_links
                    .entry(directory_path.clone())
                    .or_default()
                    .insert(path.clone());
                inventory.display.push(AttachmentDisplayRecord {
                    source: section.source.clone(),
                    section_title: section.title.clone(),
                    section_title_text: section.title_text.clone(),
                    outline_path: section.outline_path.clone(),
                    outline_path_text: section.outline_path_text.clone(),
                    tags: section.tags.clone(),
                    effective_tags: section.effective_tags.clone(),
                    attachment_id: attachment_id(&section),
                    directory_path: AttachmentDisplayDirectoryPath::new(directory_path.clone()),
                    link_path: AttachmentDisplayLinkPath::new(path.clone()),
                    absolute_path: AttachmentDisplayAbsolutePath::new(
                        absolute_path.display().to_string(),
                    ),
                    exists,
                    media_kind: media_kind(path.as_str()),
                });
                (absolute_path, exists, vcs)
            } else {
                push_missing_directory_warning(&mut inventory, path.as_str());
                push_unresolved_directory_action(&mut inventory, &section, path.as_str());
                (
                    absolute_path(options.base_dir.as_str(), path.as_str()),
                    false,
                    AttachmentVcsEvidence::default(),
                )
            };
            inventory.entries.push(AttachmentInventoryEntry {
                source: section.source.clone(),
                section_title: section.title.clone(),
                kind: AttachmentInventoryEntryKind::Link { link: link.clone() },
                path,
                absolute_path: absolute_path.display().to_string(),
                exists,
                vcs,
            });
        }
    }
    if options.scan_orphans {
        push_directory_sync_actions(&mut inventory, &directory_scan_records, &directory_links);
    }
    inventory
}

#[derive(Clone, Debug)]
struct DirectoryScanRecord {
    source: SectionIndexSource,
    section_title: String,
    path: String,
    absolute_path: PathBuf,
    has_attach_tag: bool,
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

fn push_missing_directory_action(
    inventory: &mut AttachmentInventory,
    exists: bool,
    section: &SectionIndexRecord,
    path: &str,
    absolute_path: Option<String>,
) {
    if exists {
        return;
    }
    inventory.sync_plan.actions.push(AttachmentSyncAction {
        kind: AttachmentSyncActionKind::MissingDirectory,
        source: section.source.clone(),
        section_title: section.title.clone(),
        path: path.to_string(),
        absolute_path,
        message: format!(
            "section `{}` references missing attachment directory `{path}`",
            section.title
        ),
    });
}

fn push_unresolved_directory_action(
    inventory: &mut AttachmentInventory,
    section: &SectionIndexRecord,
    link_path: &str,
) {
    inventory.sync_plan.actions.push(AttachmentSyncAction {
        kind: AttachmentSyncActionKind::MissingDirectory,
        source: section.source.clone(),
        section_title: section.title.clone(),
        path: link_path.to_string(),
        absolute_path: None,
        message: format!(
            "section `{}` has attachment link `{link_path}` without a resolved attachment directory",
            section.title
        ),
    });
}

fn push_missing_link_action(
    inventory: &mut AttachmentInventory,
    exists: bool,
    section: &SectionIndexRecord,
    path: &str,
    absolute_path: String,
) {
    if exists {
        return;
    }
    inventory.sync_plan.actions.push(AttachmentSyncAction {
        kind: AttachmentSyncActionKind::MissingLinkedFile,
        source: section.source.clone(),
        section_title: section.title.clone(),
        path: path.to_string(),
        absolute_path: Some(absolute_path),
        message: format!(
            "section `{}` references missing attachment file `{path}`",
            section.title
        ),
    });
}

fn push_directory_sync_actions(
    inventory: &mut AttachmentInventory,
    directories: &BTreeMap<String, DirectoryScanRecord>,
    directory_links: &BTreeMap<String, BTreeSet<String>>,
) {
    for (directory_path, directory) in directories {
        if !directory.absolute_path.is_dir() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&directory.absolute_path) else {
            continue;
        };
        let linked = directory_links.get(directory_path);
        let mut has_file = false;
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_file() {
                continue;
            }
            has_file = true;
            let name = entry.file_name().to_string_lossy().into_owned();
            if linked.is_some_and(|linked| linked.contains(name.as_str())) {
                continue;
            }
            let absolute_path = entry.path().display().to_string();
            inventory.sync_plan.actions.push(AttachmentSyncAction {
                kind: AttachmentSyncActionKind::OrphanFile,
                source: directory.source.clone(),
                section_title: directory.section_title.clone(),
                path: join_relative(directory.path.as_str(), name.as_str()),
                absolute_path: Some(absolute_path),
                message: format!(
                    "attachment file `{name}` is present under `{}` but no section link references it",
                    directory.path
                ),
            });
        }
        if !has_file {
            inventory.sync_plan.actions.push(AttachmentSyncAction {
                kind: AttachmentSyncActionKind::EmptyDirectory,
                source: directory.source.clone(),
                section_title: directory.section_title.clone(),
                path: directory.path.clone(),
                absolute_path: Some(directory.absolute_path.display().to_string()),
                message: format!(
                    "attachment directory `{}` has no direct files",
                    directory.path
                ),
            });
            if directory.has_attach_tag {
                inventory.sync_plan.actions.push(AttachmentSyncAction {
                    kind: AttachmentSyncActionKind::StaleAttachTag,
                    source: directory.source.clone(),
                    section_title: directory.section_title.clone(),
                    path: directory.path.clone(),
                    absolute_path: Some(directory.absolute_path.display().to_string()),
                    message: format!(
                        "section `{}` keeps :ATTACH: but `{}` has no direct files",
                        directory.section_title, directory.path
                    ),
                });
            }
        }
    }
}

fn attachment_directory_path(
    options: &AttachmentInventoryOptions,
    source: &AttachmentDirectorySource,
    path: &str,
) -> String {
    match source {
        AttachmentDirectorySource::IdDerived { id, layout } => join_relative(
            options.attach_id_dir.as_str(),
            attachment_id_suffix(id, *layout).as_str(),
        ),
        AttachmentDirectorySource::DirProperty | AttachmentDirectorySource::AttachDirProperty => {
            path.to_string()
        }
    }
}

fn attachment_id(section: &SectionIndexRecord) -> Option<AttachmentDisplayId> {
    section
        .effective_properties
        .iter()
        .find(|property| property.key.eq_ignore_ascii_case("ID"))
        .map(|property| property.value.trim())
        .filter(|value| !value.is_empty())
        .map(AttachmentDisplayId::new)
}

fn attachment_id_suffix(id: &str, layout: AttachmentIdPathLayout) -> String {
    match layout {
        AttachmentIdPathLayout::Uuid => split_after_chars(id, 2)
            .map(|(prefix, suffix)| format!("{prefix}/{suffix}"))
            .unwrap_or_else(|| id.to_string()),
        AttachmentIdPathLayout::Timestamp => split_after_chars(id, 6)
            .map(|(prefix, suffix)| format!("{prefix}/{suffix}"))
            .unwrap_or_else(|| id.to_string()),
        AttachmentIdPathLayout::Fallback => split_after_chars(id, 1)
            .map(|(prefix, _)| format!("__/{prefix}/{id}"))
            .unwrap_or_else(|| id.to_string()),
    }
}

fn split_after_chars(value: &str, count: usize) -> Option<(&str, &str)> {
    if count == 0 {
        return Some(("", value));
    }
    let index = value
        .char_indices()
        .nth(count)
        .map(|(index, _)| index)
        .or_else(|| (value.chars().count() == count).then_some(value.len()))?;
    Some(value.split_at(index))
}

fn join_relative(root: &str, suffix: &str) -> String {
    let root = root.trim_end_matches('/');
    let suffix = suffix.trim_start_matches('/');
    if root.is_empty() {
        suffix.to_string()
    } else if suffix.is_empty() {
        root.to_string()
    } else {
        format!("{root}/{suffix}")
    }
}

fn media_kind(path: &str) -> AttachmentDisplayMediaKind {
    let Some(extension) = Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
    else {
        return AttachmentDisplayMediaKind::Other;
    };
    match extension.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "avif" | "svg" | "bmp" | "tif" | "tiff" => {
            AttachmentDisplayMediaKind::Image
        }
        "mp4" | "webm" | "mov" | "m4v" | "mkv" => AttachmentDisplayMediaKind::Video,
        "mp3" | "flac" | "wav" | "ogg" | "m4a" => AttachmentDisplayMediaKind::Audio,
        "pdf" => AttachmentDisplayMediaKind::Pdf,
        _ => AttachmentDisplayMediaKind::Other,
    }
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
