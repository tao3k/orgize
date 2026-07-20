use std::{fs, process::Command};

use serde_json::Value;

use crate::export_cli::export_cli_common::test_dir;

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
    assert!(
        prime_stdout.contains("O=owner:path(README.md)!owner"),
        "{prime_stdout}"
    );
    assert!(prime_stdout.contains("G>{O:selects}"), "{prime_stdout}");
    assert!(prime_stdout.contains("frontier=O.owner"), "{prime_stdout}");
    assert!(prime_stdout.contains("paragraph="), "{prime_stdout}");
    assert!(prime_stdout.contains("|paragraph"), "{prime_stdout}");
    assert!(prime_stdout.contains("|checklistItem"), "{prime_stdout}");
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

    let selector_query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("md")
        .arg("query")
        .arg("--term")
        .arg("Project")
        .arg("--json")
        .arg(&root)
        .output()
        .expect("run orgize md selector query json");
    assert!(selector_query.status.success());
    let selector_packet: Value =
        serde_json::from_slice(&selector_query.stdout).expect("parse md selector packet");
    let selector = selector_packet["documentFacts"]
        .as_array()
        .expect("document facts")
        .iter()
        .find(|fact| fact["kind"] == "heading" && fact["attributes"]["title"] == "Project")
        .and_then(|fact| fact["structuralSelector"].as_str())
        .expect("Project heading selector")
        .to_string();
    let query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("md")
        .arg("query")
        .arg("--selector")
        .arg(&selector)
        .arg("--verbatim")
        .output()
        .expect("run orgize md query");
    assert!(query.status.success());
    let query_stdout = String::from_utf8(query.stdout).expect("utf8 query");
    assert_eq!(query_stdout, "# Project\n");

    let selector_frontier = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("md")
        .arg("query")
        .arg("--selector")
        .arg(&selector)
        .output()
        .expect("run orgize md selector frontier query");
    assert!(selector_frontier.status.success());
    let selector_stdout = String::from_utf8(selector_frontier.stdout).expect("utf8 selector query");
    assert!(
        selector_stdout.contains("[query-selector] lang=md"),
        "{selector_stdout}"
    );
    assert!(
        selector_stdout.contains("content-query=\"asp md query --selector"),
        "{selector_stdout}"
    );
    assert!(
        selector_stdout.contains("--content --workspace .\""),
        "{selector_stdout}"
    );
    assert!(selector_stdout.contains("|heading"), "{selector_stdout}");

    let selector_content = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("md")
        .arg("query")
        .arg("--selector")
        .arg(&selector)
        .arg("--content")
        .output()
        .expect("run orgize md selector content query");
    assert!(selector_content.status.success());
    let selector_content_stdout =
        String::from_utf8(selector_content.stdout).expect("utf8 selector content query");
    assert!(
        selector_content_stdout.contains("# Project\n"),
        "{selector_content_stdout}"
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
                && action["command"]
                    == "asp md query --selector <structural-selector> --view metadata"),
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
            .any(|fact| fact["kind"] == "checklistItem"
                && fact["sourceKind"] == "NodeValue::TaskItem"),
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
