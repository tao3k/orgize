use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{
        AttachmentAnnexStatus, AttachmentArchiveDeletePolicy, AttachmentDirectorySource,
        AttachmentIdPathLayout, AttachmentInventoryOptions, AttachmentLinkSearchKind,
        AttachmentVcsStatus, ElementData, ObjectData,
    },
};

const SOURCE: &str = r#"* Project :ATTACH:
:PROPERTIES:
:DIR: assets
:END:
See [[attachment:diagram.png::255]].
** Child
See [[attachment:child.txt::*Heading]].
* ID backed
:PROPERTIES:
:ID: 95d50008-c12e-479f-a4f2-cc0238205319
:END:
See [[attachment:info.org::#custom]].
* Legacy
:PROPERTIES:
:ATTACH_DIR: legacy
:END:
See [[attachment:old.pdf::/needle/]].
"#;

const INVENTORY_SOURCE: &str = r#"* Project
:PROPERTIES:
:DIR: assets
:END:
See [[attachment:tracked.txt]], [[attachment:untracked.txt]], and [[attachment:missing.txt]].
* Rootless
See [[attachment:loose.txt]].
* Archived :ARCHIVE:
:PROPERTIES:
:DIR: archived
:END:
See [[attachment:old.txt]].
"#;

#[test]
fn semantic_ast_projects_attachment_directories_and_links() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let project = &doc.sections[0];
    assert!(project.attachment.has_attach_tag);
    let project_dir = project
        .attachment
        .directory
        .as_ref()
        .expect("project attachment directory");
    assert_eq!(project_dir.path, "assets");
    assert_eq!(project_dir.source, AttachmentDirectorySource::DirProperty);

    let project_link = first_section_link(project);
    let project_attachment = project_link.attachment.as_ref().expect("attachment link");
    assert_eq!(project_attachment.path, "diagram.png");
    assert_eq!(
        project_attachment
            .search
            .as_ref()
            .map(|search| (&search.raw, search.kind)),
        Some((&"255".to_string(), AttachmentLinkSearchKind::LineNumber))
    );

    let child = &project.subsections[0];
    assert_eq!(
        child
            .attachment
            .directory
            .as_ref()
            .map(|directory| directory.path.as_str()),
        Some("assets")
    );
    let child_attachment = first_section_link(child)
        .attachment
        .as_ref()
        .expect("child attachment link");
    assert_eq!(
        child_attachment
            .search
            .as_ref()
            .map(|search| (&search.raw, search.kind)),
        Some((&"*Heading".to_string(), AttachmentLinkSearchKind::Headline))
    );

    let id_backed = &doc.sections[1];
    let id_dir = id_backed
        .attachment
        .directory
        .as_ref()
        .expect("id attachment directory");
    assert_eq!(id_dir.path, "data/95/d50008-c12e-479f-a4f2-cc0238205319");
    assert!(matches!(
        &id_dir.source,
        AttachmentDirectorySource::IdDerived {
            id,
            layout: AttachmentIdPathLayout::Uuid,
        } if id == "95d50008-c12e-479f-a4f2-cc0238205319"
    ));

    let legacy = &doc.sections[2];
    assert_eq!(
        legacy
            .attachment
            .directory
            .as_ref()
            .map(|directory| (&directory.source, directory.path.as_str())),
        Some((
            &AttachmentDirectorySource::LegacyAttachDirProperty,
            "legacy"
        ))
    );
    assert_eq!(
        first_section_link(legacy)
            .attachment
            .as_ref()
            .and_then(|attachment| attachment.search.as_ref())
            .map(|search| search.kind),
        Some(AttachmentLinkSearchKind::Regexp)
    );

    insta::assert_debug_snapshot!(
        "semantic_ast__semantic_attachment_projection",
        doc.section_index_records()
    );
}

#[test]
fn semantic_ast_projects_attachment_inventory_resolves_directory_and_vcs() {
    let temp = unique_temp_dir("orgize-attachment-vcs");
    fs::create_dir_all(temp.join("assets")).expect("create attachment directory");
    fs::create_dir_all(temp.join("archived")).expect("create archived attachment directory");
    run_git(&temp, &["init", "--quiet"]);
    fs::write(temp.join("assets/tracked.txt"), b"tracked").expect("write tracked attachment");
    fs::write(temp.join("archived/old.txt"), b"old attachment").expect("write archived attachment");
    run_git(&temp, &["add", "assets/tracked.txt"]);
    run_git(
        &temp,
        &[
            "-c",
            "user.name=Orgize Test",
            "-c",
            "user.email=orgize@example.test",
            "commit",
            "--quiet",
            "-m",
            "track attachment",
        ],
    );
    fs::write(temp.join("assets/untracked.txt"), b"untracked").expect("write untracked attachment");

    let doc = Org::parse(INVENTORY_SOURCE).document();
    assert_clean_projection(&doc);
    let inventory = doc.attachment_inventory(
        &AttachmentInventoryOptions::new(path_str(&temp))
            .check_vcs(true)
            .check_annex(true)
            .archive_delete_policy(AttachmentArchiveDeletePolicy::Query),
    );

    let tracked = inventory_entry(&inventory, "tracked.txt");
    assert!(tracked.absolute_path.ends_with("assets/tracked.txt"));
    assert!(tracked.exists);
    assert_eq!(tracked.vcs.status, AttachmentVcsStatus::Clean);
    assert_eq!(
        tracked.vcs.annex.status,
        AttachmentAnnexStatus::NotAnnexRepository
    );
    let untracked = inventory_entry(&inventory, "untracked.txt");
    assert_eq!(untracked.vcs.status, AttachmentVcsStatus::Untracked);
    let missing = inventory_entry(&inventory, "missing.txt");
    assert!(!missing.exists);
    assert_eq!(missing.vcs.status, AttachmentVcsStatus::Missing);
    assert!(inventory.warnings.iter().any(|warning| {
        warning.kind.as_str() == "missingDirectory" && warning.message.contains("loose.txt")
    }));
    assert!(inventory.archive_advice.iter().any(|advice| {
        advice.section_title == "Archived"
            && advice.path == "archived"
            && advice.policy == AttachmentArchiveDeletePolicy::Query
    }));

    insta::assert_snapshot!(
        "semantic_ast__semantic_attachment_inventory_vcs",
        render_attachment_inventory(&inventory, &temp)
    );
    let _ = fs::remove_dir_all(temp);
}

fn first_section_link(
    section: &orgize::ast::Section<orgize::ast::ParsedAnnotation>,
) -> &orgize::ast::Link<orgize::ast::ParsedAnnotation> {
    match &section.children[0].data {
        ElementData::Paragraph(objects) => objects
            .iter()
            .find_map(|object| match &object.data {
                ObjectData::Link(link) => Some(link),
                _ => None,
            })
            .expect("attachment link"),
        other => panic!("expected paragraph, got {other:#?}"),
    }
}

fn inventory_entry<'a>(
    inventory: &'a orgize::ast::AttachmentInventory,
    path: &str,
) -> &'a orgize::ast::AttachmentInventoryEntry {
    inventory
        .entries
        .iter()
        .find(|entry| entry.path == path)
        .unwrap_or_else(|| panic!("missing attachment inventory entry for {path}"))
}

