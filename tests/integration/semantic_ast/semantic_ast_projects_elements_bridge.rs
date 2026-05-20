use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{OrgElementsHostExecutionOptions, PythonDirectiveKind},
    Org,
};
use serde_json::Value;

#[test]
fn semantic_ast_projects_header_tags_and_org_elements_host_execution() {
    let doc = Org::parse(
        r#"#+TAGS: EMACS (e) COURSE (c) ENGLISH SECURITY (s) BOOK (b) EXERCISE (ex) READ(r) MATH (m) NSM LEARN
#+PYTHON: print("explicit host execution only")

* TODO Learn parser bindings :EMACS:READ:
:PROPERTIES:
:Effort: 1:00
:END:
See [[https://example.test][example]] at <2026-05-19 Tue>.
- [X] done item with src_python{print("ok")}
#+begin_src python :results output :var topic="org-elements"
print(topic)
#+end_src
"#,
    )
    .document();

    assert_clean_projection(&doc);
    assert_eq!(doc.tag_definitions[0].name, "EMACS");
    assert_eq!(doc.tag_definitions[0].shortcut.as_deref(), Some("e"));
    assert_eq!(doc.tag_definitions[5].name, "EXERCISE");
    assert_eq!(doc.tag_definitions[5].shortcut.as_deref(), Some("ex"));
    assert_eq!(doc.tag_definitions[6].name, "READ");
    assert_eq!(doc.tag_definitions[6].shortcut.as_deref(), Some("r"));

    let plan = doc.org_elements_execution_plan();
    assert_eq!(plan.python_directives.len(), 1);
    assert_eq!(plan.python_directives[0].kind, PythonDirectiveKind::Inline);
    assert_eq!(
        plan.python_directives[0].value,
        r#"print("explicit host execution only")"#
    );

    let payload: Value =
        serde_json::from_str(&doc.org_elements_json()).expect("payload JSON should parse");
    assert_eq!(payload["schemaVersion"], 1);
    assert_eq!(payload["tagDefinitions"][0]["name"], "EMACS");
    assert_eq!(payload["tagDefinitions"][0]["shortcut"], "e");
    assert_eq!(payload["sections"][0]["title"], "Learn parser bindings");
    assert_eq!(payload["sections"][0]["todo"], "TODO");
    assert_eq!(payload["sections"][0]["tags"][0], "EMACS");
    assert_eq!(
        payload["sections"][0]["properties"][0]["duration"]["raw"],
        "1:00"
    );
    assert!(payload["sections"][0]["titleObjects"]
        .as_array()
        .expect("title objects")
        .iter()
        .any(|object| object["kind"] == "plain-text"));
    let section_elements = payload["sections"][0]["elements"]
        .as_array()
        .expect("section elements");
    let paragraph = section_elements
        .iter()
        .find(|element| element["kind"] == "paragraph")
        .expect("paragraph element");
    assert!(paragraph["objects"]
        .as_array()
        .expect("paragraph objects")
        .iter()
        .any(|object| object["kind"] == "link" && object["path"] == "https://example.test"));
    assert!(paragraph["objects"]
        .as_array()
        .expect("paragraph objects")
        .iter()
        .any(|object| object["kind"] == "timestamp" && object["raw"] == "<2026-05-19 Tue>"));
    let list = section_elements
        .iter()
        .find(|element| element["kind"] == "plain-list")
        .expect("plain-list element");
    assert_eq!(list["items"][0]["checkbox"], "on");
    assert!(section_elements
        .iter()
        .any(|element| element["kind"] == "src-block" && element["language"] == "python"));
    let index = payload["index"].as_array().expect("flat node index");
    assert!(index.iter().any(|node| node["category"] == "section"
        && node["kind"] == "headline"
        && node["summary"]["title"] == "Learn parser bindings"));
    assert!(index.iter().any(|node| node["category"] == "object"
        && node["kind"] == "link"
        && node["summary"]["path"] == "https://example.test"));
    assert!(index.iter().any(|node| node["category"] == "object"
        && node["kind"] == "timestamp"
        && node["context"] == "paragraph"));
    assert!(index.iter().any(|node| node["category"] == "element"
        && node["kind"] == "src-block"
        && node["summary"]["language"] == "python"));
    assert!(payload["sourceBlocks"]
        .as_array()
        .expect("source blocks")
        .iter()
        .any(|block| block["kind"] == "inlineSource" && block["language"] == "python"));
    let python_block = payload["sourceBlocks"]
        .as_array()
        .expect("source blocks")
        .iter()
        .find(|block| block["kind"] == "block" && block["language"] == "python")
        .expect("python source block");
    assert_eq!(
        python_block["normalizedHeaderArgs"]
            .as_array()
            .expect("normalized args")
            .iter()
            .find(|arg| arg["kind"] == "var")
            .and_then(|arg| arg["variable"]["name"].as_str()),
        Some("topic")
    );

    let output = doc
        .execute_org_elements(&OrgElementsHostExecutionOptions::new("python3").args([
            "-I",
            "-c",
            r#"
import json
import sys

doc = json.load(sys.stdin)
result = {
    "firstTitle": doc["sections"][0]["title"],
    "tagShortcuts": {item["name"]: item.get("shortcut") for item in doc["tagDefinitions"]},
    "pythonBlocks": [
        block["language"] for block in doc["sourceBlocks"] if block["language"] == "python"
    ],
}
print(json.dumps(result, sort_keys=True))
"#,
        ]))
        .expect("host should execute");
    assert!(output.status.success, "stderr: {}", output.stderr);

    let result: Value = serde_json::from_str(output.stdout.trim()).expect("Python JSON output");
    assert_eq!(result["firstTitle"], "Learn parser bindings");
    assert_eq!(result["tagShortcuts"]["EXERCISE"], "ex");
    assert_eq!(result["pythonBlocks"][0], "python");
}
