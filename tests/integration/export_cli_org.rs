use std::path::Path;

use serde_json::Value;

use crate::export_cli::export_cli_common::{
    assert_document_query_evidence, assert_document_selector_query_evidence, test_dir,
};

#[test]
fn org_document_query_commands_run() {
    let guide = crate::library_cli::orgize_cli_command()
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
            "|recipe paragraph-content=orgize org query --kind paragraph --term <term> --workspace . --content"
        ),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains(
            "|cmd elements-query=orgize org elements-query --packet <json-query-packet> <org-file>"
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
            "|cmd capture=orgize org capture --org-contract-registry <contract.org> --contract agent.task.v1 --title <TITLE> --target-file <ORG_FILE>"
        ),
        "{guide_stdout}"
    );
    assert!(!guide_stdout.contains("capture init"), "{guide_stdout}");
    assert!(
        guide_stdout.contains(
            "|recipe capture-task=orgize org capture --org-contract-registry <contract.org> --contract agent.task.v1 --title <TITLE> --target-file <ORG_FILE>"
        ),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains(
            "|recipe sdd-kind-properties=orgize org query --kind property --field key=SDD_KIND --workspace . --view metadata"
        ),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains(
            "|recipe org-id-properties=orgize org query --kind property --field key=ID --field value=<ID> --workspace . --view metadata"
        ),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains(
            "|recipe tagged-tasks=orgize org query --kind task --term <TEXT> --field tag=<TAG> --workspace . --view metadata"
        ),
        "{guide_stdout}"
    );
    assert!(
        guide_stdout.contains(
            "|recipe done-tasks=orgize org query --kind task --field todo=DONE --workspace . --view metadata"
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

    let capture_help = crate::library_cli::orgize_cli_command()
        .arg("org")
        .arg("capture")
        .arg("--help")
        .output()
        .expect("run orgize org capture help");
    assert!(capture_help.status.success());
    assert!(
        String::from_utf8_lossy(&capture_help.stderr).contains("orgize org capture"),
        "{}",
        String::from_utf8_lossy(&capture_help.stderr)
    );

    let root = test_dir("org-document-query");
    let path = root.join("plan.org");
    std::fs::write(
        &path,
        "* TODO [#A] Task :work:sdd:\nSCHEDULED: <2026-06-06 Sat>\n:PROPERTIES:\n:CUSTOM_ID: task-1\n:SDD_KIND: capability\n:SDD_STATUS: draft\n:END:\n\nProvider activation carries execution mode.\nDocument providers stay embedded inside ASP.\n\n** Repository Map\n*** Docs\n- [X] ship element map\nBefore [[https://example.com][site]] after\n[[file:diagram.png]]\n\n#+begin_src rust\nfn main() {\n  println!(  \"x\");\n}\n#+end_src\n",
    )
    .expect("write org fixture");

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
    let query = crate::library_cli::orgize_cli_command()
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

    let link_query = crate::library_cli::orgize_cli_command()
        .arg("org")
        .arg("query")
        .arg("--kind")
        .arg("link")
        .arg("--json")
        .arg(&root)
        .output()
        .expect("run orgize org link query");
    assert!(
        link_query.status.success(),
        "org facade link query failed: {}",
        String::from_utf8_lossy(&link_query.stderr)
    );
    let link_packet: Value =
        serde_json::from_slice(&link_query.stdout).expect("parse org facade link packet");
    let link_selector = link_packet["documentFacts"]
        .as_array()
        .expect("document facts")
        .iter()
        .find(|fact| fact["attributes"]["target"] == "https://example.com")
        .and_then(|fact| fact["structuralSelector"].as_str())
        .expect("site link selector");
    let verbatim_link = crate::library_cli::orgize_cli_command()
        .arg("org")
        .arg("query")
        .arg("--selector")
        .arg(link_selector)
        .arg("--verbatim")
        .current_dir(&root)
        .output()
        .expect("run orgize org verbatim link query");
    assert!(verbatim_link.status.success());
    assert_eq!(
        String::from_utf8(verbatim_link.stdout).expect("utf8 verbatim link"),
        "[[https://example.com][site]]"
    );

    let selector_frontier = crate::library_cli::orgize_cli_command()
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
        selector_stdout.contains("content-query=\"orgize org query --selector"),
        "{selector_stdout}"
    );
    assert!(selector_stdout.contains("|property"), "{selector_stdout}");
    assert!(
        selector_stdout.contains("key=\"CUSTOM_ID\""),
        "{selector_stdout}"
    );

    let term_query = crate::library_cli::orgize_cli_command()
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

    let sdd_kind_query = crate::library_cli::orgize_cli_command()
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

    let sdd_status_query = crate::library_cli::orgize_cli_command()
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

    let paragraph_query = crate::library_cli::orgize_cli_command()
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

    let paragraph_content = crate::library_cli::orgize_cli_command()
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

    let source_block_content = crate::library_cli::orgize_cli_command()
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
        "fn main() {\n  println!(  \"x\");\n}",
        "{source_block_content_stdout}"
    );

    let missing_content = crate::library_cli::orgize_cli_command()
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

    let broad_content = crate::library_cli::orgize_cli_command()
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
    let verbatim_paragraph = crate::library_cli::orgize_cli_command()
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
    assert_eq!(query_packet["providerId"], "asp-org");
    assert_eq!(query_packet["binary"], "orgize");
    assert_eq!(
        query_packet["namespace"],
        "agent.semantic-protocols.languages.org.asp-org"
    );
    assert_eq!(query_packet["method"], "query/document");
    assert_eq!(query_packet["documentMode"], "metadata");
    assert_eq!(query_packet["queryKind"], "term");
    assert_eq!(query_packet["querySurface"], "metadata");
    assert_document_query_evidence(&query_packet);
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
    let elements_query = crate::library_cli::orgize_cli_command()
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
    assert_document_query_evidence(&content_query_packet);
    for block in content_query_packet["contentBlocks"]
        .as_array()
        .expect("content blocks")
    {
        let selector = block["structuralSelector"]
            .as_str()
            .expect("content structural selector");
        let source_path = selector
            .strip_prefix("org://")
            .and_then(|selector| selector.split_once('#'))
            .map(|(path, _)| path)
            .expect("replayable Org structural selector");
        assert!(
            Path::new(source_path).is_file(),
            "content selector source must be replayable: {selector}"
        );
    }
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

    let dot_root_inventory = orgize_command()
        .current_dir(&root)
        .arg("query")
        .arg("--json")
        .arg(".")
        .output()
        .expect("run orgize dot-root query inventory json");
    assert!(
        dot_root_inventory.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&dot_root_inventory.stderr)
    );
    let dot_root_inventory_packet: Value = serde_json::from_slice(&dot_root_inventory.stdout)
        .expect("parse dot-root query inventory packet");
    assert_eq!(dot_root_inventory_packet["projectRoot"], ".");
    assert!(
        dot_root_inventory_packet["documentFacts"]
            .as_array()
            .expect("document facts")
            .iter()
            .any(|fact| fact["documentPath"] == "plan.org"),
        "{dot_root_inventory_packet:#}"
    );

    let dot_root_query = orgize_command()
        .current_dir(&root)
        .arg("query")
        .arg("--selector")
        .arg(
            dot_root_inventory_packet["documentFacts"]
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
    assert_document_selector_query_evidence(&dot_root_packet, "plan.org");
    assert!(
        dot_root_packet["documentFacts"]
            .as_array()
            .expect("document facts")
            .iter()
            .any(|fact| fact["documentPath"] == "plan.org"),
        "{dot_root_packet:#}"
    );

    let absolute_heading_selector = dot_root_inventory_packet["documentFacts"]
        .as_array()
        .expect("document facts")
        .iter()
        .find(|fact| fact["kind"] == "heading")
        .and_then(|fact| fact["structuralSelector"].as_str())
        .expect("dot-root heading structural selector");
    let relative_heading_selector = format!(
        "org://plan.org#{}",
        absolute_heading_selector
            .split_once('#')
            .map(|(_, fragment)| fragment)
            .expect("selector fragment")
    );
    let relative_selector_query = orgize_command()
        .current_dir(&root)
        .args([
            "query",
            "--selector",
            &relative_heading_selector,
            "--json",
            ".",
        ])
        .output()
        .expect("run relative structural selector query");
    assert!(
        relative_selector_query.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&relative_selector_query.stderr)
    );
    let relative_selector_packet: Value = serde_json::from_slice(&relative_selector_query.stdout)
        .expect("parse relative selector packet");
    assert_eq!(relative_selector_packet["projectRoot"], ".");

    let nested_dir = root.join("sub");
    std::fs::create_dir_all(&nested_dir).expect("create nested query fixture");
    std::fs::write(nested_dir.join("plan.org"), "* Nested plan\n")
        .expect("write nested query fixture");
    let nested_inventory = orgize_command()
        .current_dir(&root)
        .args(["query", "--json", "sub/plan.org"])
        .output()
        .expect("run nested relative inventory");
    assert!(nested_inventory.status.success());
    let nested_inventory: Value =
        serde_json::from_slice(&nested_inventory.stdout).expect("parse nested inventory");
    let nested_fragment = nested_inventory["documentFacts"]
        .as_array()
        .expect("nested document facts")
        .iter()
        .find(|fact| fact["kind"] == "heading")
        .and_then(|fact| fact["structuralSelector"].as_str())
        .and_then(|selector| selector.split_once('#'))
        .map(|(_, fragment)| fragment)
        .expect("nested heading fragment");
    let nested_selector = format!("org://sub/plan.org#{nested_fragment}");
    let nested_query = orgize_command()
        .current_dir(&root)
        .args(["query", "--selector", &nested_selector, "--json"])
        .output()
        .expect("run nested relative selector query");
    assert!(
        nested_query.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&nested_query.stderr)
    );
    let nested_packet: Value =
        serde_json::from_slice(&nested_query.stdout).expect("parse nested selector packet");
    assert_eq!(nested_packet["projectRoot"], ".");
    assert_document_selector_query_evidence(&nested_packet, "sub/plan.org");
    assert!(
        nested_packet["documentFacts"]
            .as_array()
            .expect("nested selector facts")
            .iter()
            .all(|fact| fact["documentPath"] == "sub/plan.org"),
        "{nested_packet:#}"
    );

    let relative_parent_dir = root.join("selector-client");
    std::fs::create_dir_all(&relative_parent_dir).expect("create parent-relative client");
    let parent_relative_query = orgize_command()
        .current_dir(&relative_parent_dir)
        .args([
            "query",
            "--selector",
            &format!("org://../plan.org#{nested_fragment}"),
            "--json",
        ])
        .output()
        .expect("run parent-relative selector query");
    assert!(
        parent_relative_query.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&parent_relative_query.stderr)
    );
    let parent_relative_packet: Value = serde_json::from_slice(&parent_relative_query.stdout)
        .expect("parse parent-relative selector packet");
    assert_eq!(parent_relative_packet["projectRoot"], "..");
    assert_document_selector_query_evidence(&parent_relative_packet, "plan.org");
    assert!(
        parent_relative_packet["documentFacts"]
            .as_array()
            .expect("parent-relative selector facts")
            .iter()
            .all(|fact| fact["documentPath"] == "plan.org"),
        "{parent_relative_packet:#}"
    );

    for kind in ["heading", "task"] {
        let selector = dot_root_inventory_packet["documentFacts"]
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
        assert_eq!(content, "* TODO [#A] Task :work:sdd:\n");
    }
}
fn orgize_command() -> crate::library_cli::OrgizeLibraryCliCommand {
    crate::library_cli::orgize_cli_command()
}

#[test]
fn org_document_legacy_search_facade_is_rejected() {
    let usage = orgize_command().output().expect("render top-level usage");
    let usage = String::from_utf8_lossy(&usage.stderr);
    assert!(usage.contains("|org|"), "{usage}");
    assert!(!usage.contains("|search|"), "{usage}");

    for view in ["prime", "toc", "owner", "fzf", "memory"] {
        let output = orgize_command().args(["search", view]).output().unwrap();
        assert!(
            !output.status.success(),
            "legacy search view {view} was admitted"
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("unknown command `search`"));
    }
}

#[test]
fn orgize_version_reports_build_provenance() {
    let output = orgize_command()
        .args(["version", "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("version receipt");
    assert_eq!(receipt["name"], "orgize");
    assert!(
        receipt["sourceRevision"]
            .as_str()
            .is_some_and(|value| value.len() == 40)
    );
    assert!(receipt["sourceDirty"].is_boolean());
}
