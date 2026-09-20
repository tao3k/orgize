//! Build-time source revision provenance.

use std::{env, fs, path::Path, process::Command};

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
    let mut features = env::vars()
        .filter_map(|(key, _)| key.strip_prefix("CARGO_FEATURE_").map(str::to_owned))
        .collect::<Vec<_>>();
    features.sort();
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    let profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_string());
    let source_digest = source_content_digest(root);
    println!(
        "cargo:rustc-env=ORGIZE_BUILD_IDENTITY=version={version};revision={revision};dirty={dirty};source={source_digest};target={target};profile={profile};features={}",
        features.join(",")
    );
}

fn source_content_digest(root: &Path) -> String {
    let mut paths = Vec::new();
    for relative in ["src", "contracts", "build-support/src"] {
        collect_files(&root.join(relative), &mut paths);
    }
    for relative in [
        "Cargo.toml",
        "Cargo.lock",
        "build.rs",
        "build-support/Cargo.toml",
    ] {
        let path = root.join(relative);
        if path.is_file() {
            paths.push(path);
        }
    }
    paths.sort();
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"orgize.parser-source.v1\0");
    for path in paths {
        println!("cargo:rerun-if-changed={}", path.display());
        let relative = path.strip_prefix(root).unwrap_or(&path).to_string_lossy();
        let source = fs::read(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        hasher.update(&(relative.len() as u64).to_be_bytes());
        hasher.update(relative.as_bytes());
        hasher.update(&(source.len() as u64).to_be_bytes());
        hasher.update(&source);
    }
    hasher.finalize().to_hex().to_string()
}

fn collect_files(directory: &Path, files: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    let mut entries = entries
        .map(|entry| entry.expect("read source directory entry").path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_files(&path, files);
        } else if path.is_file() {
            files.push(path);
        }
    }
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
