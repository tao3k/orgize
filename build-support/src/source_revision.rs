//! Build-time source revision provenance.

use std::{env, path::Path, process::Command};

/// Exposes the exact Git source revision and dirty state to the compiled artifact.
pub fn write_source_revision() {
    let root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let root = Path::new(&root);
    emit_git_rerun_inputs(root);
    let revision = git_output(root, &["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let dirty = git_output(root, &["status", "--porcelain", "--untracked-files=no"])
        .is_some_and(|output| !output.is_empty());
    println!("cargo:rerun-if-env-changed=ORGIZE_SOURCE_REVISION");
    let revision = env::var("ORGIZE_SOURCE_REVISION").unwrap_or(revision);
    println!("cargo:rustc-env=ORGIZE_SOURCE_REVISION={revision}");
    println!("cargo:rustc-env=ORGIZE_SOURCE_DIRTY={dirty}");
}

fn emit_git_rerun_inputs(root: &Path) {
    for git_path in ["HEAD", "index", "packed-refs"] {
        if let Some(path) = git_output(root, &["rev-parse", "--git-path", git_path]) {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    if let Some(reference) = git_output(root, &["symbolic-ref", "-q", "HEAD"])
        && let Some(path) = git_output(root, &["rev-parse", "--git-path", &reference])
    {
        println!("cargo:rerun-if-changed={path}");
    }
    if let Some(paths) = git_output(root, &["ls-files"]) {
        for path in paths.lines().filter(|path| !path.is_empty()) {
            println!("cargo:rerun-if-changed={}", root.join(path).display());
        }
    }
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
