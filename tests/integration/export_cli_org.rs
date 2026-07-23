use std::{fs, process::Command};

use serde_json::Value;

use crate::export_cli::export_cli_common::test_dir;

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
        guide_stdout.contains(
            "|field-map task source=Headline fields=level,title,todo,todoType,priority,tag"
        ),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains("|field-map block fields=kind=source|export,lang,backend"),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains(
            "|recipe paragraph-content=asp org query --kind paragraph --term <term> --workspace . --content"
        ),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains("|cmd search-toc=asp org search toc --workspace ."),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains(
            "|cmd elements-query=asp org elements-query --packet <json-query-packet> <org-file>"
        ),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains(
            "|surface capture purpose=state-init-and-non-mutating-org-entry-plan output=compact-plan content=false"
        ),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains(
            "|cmd capture=asp org capture --contract agent.task.v1 --title <TITLE> --target-file <ORG_FILE>"
        ),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains(
            "|recipe capture-task=asp org capture --contract agent.task.v1 --title <TITLE> --target-file <ORG_FILE>"
        ),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains(
            "|recipe sdd-kind-properties=asp org query --kind property --field key=SDD_KIND --workspace . --view metadata"
        ),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains(
            "|recipe org-id-properties=asp org query --kind property --field key=ID --field value=<ID> --workspace . --view metadata"
        ),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains(
            "|recipe tagged-tasks=asp org query --kind task --term <TEXT> --field tag=<TAG> --workspace . --view metadata"
        ),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains(
            "|recipe done-tasks=asp org query --kind task --field todo=DONE --workspace . --view metadata"
        ),
        "{guide_stdout}"
    );
    for domain_recipe in [
        "agent-plan-template",
        "agent-plan-state",
        "agent-plan-session",
        "agent-plan-branch",
        "sdd-property",
        "wendao-task",
        "wendao-orgid",
        "plan-record",
    ] {
        assert!(
            !guide_stdout.contains(domain_recipe),
            "legacy recipe `{domain_recipe}` leaked into guide:\n{guide_stdout}"
        );
    }
    assert!(
        !guide_stdout.contains("orgize task-probe"),
        "{guide_stdout}"
    );

    let root = test_dir("org-document-search");
    let path = root.join("plan.org");
    std::fs::write(
        &path,
        "* TODO [#A] Task :work:sdd:\nSCHEDULED: <2026-06-06 Sat>\n:PROPERTIES:\n:CUSTOM_ID: task-1\n:SDD_KIND: capability\n:SDD_STATUS: draft\n:END:\n\nProvider activation carries execution mode.\nDocument providers stay embedded inside ASP.\n\n** Repository Map\n*** Docs\n- [X] ship element map\n[[https://example.com][site]]\n[[file:diagram.png]]\n\n#+begin_src rust\nfn main() {\n  println!(  \"x\");\n}\n#+end_src\n",
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
    assert!(
        search_stdout.contains("O=owner:path(plan.org)!owner"),
        "{search_stdout}"
    );
    assert!(search_stdout.contains("G>{O:selects}"), "{search_stdout}");
    assert!(
        search_stdout.contains("frontier=O.owner"),
        "{search_stdout}"
    );
    assert!(search_stdout.contains("|heading"), "{search_stdout}");
    assert!(
        search_stdout.contains("key=\"CUSTOM_ID\""),
        "{search_stdout}"
    );
    assert!(
        search_stdout.contains("key=\"SDD_KIND\" value=\"capability\""),
        "{search_stdout}"
    );
    assert!(
        search_stdout.contains("key=\"SDD_STATUS\" value=\"draft\""),
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
    assert!(search_stdout.contains("|checklistItem"), "{search_stdout}");
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

    let selector_query = orgize_command()
        .arg("query")
        .arg("--kind")
        .arg("property")
        .arg("--field")
        .arg("key=CUSTOM_ID")
        .arg("--json")
        .arg(&root)
        .output()
        .expect("run orgize property selector query json");
    assert!(
        selector_query.status.success(),
        "selector query failed: {}",
        String::from_utf8_lossy(&selector_query.stderr)
    );
    let selector_packet: Value =
        serde_json::from_slice(&selector_query.stdout).expect("parse property selector packet");
    let selector = selector_packet["documentFacts"]
        .as_array()
        .expect("document facts")
        .iter()
        .find(|fact| fact["kind"] == "property" && fact["attributes"]["key"] == "CUSTOM_ID")
        .and_then(|fact| fact["structuralSelector"].as_str())
        .expect("CUSTOM_ID structural selector")
        .to_string();
    let query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("query")
        .arg("--selector")
        .arg(&selector)
        .arg("--verbatim")
        .current_dir(&root)
        .output()
        .expect("run orgize query");
    assert!(
        query.status.success(),
        "verbatim selector query failed: {}",
        String::from_utf8_lossy(&query.stderr)
    );
    let query_stdout = String::from_utf8(query.stdout).expect("utf8 query");
    assert!(
        query_stdout.contains(":CUSTOM_ID: task-1"),
        "{query_stdout}"
    );

    let selector_frontier = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("query")
        .arg("--selector")
        .arg(&selector)
        .current_dir(&root)
        .output()
        .expect("run orgize selector frontier query");
    assert!(
        selector_frontier.status.success(),
        "selector frontier failed: {}",
        String::from_utf8_lossy(&selector_frontier.stderr)
    );
    let selector_stdout = String::from_utf8(selector_frontier.stdout).expect("utf8 selector query");
    assert!(
        selector_stdout.contains("[query-selector] lang=org"),
        "{selector_stdout}"
    );
    assert!(
        selector_stdout.contains("content-query=\"asp org query --selector"),
        "{selector_stdout}"
    );
    assert!(selector_stdout.contains("|property"), "{selector_stdout}");
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

    let sdd_kind_query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("query")
        .arg("--kind")
        .arg("property")
        .arg("--field")
        .arg("key=SDD_KIND")
        .arg(&root)
        .output()
        .expect("run orgize SDD property field query");
    assert!(sdd_kind_query.status.success());
    let sdd_kind_stdout =
        String::from_utf8(sdd_kind_query.stdout).expect("utf8 SDD property query");
    assert!(
        sdd_kind_stdout.contains("[query] lang=org"),
        "{sdd_kind_stdout}"
    );
    assert!(
        sdd_kind_stdout.contains("key=\"SDD_KIND\" value=\"capability\""),
        "{sdd_kind_stdout}"
    );
    assert!(!sdd_kind_stdout.contains("SDD_STATUS"), "{sdd_kind_stdout}");

    let sdd_status_query = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("query")
        .arg("--kind")
        .arg("property")
        .arg("--field")
        .arg("key=SDD_STATUS")
        .arg("--field")
        .arg("value=draft")
        .arg(&root)
        .output()
        .expect("run orgize SDD status property query");
    assert!(sdd_status_query.status.success());
    let sdd_status_stdout =
        String::from_utf8(sdd_status_query.stdout).expect("utf8 SDD status query");
    assert!(
        sdd_status_stdout.contains("key=\"SDD_STATUS\" value=\"draft\""),
        "{sdd_status_stdout}"
    );

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

    let source_block_content = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("query")
        .arg("--kind")
        .arg("block")
        .arg("--field")
        .arg("lang=rust")
        .arg("--content")
        .arg(&root)
        .output()
        .expect("run orgize source block content query");
    assert!(source_block_content.status.success());
    let source_block_content_stdout =
        String::from_utf8(source_block_content.stdout).expect("utf8 source block content query");
    assert_eq!(
        source_block_content_stdout.trim(),
        "#+begin_src rust\nfn main() {\n  println!(  \"x\");\n}\n#+end_src",
        "{source_block_content_stdout}"
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

    let paragraph_selector_query = orgize_command()
        .arg("query")
        .arg("--term")
        .arg("embedded")
        .arg("--json")
        .arg(&root)
        .output()
        .expect("run orgize paragraph selector query json");
    assert!(paragraph_selector_query.status.success());
    let paragraph_selector_packet: Value = serde_json::from_slice(&paragraph_selector_query.stdout)
        .expect("parse paragraph selector packet");
    let paragraph_selector = paragraph_selector_packet["documentFacts"]
        .as_array()
        .expect("document facts")
        .iter()
        .find(|fact| fact["kind"] == "paragraph")
        .and_then(|fact| fact["structuralSelector"].as_str())
        .expect("paragraph structural selector");
    let verbatim_paragraph = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .arg("query")
        .arg("--selector")
        .arg(paragraph_selector)
        .arg("--verbatim")
        .current_dir(&root)
        .output()
        .expect("run orgize verbatim paragraph query");
    assert!(
        verbatim_paragraph.status.success(),
        "verbatim paragraph query failed: {}",
        String::from_utf8_lossy(&verbatim_paragraph.stderr)
    );
    let verbatim_paragraph_stdout =
        String::from_utf8(verbatim_paragraph.stdout).expect("utf8 verbatim paragraph stdout");
    assert_eq!(
        verbatim_paragraph_stdout,
        "Provider activation carries execution mode.\nDocument providers stay embedded inside ASP.\n\n"
    );

    let json_search = orgize_command()
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
                    == "asp org query --selector <structural-selector> --view metadata"),
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
                && fact["sourceKind"] == "Headline"
                && fact["attributes"]["todo"] == "TODO"),
        "{search_packet:#}"
    );
    assert!(
        search_packet["documentFacts"]
            .as_array()
            .expect("document facts")
            .iter()
            .any(|fact| fact["kind"] == "checklistItem"
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

    let json_query = orgize_command()
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
    assert_eq!(query_packet["binary"], "orgize");
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

    let json_paragraph_query = orgize_command()
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

    let json_content_query = orgize_command()
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

    let dot_root_search = orgize_command()
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

    let dot_root_query = orgize_command()
        .current_dir(&root)
        .arg("query")
        .arg("--selector")
        .arg(
            dot_root_search_packet["documentFacts"]
                .as_array()
                .expect("document facts")
                .iter()
                .find(|fact| fact["kind"] == "heading")
                .and_then(|fact| fact["structuralSelector"].as_str())
                .expect("dot-root heading structural selector"),
        )
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

    for kind in ["heading", "task"] {
        let selector = dot_root_search_packet["documentFacts"]
            .as_array()
            .expect("document facts")
            .iter()
            .find(|fact| fact["kind"] == kind && fact["attributes"]["title"] == "Task")
            .and_then(|fact| fact["structuralSelector"].as_str())
            .unwrap_or_else(|| panic!("{kind} structural selector"));
        let content = orgize_command()
            .current_dir(&root)
            .arg("query")
            .arg("--selector")
            .arg(selector)
            .arg("--content")
            .arg(".")
            .output()
            .unwrap_or_else(|error| panic!("run {kind} content query: {error}"));
        assert!(
            content.status.success(),
            "{kind} content query failed: {}",
            String::from_utf8_lossy(&content.stderr)
        );
        let content = String::from_utf8(content.stdout).expect("utf8 headline content");
        assert!(content.contains("* TODO [#A] Task :work:sdd:"), "{content}");
        assert!(content.contains(":CUSTOM_ID: task-1"), "{content}");
        assert!(
            content.contains("Provider activation carries execution mode."),
            "{content}"
        );
        assert!(content.contains("** Repository Map"), "{content}");
        assert!(content.contains("- [X] ship element map"), "{content}");
        assert!(content.contains("#+begin_src rust"), "{content}");
    }
}
fn orgize_command() -> Command {
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_orgize"));
    command.env(
        "ASP_PROVIDER_EXECUTION_COMMAND_DIGEST",
        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
    );
    command
}
