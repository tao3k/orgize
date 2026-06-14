use std::{
    io::Write,
    process::{Command, Stdio},
};

use serde_json::Value;

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
    assert!(!guide_stdout.contains("owner tests"), "{guide_stdout}");
    assert!(
        guide_stdout.contains("|query-axis field matches=key-or-key=value value-match=contains"),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains("|field-map heading fields=level,title,todo,todoType,priority,tag"),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains("|field-map block fields=kind=source|export,lang,backend"),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains(
            "|recipe paragraph-content=asp org query --kind paragraph --term <term> --content ."
        ),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains("|cmd search-toc=asp org search toc ."),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains(
            "|cmd elements-query=asp org elements-query --packet <json-query-packet> <org-file>"
        ),
        "{guide_stdout}"
    );

    let root = test_dir("org-document-search");
    let path = root.join("plan.org");
    std::fs::write(
        &path,
        "* TODO [#A] Task :work:\nSCHEDULED: <2026-06-06 Sat>\n:PROPERTIES:\n:CUSTOM_ID: task-1\n:END:\n\nProvider activation carries execution mode.\nDocument providers stay embedded inside ASP.\n\n** Repository Map\n*** Docs\n- [X] ship element map\n[[https://example.com][site]]\n[[file:diagram.png]]\n\n#+begin_src rust\nfn main() {}\n#+end_src\n",
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
    assert!(
        search_stdout.contains("sourceKind=\"Headline\""),
        "{search_stdout}"
    );
    assert!(search_stdout.contains("|planning"), "{search_stdout}");
    assert!(search_stdout.contains("|paragraph"), "{search_stdout}");
    assert!(search_stdout.contains("execution mode"), "{search_stdout}");
    assert!(search_stdout.contains("|task"), "{search_stdout}");
    assert!(search_stdout.contains("|link"), "{search_stdout}");
    assert!(search_stdout.contains("|image"), "{search_stdout}");

    let toc = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("search")
        .arg("toc")
        .arg(&root)
        .output()
        .expect("run orgize toc search");
    assert!(toc.status.success());
    let toc_stdout = String::from_utf8(toc.stdout).expect("utf8 toc");
    assert!(toc_stdout.contains("[search-toc] lang=org"), "{toc_stdout}");
    assert!(toc_stdout.contains("heading=3"), "{toc_stdout}");
    assert!(toc_stdout.contains("maxLevel=3"), "{toc_stdout}");
    assert!(toc_stdout.contains("|doc path="), "{toc_stdout}");
    assert!(
        toc_stdout.contains("level=1 title=\"Task\" todo=\"TODO\""),
        "{toc_stdout}"
    );
    assert!(
        toc_stdout.contains("level=2 title=\"Repository Map\""),
        "{toc_stdout}"
    );
    assert!(
        toc_stdout.contains("level=3 title=\"Docs\""),
        "{toc_stdout}"
    );
    assert!(
        toc_stdout.contains("next=\"asp org query --selector"),
        "{toc_stdout}"
    );

    let selector = format!("{}:1-4", path.display());
    let query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("query")
        .arg("--from-hook")
        .arg("direct-source-read")
        .arg("--selector")
        .arg(selector)
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
        selector_stdout
            .contains("direct-read=\"asp org query --from-hook direct-source-read --selector"),
        "{selector_stdout}"
    );
    assert!(selector_stdout.contains("|heading"), "{selector_stdout}");
    assert!(
        selector_stdout.contains("key=\"CUSTOM_ID\""),
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

    let paragraph_query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("query")
        .arg("--term")
        .arg("embedded")
        .arg(&root)
        .output()
        .expect("run orgize paragraph term query");
    assert!(paragraph_query.status.success());
    let paragraph_stdout = String::from_utf8(paragraph_query.stdout).expect("utf8 paragraph query");
    assert!(
        paragraph_stdout.contains("[query] lang=org"),
        "{paragraph_stdout}"
    );
    assert!(
        paragraph_stdout.contains("|paragraph"),
        "{paragraph_stdout}"
    );
    assert!(
        paragraph_stdout.contains("embedded inside ASP"),
        "{paragraph_stdout}"
    );

    let paragraph_content = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("query")
        .arg("--term")
        .arg("embedded")
        .arg("--content")
        .arg(&root)
        .output()
        .expect("run orgize paragraph content query");
    assert!(paragraph_content.status.success());
    let paragraph_content_stdout =
        String::from_utf8(paragraph_content.stdout).expect("utf8 paragraph content query");
    assert_eq!(
        paragraph_content_stdout.trim(),
        "Provider activation carries execution mode. Document providers stay embedded inside ASP.",
        "{paragraph_content_stdout}"
    );

    let missing_content = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("query")
        .arg("--term")
        .arg("missing-content")
        .arg("--content")
        .arg(&root)
        .output()
        .expect("run orgize missing content query");
    assert!(missing_content.status.success());
    let missing_content_stdout =
        String::from_utf8(missing_content.stdout).expect("utf8 missing content query");
    assert_eq!(missing_content_stdout, "", "{missing_content_stdout}");

    let broad_content = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("query")
        .arg("--content")
        .arg(&root)
        .output()
        .expect("run orgize broad content query");
    assert!(!broad_content.status.success());
    let broad_content_stderr =
        String::from_utf8(broad_content.stderr).expect("utf8 broad content stderr");
    assert!(
        broad_content_stderr.contains("--content requires --selector, --term, --kind, or --field"),
        "{broad_content_stderr}"
    );

    let selector = format!("{}:1-4", path.display());
    let direct_read_content = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("query")
        .arg("--from-hook")
        .arg("direct-source-read")
        .arg("--selector")
        .arg(selector)
        .arg("--content")
        .output()
        .expect("run orgize direct-read content query");
    assert!(direct_read_content.status.success());
    let direct_read_content_stdout =
        String::from_utf8(direct_read_content.stdout).expect("utf8 direct-read content stdout");
    assert!(
        direct_read_content_stdout.contains("* TODO [#A] Task :work:"),
        "{direct_read_content_stdout}"
    );
    assert!(
        direct_read_content_stdout.contains(":CUSTOM_ID: task-1"),
        "{direct_read_content_stdout}"
    );

    let json_search = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("search")
        .arg("prime")
        .arg("--view")
        .arg("seeds")
        .arg("--json")
        .arg(&root)
        .output()
        .expect("run orgize search json");
    assert!(
        json_search.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&json_search.stderr)
    );
    let search_packet: Value =
        serde_json::from_slice(&json_search.stdout).expect("parse search packet");
    assert_eq!(
        search_packet["schemaId"],
        "agent.semantic-protocols.semantic-document-search-packet"
    );
    assert_eq!(search_packet["languageId"], "org");
    assert_eq!(search_packet["binary"], "asp");
    assert_eq!(search_packet["method"], "search/prime");
    assert_eq!(search_packet["documentMode"], "metadata");
    assert!(
        search_packet["nextActions"]
            .as_array()
            .expect("next actions")
            .iter()
            .any(|action| action["target"] == "selector"
                && action["command"]
                    == "asp org query --selector <path:start-end> --view metadata"),
        "{search_packet:#}"
    );
    assert!(
        search_packet["nextActions"]
            .as_array()
            .expect("next actions")
            .iter()
            .any(|action| action["target"] == "content"
                && action["command"] == "asp org query --term <term> --content"),
        "{search_packet:#}"
    );
    assert!(
        search_packet["documentFacts"]
            .as_array()
            .expect("document facts")
            .iter()
            .any(|fact| fact["kind"] == "property"
                && fact["sourceKind"] == "PropertyDrawer"
                && fact["attributes"]["key"] == "CUSTOM_ID"),
        "{search_packet:#}"
    );
    assert!(
        search_packet["documentFacts"]
            .as_array()
            .expect("document facts")
            .iter()
            .any(|fact| fact["kind"] == "task"
                && fact["sourceKind"] == "SyntaxListItem"
                && fact["attributes"]["checked"] == "true"),
        "{search_packet:#}"
    );
    assert!(
        search_packet["documentFacts"]
            .as_array()
            .expect("document facts")
            .iter()
            .any(|fact| fact["kind"] == "paragraph"
                && fact["sourceKind"] == "Paragraph"
                && fact["attributes"]["text"]
                    .as_str()
                    .is_some_and(|text| text.contains("execution mode"))),
        "{search_packet:#}"
    );

    let json_query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("query")
        .arg("--term")
        .arg("CUSTOM_ID")
        .arg("--json")
        .arg(&root)
        .output()
        .expect("run orgize query json");
    assert!(
        json_query.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&json_query.stderr)
    );
    let query_packet: Value =
        serde_json::from_slice(&json_query.stdout).expect("parse query packet");
    assert_eq!(
        query_packet["schemaId"],
        "agent.semantic-protocols.semantic-document-query-packet"
    );
    assert_eq!(query_packet["languageId"], "org");
    assert_eq!(query_packet["binary"], "asp");
    assert_eq!(query_packet["method"], "query/document");
    assert_eq!(query_packet["documentMode"], "metadata");
    assert_eq!(query_packet["queryKind"], "term");
    assert_eq!(query_packet["querySurface"], "metadata");
    assert!(
        query_packet["documentFacts"]
            .as_array()
            .expect("document facts")
            .iter()
            .any(|item| item["kind"] == "property" && item["attributes"]["key"] == "CUSTOM_ID"),
        "{query_packet:#}"
    );

    let elements_query_packet = serde_json::json!({
        "schemaVersion": 1,
        "predicate": {
            "all": [
                { "kind": "src-block" },
                { "summary": { "key": "language", "equals": "rust" } }
            ]
        },
        "limit": 1
    })
    .to_string();
    let elements_query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("elements-query")
        .arg("--packet")
        .arg(elements_query_packet)
        .arg(&path)
        .output()
        .expect("run orgize elements query packet");
    assert!(
        elements_query.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&elements_query.stderr)
    );
    let elements_records: Value =
        serde_json::from_slice(&elements_query.stdout).expect("parse elements query records");
    let elements_records = elements_records.as_array().expect("elements records");
    assert_eq!(elements_records.len(), 1, "{elements_records:#?}");
    assert_eq!(elements_records[0]["kind"], "src-block");
    assert_eq!(elements_records[0]["summary"]["language"], "rust");
    assert_eq!(elements_records[0]["kindNamespace"], "upstream");

    let json_paragraph_query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("query")
        .arg("--term")
        .arg("embedded")
        .arg("--json")
        .arg(&root)
        .output()
        .expect("run orgize paragraph query json");
    assert!(
        json_paragraph_query.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&json_paragraph_query.stderr)
    );
    let paragraph_query_packet: Value =
        serde_json::from_slice(&json_paragraph_query.stdout).expect("parse paragraph query packet");
    assert!(
        paragraph_query_packet["documentFacts"]
            .as_array()
            .expect("document facts")
            .iter()
            .any(|item| item["kind"] == "paragraph"
                && item["attributes"]["text"]
                    .as_str()
                    .is_some_and(|text| text.contains("embedded inside ASP"))),
        "{paragraph_query_packet:#}"
    );

    let json_content_query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("query")
        .arg("--term")
        .arg("embedded")
        .arg("--content")
        .arg("--json")
        .arg(&root)
        .output()
        .expect("run orgize content query json");
    assert!(
        json_content_query.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&json_content_query.stderr)
    );
    let content_query_packet: Value =
        serde_json::from_slice(&json_content_query.stdout).expect("parse content query packet");
    assert_eq!(content_query_packet["querySurface"], "content");
    assert_eq!(content_query_packet["documentMode"], "content");
    assert!(
        content_query_packet["contentBlocks"]
            .as_array()
            .expect("content blocks")
            .iter()
            .any(|item| item["kind"] == "element"
                && item["content"]
                    .as_str()
                    .is_some_and(|text| text.contains("embedded inside ASP"))),
        "{content_query_packet:#}"
    );

    let dot_root_search = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&root)
        .arg("search")
        .arg("prime")
        .arg("--view")
        .arg("seeds")
        .arg("--json")
        .arg(".")
        .output()
        .expect("run orgize dot-root search json");
    assert!(
        dot_root_search.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&dot_root_search.stderr)
    );
    let dot_root_search_packet: Value =
        serde_json::from_slice(&dot_root_search.stdout).expect("parse dot-root search packet");
    assert_eq!(dot_root_search_packet["projectRoot"], ".");
    assert!(
        dot_root_search_packet["owners"]
            .as_array()
            .expect("owners")
            .iter()
            .any(|owner| owner["path"] == "plan.org"),
        "{dot_root_search_packet:#}"
    );
    assert!(
        dot_root_search_packet["documentFacts"]
            .as_array()
            .expect("document facts")
            .iter()
            .any(|fact| fact["documentPath"] == "plan.org"),
        "{dot_root_search_packet:#}"
    );

    let dot_root_query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&root)
        .arg("query")
        .arg("--selector")
        .arg("plan.org:1-4")
        .arg("--json")
        .arg(".")
        .output()
        .expect("run orgize relative selector json");
    assert!(
        dot_root_query.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&dot_root_query.stderr)
    );
    let dot_root_packet: Value =
        serde_json::from_slice(&dot_root_query.stdout).expect("parse dot-root query packet");
    assert_eq!(dot_root_packet["projectRoot"], ".");
    assert_eq!(dot_root_packet["documentMode"], "metadata");
    assert_eq!(dot_root_packet["queryKind"], "selector");
    assert_eq!(dot_root_packet["querySurface"], "metadata");
    assert!(
        dot_root_packet["documentFacts"]
            .as_array()
            .expect("document facts")
            .iter()
            .any(|fact| fact["documentPath"] == "plan.org"),
        "{dot_root_packet:#}"
    );
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
    assert!(!guide_stdout.contains("owner tests"), "{guide_stdout}");

    let root = test_dir("md-document-search");
    let path = root.join("README.md");
    std::fs::write(
        &path,
        "---\nname: project-doc\ndescription: Document map\n---\n\n# Project\n\n## Overview\n\n### Details\n\nThis paragraph mentions repeat frontier behavior.\n\n- [x] Write tests\n- item\n\n[site](https://example.com)\n![diagram](diagram.png)\n\n---\n\n```rust\nfn main() {}\n```\n",
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
    assert!(
        search_stdout.contains("sourceKind=\"NodeValue::Heading\""),
        "{search_stdout}"
    );

    let prime_search = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("md")
        .arg("search")
        .arg("prime")
        .arg("--view")
        .arg("seeds")
        .arg(&root)
        .output()
        .expect("run orgize md prime search");
    assert!(
        prime_search.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&prime_search.stderr)
    );
    let prime_stdout = String::from_utf8(prime_search.stdout).expect("utf8 prime search");
    assert!(prime_stdout.contains("paragraph="), "{prime_stdout}");
    assert!(prime_stdout.contains("|paragraph"), "{prime_stdout}");
    assert!(prime_stdout.contains("|task"), "{prime_stdout}");
    assert!(prime_stdout.contains("|listItem"), "{prime_stdout}");
    assert!(prime_stdout.contains("|image"), "{prime_stdout}");
    assert!(prime_stdout.contains("|thematicBreak"), "{prime_stdout}");

    let toc = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("md")
        .arg("search")
        .arg("toc")
        .arg(&root)
        .output()
        .expect("run orgize md toc search");
    assert!(
        toc.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&toc.stderr)
    );
    let toc_stdout = String::from_utf8(toc.stdout).expect("utf8 md toc");
    assert!(toc_stdout.contains("[search-toc] lang=md"), "{toc_stdout}");
    assert!(toc_stdout.contains("heading=3"), "{toc_stdout}");
    assert!(toc_stdout.contains("maxLevel=3"), "{toc_stdout}");
    assert!(
        toc_stdout.contains("level=1 title=\"Project\""),
        "{toc_stdout}"
    );
    assert!(!toc_stdout.contains("project-doc"), "{toc_stdout}");
    assert!(
        toc_stdout.contains("level=2 title=\"Overview\""),
        "{toc_stdout}"
    );
    assert!(
        toc_stdout.contains("level=3 title=\"Details\""),
        "{toc_stdout}"
    );
    assert!(
        toc_stdout.contains("next=\"asp md query --selector"),
        "{toc_stdout}"
    );

    let selector = format!("{}:6-6", path.display());
    let query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("md")
        .arg("query")
        .arg("--from-hook")
        .arg("direct-source-read")
        .arg("--selector")
        .arg(selector)
        .output()
        .expect("run orgize md query");
    assert!(query.status.success());
    let query_stdout = String::from_utf8(query.stdout).expect("utf8 query");
    assert_eq!(query_stdout, "# Project\n");

    let selector = format!("{}:6-6", path.display());
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
        selector_stdout
            .contains("direct-read=\"asp md query --from-hook direct-source-read --selector"),
        "{selector_stdout}"
    );
    assert!(selector_stdout.contains("|heading"), "{selector_stdout}");

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

    let paragraph_query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("md")
        .arg("query")
        .arg("--kind")
        .arg("paragraph")
        .arg("--term")
        .arg("repeat frontier")
        .arg("--view")
        .arg("metadata")
        .arg(&root)
        .output()
        .expect("run orgize md paragraph query");
    assert!(paragraph_query.status.success());
    let paragraph_stdout = String::from_utf8(paragraph_query.stdout).expect("utf8 paragraph query");
    assert!(
        paragraph_stdout.contains("[query] lang=md"),
        "{paragraph_stdout}"
    );
    assert!(
        paragraph_stdout.contains("|paragraph"),
        "{paragraph_stdout}"
    );
    assert!(
        paragraph_stdout.contains("repeat frontier behavior"),
        "{paragraph_stdout}"
    );

    let paragraph_content = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("md")
        .arg("query")
        .arg("--kind")
        .arg("paragraph")
        .arg("--term")
        .arg("repeat frontier")
        .arg("--content")
        .arg(&root)
        .output()
        .expect("run orgize md paragraph content query");
    assert!(paragraph_content.status.success());
    let paragraph_content_stdout =
        String::from_utf8(paragraph_content.stdout).expect("utf8 md paragraph content query");
    assert_eq!(
        paragraph_content_stdout.trim(),
        "This paragraph mentions repeat frontier behavior.",
        "{paragraph_content_stdout}"
    );

    let json_search = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("md")
        .arg("search")
        .arg("prime")
        .arg("--view")
        .arg("seeds")
        .arg("--json")
        .arg(&root)
        .output()
        .expect("run orgize md search json");
    assert!(
        json_search.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&json_search.stderr)
    );
    let search_packet: Value =
        serde_json::from_slice(&json_search.stdout).expect("parse search packet");
    assert_eq!(
        search_packet["schemaId"],
        "agent.semantic-protocols.semantic-document-search-packet"
    );
    assert_eq!(search_packet["languageId"], "md");
    assert_eq!(search_packet["binary"], "asp");
    assert_eq!(search_packet["method"], "search/prime");
    assert_eq!(search_packet["documentMode"], "metadata");
    assert!(
        search_packet["nextActions"]
            .as_array()
            .expect("next actions")
            .iter()
            .any(|action| action["target"] == "selector"
                && action["command"] == "asp md query --selector <path:start-end> --view metadata"),
        "{search_packet:#}"
    );
    assert!(
        search_packet["nextActions"]
            .as_array()
            .expect("next actions")
            .iter()
            .any(|action| action["target"] == "content"
                && action["command"] == "asp md query --term <term> --content"),
        "{search_packet:#}"
    );
    assert!(
        search_packet["documentFacts"]
            .as_array()
            .expect("document facts")
            .iter()
            .any(|fact| fact["kind"] == "heading"
                && fact["sourceKind"] == "NodeValue::Heading"
                && fact["attributes"]["title"] == "Project"),
        "{search_packet:#}"
    );
    assert!(
        search_packet["documentFacts"]
            .as_array()
            .expect("document facts")
            .iter()
            .any(|fact| fact["kind"] == "paragraph"
                && fact["sourceKind"] == "NodeValue::Paragraph"
                && fact["attributes"]["text"]
                    .as_str()
                    .unwrap_or_default()
                    .contains("repeat frontier behavior")),
        "{search_packet:#}"
    );
    assert!(
        search_packet["documentFacts"]
            .as_array()
            .expect("document facts")
            .iter()
            .any(|fact| fact["kind"] == "task" && fact["sourceKind"] == "NodeValue::TaskItem"),
        "{search_packet:#}"
    );

    let json_query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("md")
        .arg("query")
        .arg("--term")
        .arg("Project")
        .arg("--json")
        .arg(&root)
        .output()
        .expect("run orgize md query json");
    assert!(
        json_query.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&json_query.stderr)
    );
    let query_packet: Value =
        serde_json::from_slice(&json_query.stdout).expect("parse query packet");
    assert_eq!(
        query_packet["schemaId"],
        "agent.semantic-protocols.semantic-document-query-packet"
    );
    assert_eq!(query_packet["languageId"], "md");
    assert_eq!(query_packet["binary"], "asp");
    assert_eq!(query_packet["method"], "query/document");
    assert_eq!(query_packet["documentMode"], "metadata");
    assert_eq!(query_packet["queryKind"], "term");
    assert_eq!(query_packet["querySurface"], "metadata");
    assert!(
        query_packet["documentFacts"]
            .as_array()
            .expect("document facts")
            .iter()
            .any(|item| item["kind"] == "heading" && item["attributes"]["title"] == "Project"),
        "{query_packet:#}"
    );

    let json_content_query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("md")
        .arg("query")
        .arg("--term")
        .arg("repeat frontier")
        .arg("--content")
        .arg("--json")
        .arg(&root)
        .output()
        .expect("run orgize md content query json");
    assert!(
        json_content_query.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&json_content_query.stderr)
    );
    let content_query_packet: Value =
        serde_json::from_slice(&json_content_query.stdout).expect("parse md content query packet");
    assert_eq!(content_query_packet["querySurface"], "content");
    assert_eq!(content_query_packet["documentMode"], "content");
    assert!(
        content_query_packet["contentBlocks"]
            .as_array()
            .expect("content blocks")
            .iter()
            .any(|item| item["kind"] == "element"
                && item["content"]
                    .as_str()
                    .is_some_and(|text| text.contains("repeat frontier behavior"))),
        "{content_query_packet:#}"
    );
}

fn test_dir(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("orgize-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create test dir");
    root
}
