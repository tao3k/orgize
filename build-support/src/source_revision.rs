//! Build-time source revision provenance.

use std::{
    collections::BTreeSet,
    env,
    path::{Path, PathBuf},
    process::Command,
};

/// Exposes the exact Git source revision and dirty state to the compiled artifact.
pub fn write_source_revision() {
    let root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let root = Path::new(&root);
    emit_git_rerun_inputs(root);
    let revision = git_output(root, &["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let dirty = git_dirty(root);
    println!("cargo:rerun-if-env-changed=ORGIZE_SOURCE_REVISION");
    let revision = env::var("ORGIZE_SOURCE_REVISION").unwrap_or(revision);
    println!("cargo:rustc-env=ORGIZE_SOURCE_REVISION={revision}");
    println!("cargo:rustc-env=ORGIZE_SOURCE_DIRTY={dirty}");
}

fn git_dirty(root: &Path) -> bool {
    git_output(root, &["status", "--porcelain", "--untracked-files=normal"])
        .is_some_and(|output| !output.is_empty())
}

fn emit_git_rerun_inputs(root: &Path) {
    let mut inputs = BTreeSet::new();
    for git_path in ["HEAD", "index", "packed-refs"] {
        if let Some(path) = git_output(root, &["rev-parse", "--git-path", git_path]) {
            inputs.insert(existing_rerun_input(root, Path::new(&path)));
        }
    }
    if let Some(reference) = git_output(root, &["symbolic-ref", "-q", "HEAD"])
        && let Some(path) = git_output(root, &["rev-parse", "--git-path", &reference])
    {
        inputs.insert(existing_rerun_input(root, Path::new(&path)));
    }
    if let Some(paths) = git_output(root, &["ls-files"]) {
        for path in paths.lines().filter(|path| !path.is_empty()) {
            inputs.insert(existing_rerun_input(root, Path::new(path)));
        }
    }
    for input in inputs {
        println!("cargo:rerun-if-changed={}", input.display());
    }
}

// Cargo treats an absent rerun input as perpetually dirty. Watching its nearest
// existing ancestor still detects restoration without resurrecting legacy files.
fn existing_rerun_input(root: &Path, input: &Path) -> PathBuf {
    let mut path = if input.is_absolute() {
        input.to_path_buf()
    } else {
        root.join(input)
    };
    while !path.exists() {
        if !path.pop() {
            return root.to_path_buf();
        }
    }
    path
}

fn git_output(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
#[path = "../tests/unit/source_revision.rs"]
mod tests;
