use super::{existing_rerun_input, git_dirty};
use std::fs;
use std::process::Command;

#[test]
fn absent_legacy_input_tracks_existing_ancestor_and_then_restoration() {
    let root = std::env::temp_dir().join(format!("orgize-source-rerun-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    fs::create_dir_all(root.join("bindings/python")).unwrap();
    let removed = root.join("bindings/python/build.rs");
    assert_eq!(
        existing_rerun_input(&root, &removed),
        root.join("bindings/python")
    );
    assert_eq!(
        existing_rerun_input(&root, std::path::Path::new("bindings/python/build.rs")),
        root.join("bindings/python")
    );
    fs::write(&removed, "// restored input").unwrap();
    assert_eq!(existing_rerun_input(&root, &removed), removed);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn untracked_source_is_dirty_but_ignored_artifacts_are_not() {
    let root = std::env::temp_dir().join(format!("orgize-source-dirty-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    assert!(
        Command::new("git")
            .args(["init", "--quiet"])
            .arg(&root)
            .status()
            .unwrap()
            .success()
    );
    assert!(!git_dirty(&root));

    fs::write(root.join(".git/info/exclude"), "ignored.scm\n").unwrap();
    fs::write(root.join("ignored.scm"), "; generated artifact\n").unwrap();
    assert!(!git_dirty(&root));

    let source = root.join("native-owner.ss");
    fs::write(&source, "; new semantic owner\n").unwrap();
    assert!(git_dirty(&root));
    fs::remove_file(&source).unwrap();
    assert!(!git_dirty(&root));

    fs::write(&source, "; staged semantic owner\n").unwrap();
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(["add", "native-owner.ss"])
            .status()
            .unwrap()
            .success()
    );
    assert!(git_dirty(&root));
    fs::remove_dir_all(root).unwrap();
}
