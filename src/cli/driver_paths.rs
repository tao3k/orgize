//! Shared filesystem and error rendering helpers for CLI drivers.

use std::{
    fs,
    io::{ErrorKind, Read},
    path::{Path, PathBuf},
};

pub(crate) fn read_stdin() -> Result<String, String> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| format!("failed to read stdin: {error}"))?;
    Ok(input)
}

pub(crate) fn collect_org_paths(paths: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for path in paths {
        collect_org_path(Path::new(path), &mut files)?;
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn collect_org_path(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|error| format_path_error(path, error))?;
    if metadata.is_file() {
        if !is_org_file(path) {
            return Err(format!("{}: expected .org file", display_path(path)));
        }
        files.push(path.to_path_buf());
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(format!("{}: unsupported path type", display_path(path)));
    }

    let mut entries = fs::read_dir(path)
        .map_err(|error| format_path_error(path, error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format_path_error(path, error))?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let entry_path = entry.path();
        let entry_type = entry
            .file_type()
            .map_err(|error| format_path_error(&entry_path, error))?;
        if entry_type.is_dir() {
            collect_org_path(&entry_path, files)?;
        } else if entry_type.is_file() && is_org_file(&entry_path) {
            files.push(entry_path);
        }
    }
    Ok(())
}

fn is_org_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("org"))
}

pub(crate) fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub(crate) fn format_path_error(path: &Path, error: std::io::Error) -> String {
    format!("{}: {}", display_path(path), stable_io_error(&error))
}

pub(crate) fn stable_io_error(error: &std::io::Error) -> String {
    match error.kind() {
        ErrorKind::NotFound => "No such file or directory (os error 2)".to_string(),
        _ => error.to_string(),
    }
}
