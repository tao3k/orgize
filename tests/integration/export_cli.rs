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

#[test]
fn org_document_search_and_query_commands_run() {
    let guide = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("guide")
        .output()
        .expect("run orgize guide");
    assert!(guide.status.success());
    let guide_stdout = String::from_utf8(guide.stdout).expect("utf8 guide");
    assert!(guide_stdout.contains("[guide] lang=org"), "{guide_stdout}");

    let root = test_dir("org-document-search");
    let path = root.join("plan.org");
    std::fs::write(
        &path,
        "* Task\n:PROPERTIES:\n:CUSTOM_ID: task-1\n:END:\n\n#+begin_src rust\nfn main() {}\n#+end_src\n",
    )
    .expect("write org fixture");

    let search = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("search")
        .arg("prime")
        .arg("--view")
        .arg("seeds")
        .arg(&root)
        .output()
        .expect("run orgize search");
    assert!(search.status.success());
    let search_stdout = String::from_utf8(search.stdout).expect("utf8 search");
    assert!(
        search_stdout.contains("[search-prime] lang=org"),
        "{search_stdout}"
    );
    assert!(search_stdout.contains("|heading"), "{search_stdout}");
    assert!(
        search_stdout.contains("key=\"CUSTOM_ID\""),
        "{search_stdout}"
    );

    let selector = format!("{}:1-4", path.display());
    let query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("query")
        .arg("--selector")
        .arg(selector)
        .arg("--code")
        .output()
        .expect("run orgize query");
    assert!(query.status.success());
    let query_stdout = String::from_utf8(query.stdout).expect("utf8 query");
    assert!(
        query_stdout.contains(":CUSTOM_ID: task-1"),
        "{query_stdout}"
    );

    let selector = format!("{}:1-4", path.display());
    let selector_frontier = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("query")
        .arg("--selector")
        .arg(selector)
        .output()
        .expect("run orgize selector frontier query");
    assert!(selector_frontier.status.success());
    let selector_stdout = String::from_utf8(selector_frontier.stdout).expect("utf8 selector query");
    assert!(
        selector_stdout.contains("[query-selector] lang=org"),
        "{selector_stdout}"
    );
    assert!(
        selector_stdout.contains("code=\"orgize query --selector"),
        "{selector_stdout}"
    );

    let term_query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("query")
        .arg("--term")
        .arg("CUSTOM_ID")
        .arg(&root)
        .output()
        .expect("run orgize term query");
    assert!(term_query.status.success());
    let term_stdout = String::from_utf8(term_query.stdout).expect("utf8 term query");
    assert!(term_stdout.contains("[query] lang=org"), "{term_stdout}");
    assert!(term_stdout.contains("terms=1"), "{term_stdout}");
    assert!(term_stdout.contains("key=\"CUSTOM_ID\""), "{term_stdout}");
}

#[cfg(feature = "md")]
#[test]
fn markdown_document_search_and_query_commands_run() {
    let guide = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("md")
        .arg("guide")
        .output()
        .expect("run orgize md guide");
    assert!(guide.status.success());
    let guide_stdout = String::from_utf8(guide.stdout).expect("utf8 guide");
    assert!(guide_stdout.contains("[guide] lang=md"), "{guide_stdout}");

    let root = test_dir("md-document-search");
    let path = root.join("README.md");
    std::fs::write(
        &path,
        "# Project\n\n[site](https://example.com)\n\n```rust\nfn main() {}\n```\n",
    )
    .expect("write markdown fixture");

    let search = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("md")
        .arg("search")
        .arg("fzf")
        .arg("Project")
        .arg("owner")
        .arg("tests")
        .arg("--view")
        .arg("seeds")
        .arg(&root)
        .output()
        .expect("run orgize md search");
    assert!(
        search.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&search.stderr)
    );
    let search_stdout = String::from_utf8(search.stdout).expect("utf8 search");
    assert!(
        search_stdout.contains("[search-fzf] lang=md"),
        "{search_stdout}"
    );
    assert!(search_stdout.contains("|heading"), "{search_stdout}");

    let selector = format!("{}:1-1", path.display());
    let query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("md")
        .arg("query")
        .arg("--selector")
        .arg(selector)
        .arg("--code")
        .output()
        .expect("run orgize md query");
    assert!(query.status.success());
    let query_stdout = String::from_utf8(query.stdout).expect("utf8 query");
    assert_eq!(query_stdout, "# Project\n");

    let selector = format!("{}:1-1", path.display());
    let selector_frontier = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("md")
        .arg("query")
        .arg("--selector")
        .arg(selector)
        .output()
        .expect("run orgize md selector frontier query");
    assert!(selector_frontier.status.success());
    let selector_stdout = String::from_utf8(selector_frontier.stdout).expect("utf8 selector query");
    assert!(
        selector_stdout.contains("[query-selector] lang=md"),
        "{selector_stdout}"
    );
    assert!(
        selector_stdout.contains("code=\"orgize md query --selector"),
        "{selector_stdout}"
    );

    let term_query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("md")
        .arg("query")
        .arg("--term")
        .arg("Project")
        .arg(&root)
        .output()
        .expect("run orgize md term query");
    assert!(term_query.status.success());
    let term_stdout = String::from_utf8(term_query.stdout).expect("utf8 term query");
    assert!(term_stdout.contains("[query] lang=md"), "{term_stdout}");
    assert!(term_stdout.contains("terms=1"), "{term_stdout}");
    assert!(term_stdout.contains("|heading"), "{term_stdout}");
}

fn test_dir(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("orgize-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create test dir");
    root
}
