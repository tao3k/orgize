use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn export_md_reads_stdin_and_writes_markdown() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("export")
        .arg("md")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn orgize export md");

    child
        .stdin
        .as_mut()
        .expect("open stdin")
        .write_all(b"* Task\n:PROPERTIES:\n:CUSTOM_ID: task-1\n:END:\n")
        .expect("write org input");

    let output = child.wait_with_output().expect("read orgize output");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("# Task"), "{stdout}");
    assert!(stdout.contains("| Key | Value |"), "{stdout}");
    assert!(stdout.contains("| CUSTOM_ID | task-1 |"), "{stdout}");
}