fn render_attachment_inventory(
    inventory: &orgize::ast::AttachmentInventory,
    base_dir: &Path,
) -> String {
    let mut out = String::new();
    for entry in &inventory.entries {
        out.push_str(&format!(
            "entry {} title={} path={} absolute={} exists={} vcs={} annex={}\n",
            entry.kind.as_str(),
            entry.section_title,
            entry.path,
            relative_path(&entry.absolute_path, base_dir),
            entry.exists,
            entry.vcs.status.as_str(),
            entry.vcs.annex.status.as_str()
        ));
        if let Some(raw) = &entry.vcs.raw {
            out.push_str(&format!("  raw={}\n", raw.replace('\n', "\\n")));
        }
    }
    for warning in &inventory.warnings {
        out.push_str(&format!(
            "warning {} {}\n",
            warning.kind.as_str(),
            warning.message
        ));
    }
    for advice in &inventory.archive_advice {
        out.push_str(&format!(
            "archive {} title={} path={} {}\n",
            advice.policy.as_str(),
            advice.section_title,
            advice.path,
            advice.message
        ));
    }
    out
}

fn relative_path(path: &str, base_dir: &Path) -> String {
    Path::new(path)
        .strip_prefix(base_dir)
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| path.to_string())
}

fn run_git(dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("run git command");
    assert!(
        output.status.success(),
        "git {args:?} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{label}-{pid}-{nanos}"))
}

fn path_str(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
