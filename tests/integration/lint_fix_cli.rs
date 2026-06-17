use std::{fs, path::PathBuf, process::Command};

#[test]
fn lint_fix_formats_org_files_before_linting() {
    let dir = test_dir("lint-fix-format");
    let path = dir.join("notes.org");
    fs::write(&path, "* Table\n|a|bb|\n|long|c|\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .args(["lint", "--fix", path.to_str().unwrap()])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "[ok] orgize lint\n"
    );
    let fixed = fs::read_to_string(path).unwrap();
    assert!(fixed.contains("| a    | bb |"), "{fixed}");
    assert!(fixed.contains("| long | c  |"), "{fixed}");
}

#[test]
fn lint_fix_rejects_stdin() {
    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .args(["lint", "--fix"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("lint --fix requires at least one Org file or directory path"),
        "{stderr}"
    );
}

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("orgize-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}
