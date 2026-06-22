use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{
        ORG_ELEMENTS_SQL_COLUMNS, OrgElementKindNamespace, OrgElementPropertyProvenance,
        OrgElementSelector, OrgElementSelectorParseError, OrgElementsHostExecutionOptions,
        OrgElementsIndexCategory, OrgElementsIndexKind, OrgElementsIndexQuery,
        OrgElementsIndexRecord, OrgElementsIndexSummaryValue, ParsedAnnotation,
        PythonDirectiveKind, org_elements_index_query_from_json_str,
        org_elements_index_query_to_json_value,
    },
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(feature = "datafusion-sql")]
use datafusion::arrow::array::{Int64Array, StringArray};

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
    assert!(
        payload["sections"][0]["titleObjects"]
            .as_array()
            .expect("title objects")
            .iter()
            .any(|object| object["kind"] == "plain-text")
    );
    let section_elements = payload["sections"][0]["elements"]
        .as_array()
        .expect("section elements");
    let paragraph = section_elements
        .iter()
        .find(|element| element["kind"] == "paragraph")
        .expect("paragraph element");
    assert!(
        paragraph["objects"]
            .as_array()
            .expect("paragraph objects")
            .iter()
            .any(|object| object["kind"] == "link" && object["path"] == "https://example.test")
    );
    assert!(
        paragraph["objects"]
            .as_array()
            .expect("paragraph objects")
            .iter()
            .any(|object| object["kind"] == "timestamp" && object["raw"] == "<2026-05-19 Tue>")
    );
    let list = section_elements
        .iter()
        .find(|element| element["kind"] == "plain-list")
        .expect("plain-list element");
    assert_eq!(list["items"][0]["checkbox"], "on");
    assert!(
        section_elements
            .iter()
            .any(|element| element["kind"] == "src-block" && element["language"] == "python")
    );
    let typed_index = doc.org_elements_index();
    assert_eq!(typed_index[0].category, OrgElementsIndexCategory::Document);
    let graph = doc.org_elements_graph();
    assert_eq!(graph.root_id, typed_index[0].id);
    assert_eq!(
        graph.children(graph.root_id).len(),
        typed_index[0].child_ids.len()
    );
    let typed_headline = typed_index
        .iter()
        .find(|node| {
            node.category == OrgElementsIndexCategory::Section && node.kind.as_str() == "headline"
        })
        .expect("typed headline index record");
    assert_eq!(typed_headline.parent_id, Some(graph.root_id));
    assert_eq!(
        typed_headline.properties.get(":raw-value"),
        Some(&OrgElementsIndexSummaryValue::Text(
            "Learn parser bindings".to_string()
        ))
    );
    assert_eq!(
        typed_headline.property_provenance.get(":raw-value"),
        Some(&OrgElementPropertyProvenance::Local)
    );
    assert_eq!(
        typed_headline.property_provenance.get(":begin"),
        Some(&OrgElementPropertyProvenance::Standard)
    );
    assert_eq!(
        typed_headline.property_provenance.get(":EFFORT"),
        Some(&OrgElementPropertyProvenance::Local)
    );
    let plain_text = typed_index
        .iter()
        .find(|node| node.kind.as_str() == "plain-text")
        .expect("plain-text extension record");
    assert_eq!(
        plain_text.kind.namespace(),
        OrgElementKindNamespace::OrgizeExtension
    );
    assert_eq!(plain_text.kind.extension_namespace(), Some("orgize"));
    assert_eq!(
        typed_headline.properties.get(":EFFORT"),
        Some(&OrgElementsIndexSummaryValue::Text("1:00".to_string()))
    );
    let typed_link = typed_index
        .iter()
        .find(|node| {
            node.category == OrgElementsIndexCategory::Object && node.kind.as_str() == "link"
        })
        .expect("typed link index record");
    assert_eq!(typed_link.context, "paragraph");
    assert_eq!(
        typed_link.summary.get("path"),
        Some(&OrgElementsIndexSummaryValue::Text(
            "https://example.test".to_string()
        ))
    );
    assert_eq!(
        graph
            .parent(typed_link.id)
            .map(|record| record.kind.as_str()),
        Some("paragraph")
    );
    let filtered_index = doc.query_org_elements_index(
        &OrgElementsIndexQuery::new()
            .category(OrgElementsIndexCategory::Object)
            .kind("link")
            .context("paragraph")
            .limit(1),
    );
    assert_eq!(filtered_index.len(), 1);
    assert_eq!(filtered_index[0].kind.as_str(), "link");
    let summary_eq_index = doc.query_org_elements_index(
        &OrgElementsIndexQuery::new()
            .kind("link")
            .summary_eq("path", "https://example.test"),
    );
    assert_eq!(summary_eq_index.len(), 1);
    let summary_contains_index = doc.query_org_elements_index(
        &OrgElementsIndexQuery::new()
            .category(OrgElementsIndexCategory::Section)
            .summary_contains("title", "parser"),
    );
    assert_eq!(summary_contains_index.len(), 1);
    assert_eq!(
        summary_contains_index[0].outline_path,
        vec!["Learn parser bindings".to_string()]
    );
    let source_block_value_index = doc.query_org_elements_index(
        &OrgElementsIndexQuery::new()
            .category(OrgElementsIndexCategory::Element)
            .kind("src-block")
            .summary_contains("value", "print(topic)"),
    );
    assert_eq!(source_block_value_index.len(), 1);
    assert_eq!(
        source_block_value_index[0].summary.get("language"),
        Some(&OrgElementsIndexSummaryValue::Text("python".to_string()))
    );
    let property_eq_index = doc.query_org_elements_index(
        &OrgElementsIndexQuery::new()
            .category(OrgElementsIndexCategory::Section)
            .property_eq(":Effort", "1:00"),
    );
    assert_eq!(property_eq_index.len(), 1);
    assert_eq!(
        property_eq_index[0].properties.get(":todo-type"),
        Some(&OrgElementsIndexSummaryValue::Text("todo".to_string()))
    );
    assert!(
        doc.query_org_elements_index(&OrgElementsIndexQuery::new().kind("link").limit(0))
            .is_empty()
    );
    let index_only: Value =
        serde_json::from_str(&doc.org_elements_index_json()).expect("index JSON should parse");
    assert_eq!(
        index_only.as_array().expect("index array").len(),
        typed_index.len()
    );
    let filtered_json: Value = serde_json::from_str(
        &doc.org_elements_index_query_json(&OrgElementsIndexQuery::new().kind("timestamp")),
    )
    .expect("filtered index JSON should parse");
    assert!(
        filtered_json
            .as_array()
            .expect("filtered index")
            .iter()
            .all(|node| node["kind"] == "timestamp")
    );
    let sql_rows = doc.org_elements_index_query_sql_rows(
        &OrgElementsIndexQuery::new()
            .category(OrgElementsIndexCategory::Element)
            .kind("src-block"),
    );
    assert!(
        ORG_ELEMENTS_SQL_COLUMNS
            .iter()
            .any(|column| column.name == "affiliated_name")
    );
    assert_eq!(sql_rows.len(), 1);
    assert_eq!(sql_rows[0].kind, OrgElementsIndexKind::new("src-block"));
    assert_eq!(sql_rows[0].language.as_deref(), Some("python"));
    assert!(sql_rows[0].summary_json.contains(r#""language":"python""#));
    assert!(sql_rows[0].summary_json.contains("print(topic)"));
    assert!(sql_rows[0].source_start_line > 0);
    let sql_rows_json: Value = serde_json::from_str(
        &doc.org_elements_index_query_sql_rows_json(
            &OrgElementsIndexQuery::new()
                .category(OrgElementsIndexCategory::Object)
                .kind("link"),
        ),
    )
    .expect("SQL rows JSON should parse");
    assert!(
        sql_rows_json
            .as_array()
            .expect("SQL row array")
            .iter()
            .any(|row| row["kind"] == "link"
                && row["summaryJson"].as_str().unwrap().contains("example"))
    );
    let index = payload["index"].as_array().expect("flat node index");
    assert!(index.iter().any(|node| node["category"] == "section"
        && node["kind"] == "headline"
        && node["summary"]["title"] == "Learn parser bindings"));
    let headline_node = index
        .iter()
        .find(|node| {
            node["category"] == "section"
                && node["kind"] == "headline"
                && node["summary"]["title"] == "Learn parser bindings"
        })
        .expect("headline index JSON record");
    assert!(headline_node["parentId"].as_u64().is_some());
    assert_eq!(
        headline_node["properties"][":raw-value"],
        "Learn parser bindings"
    );
    assert_eq!(headline_node["properties"][":EFFORT"], "1:00");
    let plain_text_node = index
        .iter()
        .find(|node| node["kind"] == "plain-text")
        .expect("plain-text index JSON record");
    insta::assert_snapshot!(
        "org_element_namespace_and_property_provenance",
        serde_json::to_string_pretty(&serde_json::json!({
            "headline": {
                "kind": headline_node["kind"].clone(),
                "kindNamespace": headline_node["kindNamespace"].clone(),
                "extensionNamespace": headline_node["extensionNamespace"].clone(),
                "propertyProvenance": {
                    ":raw-value": headline_node["propertyProvenance"][":raw-value"].clone(),
                    ":begin": headline_node["propertyProvenance"][":begin"].clone(),
                    ":EFFORT": headline_node["propertyProvenance"][":EFFORT"].clone(),
                },
            },
            "plainText": {
                "kind": plain_text_node["kind"].clone(),
                "kindNamespace": plain_text_node["kindNamespace"].clone(),
                "extensionNamespace": plain_text_node["extensionNamespace"].clone(),
            },
        }))
        .unwrap()
    );
    assert!(index.iter().any(|node| node["category"] == "object"
        && node["kind"] == "link"
        && node["summary"]["path"] == "https://example.test"));
    assert!(index.iter().any(|node| node["category"] == "object"
        && node["kind"] == "timestamp"
        && node["context"] == "paragraph"));
    assert!(index.iter().any(|node| {
        node["category"] == "element"
            && node["kind"] == "src-block"
            && node["summary"]["language"] == "python"
            && node["summary"]["value"]
                .as_str()
                .is_some_and(|value| value.contains("print(topic)"))
    }));
    assert!(
        payload["sourceBlocks"]
            .as_array()
            .expect("source blocks")
            .iter()
            .any(|block| block["kind"] == "inlineSource" && block["language"] == "python")
    );
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

#[test]
fn semantic_ast_projects_org_elements_query_packet_has_snapshot() {
    let doc = Org::parse(
        r#"#+NAME: task_runner
#+begin_src python
print("run")
#+end_src

#+NAME: shell_runner
#+begin_src shell
echo run
#+end_src
"#,
    )
    .document();

    assert_clean_projection(&doc);

    let packet = serde_json::json!({
        "schemaVersion": 1,
        "predicate": {
            "all": [
                { "kind": "src-block" },
                { "affiliatedName": "task_runner" },
                {
                    "any": [
                        { "summary": { "key": "language", "equals": "python" } },
                        { "summary": { "key": "language", "equals": "rust" } }
                    ]
                },
                { "not": { "summary": { "key": "language", "equals": "shell" } } }
            ]
        },
        "limit": 5,
    })
    .to_string();
    let query = org_elements_index_query_from_json_str(&packet).expect("query packet should parse");
    let records = doc.query_org_elements_index(&query);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].kind.as_str(), "src-block");
    assert_eq!(records[0].affiliated.name.as_deref(), Some("task_runner"));

    let packet_json: Value =
        serde_json::from_str(&doc.org_elements_index_query_packet_json(&packet).unwrap())
            .expect("packet JSON should parse");
    let payload = serde_json::json!({
        "canonicalQuery": org_elements_index_query_to_json_value(&query),
        "records": packet_json
            .as_array()
            .expect("packet result array")
            .iter()
            .map(|record| {
                serde_json::json!({
                    "kind": record["kind"].clone(),
                    "affiliatedName": record["affiliated"]["name"].clone(),
                    "summary": {
                        "language": record["summary"]["language"].clone(),
                    },
                })
            })
            .collect::<Vec<_>>(),
    });

    insta::assert_snapshot!(
        "org_elements_query_packet_contract",
        serde_json::to_string_pretty(&payload).unwrap()
    );
}

#[test]
fn semantic_ast_projects_org_elements_graph_properties_have_snapshot() {
    let doc = Org::parse(
        r#"* TODO Task A :work:
:PROPERTIES:
:OWNER: alice
:CUSTOM_ID: task-a
:END:
See [[https://example.test][example]].
** Goal
"#,
    )
    .document();

    assert_clean_projection(&doc);

    let graph = doc.org_elements_graph();
    let headline = doc
        .query_org_elements_index(
            &OrgElementsIndexQuery::new()
                .category(OrgElementsIndexCategory::Section)
                .property_eq(":CUSTOM_ID", "task-a"),
        )
        .into_iter()
        .next()
        .expect("headline selected by inherited property");
    let link = doc
        .query_org_elements_index(
            &OrgElementsIndexQuery::new()
                .category(OrgElementsIndexCategory::Object)
                .kind("link"),
        )
        .into_iter()
        .next()
        .expect("link selected");
    let child_headlines = doc.query_org_elements_index(
        &OrgElementsIndexQuery::new()
            .category(OrgElementsIndexCategory::Section)
            .property_eq(":parent", graph.root_id.as_usize()),
    );
    assert_eq!(child_headlines.len(), 1);
    let payload = serde_json::json!({
        "headline": {
            "id": headline.id.as_usize(),
            "parentId": headline.parent_id.map(|id| id.as_usize()),
            "childIds": headline.child_ids.iter().map(|id| id.as_usize()).collect::<Vec<_>>(),
            "kind": headline.kind.as_str(),
            "outlinePath": headline.outline_path,
            "properties": {
                ":begin": snapshot_summary_value(headline.properties.get(":begin")),
                ":end": snapshot_summary_value(headline.properties.get(":end")),
                ":contents-begin": snapshot_summary_value(headline.properties.get(":contents-begin")),
                ":contents-end": snapshot_summary_value(headline.properties.get(":contents-end")),
                ":post-affiliated": snapshot_summary_value(headline.properties.get(":post-affiliated")),
                ":post-blank": snapshot_summary_value(headline.properties.get(":post-blank")),
                ":parent": snapshot_summary_value(headline.properties.get(":parent")),
                ":raw-value": snapshot_summary_value(headline.properties.get(":raw-value")),
                ":todo-keyword": snapshot_summary_value(headline.properties.get(":todo-keyword")),
                ":todo-type": snapshot_summary_value(headline.properties.get(":todo-type")),
                ":tags": snapshot_summary_value(headline.properties.get(":tags")),
                ":OWNER": snapshot_summary_value(headline.properties.get(":OWNER")),
                ":CUSTOM_ID": snapshot_summary_value(headline.properties.get(":CUSTOM_ID")),
            },
        },
        "linkLineage": graph
            .lineage(link.id)
            .into_iter()
            .map(|record| serde_json::json!({
                "id": record.id.as_usize(),
                "kind": record.kind.as_str(),
                "category": record.category.as_str(),
            }))
            .collect::<Vec<_>>(),
    });

    insta::assert_snapshot!(serde_json::to_string_pretty(&payload).unwrap());
}

#[test]
fn semantic_ast_projects_org_elements_graph_relation_queries_have_snapshot() {
    let doc = Org::parse(
        r#"* Task
** Evidence
[[https://example.test][inside]]
** Context
[[https://example.test][outside]]
"#,
    )
    .document();

    assert_clean_projection(&doc);

    let graph = doc.org_elements_graph();
    let task = doc
        .query_org_elements_index(
            &OrgElementsIndexQuery::new()
                .category(OrgElementsIndexCategory::Section)
                .kind("headline")
                .property_eq(":raw-value", "Task"),
        )
        .into_iter()
        .next()
        .expect("task headline");
    let evidence = doc
        .query_org_elements_index(
            &OrgElementsIndexQuery::new()
                .category(OrgElementsIndexCategory::Section)
                .kind("headline")
                .property_eq(":raw-value", "Evidence"),
        )
        .into_iter()
        .next()
        .expect("evidence headline");
    let direct_headline_children = graph.query(
        &OrgElementsIndexQuery::new()
            .category(OrgElementsIndexCategory::Section)
            .kind("headline")
            .child_of(task.id),
    );
    let evidence_links = graph.query(
        &OrgElementsIndexQuery::new()
            .category(OrgElementsIndexCategory::Object)
            .kind("link")
            .descendant_of(evidence.id),
    );
    let inside_link_id = evidence_links.first().expect("evidence link").id;
    let link_headline_ancestors = graph.query(
        &OrgElementsIndexQuery::new()
            .category(OrgElementsIndexCategory::Section)
            .kind("headline")
            .ancestor_of(inside_link_id),
    );
    let evidence_at = graph.query(
        &OrgElementsIndexQuery::new()
            .category(OrgElementsIndexCategory::Section)
            .kind("headline")
            .at(evidence.id),
    );

    let payload = serde_json::json!({
        "directHeadlineChildren": graph_query_snapshot_records(direct_headline_children),
        "evidenceLinks": graph_query_snapshot_records(evidence_links),
        "linkHeadlineAncestors": graph_query_snapshot_records(link_headline_ancestors),
        "evidenceAt": graph_query_snapshot_records(evidence_at),
    });

    insta::assert_snapshot!(serde_json::to_string_pretty(&payload).unwrap());
}

#[test]
fn semantic_ast_projects_org_element_alignment_gap_has_snapshot() {
    let doc = Org::parse(
        r#"#+TITLE: Alignment Fixture

#+CAPTION: Caption
#+DATA: data
#+HEADER: :var x=1
#+HEADERS: :var y=2
#+LABEL: lbl
#+NAME: named-src
#+PLOT: title
#+RESNAME: res
#+RESULT: old
#+RESULTS: output
#+SOURCE: source
#+SRCNAME: srcname
#+TBLNAME: tbl
#+begin_src rust :results output
fn main() {}
#+end_src

:PROPERTIES:
:GLOBAL: yes
:END:

#+CALL: build()

* TODO Root :audit:
SCHEDULED: <2026-06-10 Wed> DEADLINE: <2026-06-11 Thu> CLOSED: [2026-06-09 Tue]
:PROPERTIES:
:CUSTOM_ID: root
:OWNER: alice
:END:
:LOGBOOK:
CLOCK: [2026-06-10 Wed 10:00]--[2026-06-10 Wed 10:30] =>  0:30
:END:
A paragraph with *bold* /italic/ _underline_ +strike+ H_2 x^2 =code= ~verb~ \alpha \(x+y\) @@html:<span>@@ [[https://example.test][link]] <<target>> <<<radio>>> {{{macro(arg)}}} [33%] <2026-06-10 Wed> src_python{print(1)} call_build() [cite:@doe] [fn:one].
\\
- [ ] item :: tag
| A | B |
|---+---|
| 1 | 2 |
[fn:one] Footnote body.
# Comment line
%%(org-anniversary 1956 5 14)
: fixed width line
-----
\begin{equation}
x = y
\end{equation}
#+begin_example
example
#+end_example
#+begin_export html
<div></div>
#+end_export
#+begin_quote
quote
#+end_quote
#+begin_verse
verse
#+end_verse
#+begin_center
center
#+end_center
#+begin_comment
comment block
#+end_comment
#+BEGIN: clocktable
#+END:
#+begin_note
special block
#+end_note
*************** TODO Inline Task
Inline body.
*************** END
** Child
"#,
    )
    .document();

    assert_clean_projection(&doc);
    let planning_records = doc.query_org_elements_index(
        &OrgElementsIndexQuery::new()
            .category(OrgElementsIndexCategory::Element)
            .kind("planning")
            .property_contains(":scheduled", "2026-06-10"),
    );
    assert_eq!(planning_records.len(), 1);
    let property_drawers = doc.query_org_elements_index(
        &OrgElementsIndexQuery::new()
            .category(OrgElementsIndexCategory::Element)
            .kind("property-drawer"),
    );
    assert_eq!(property_drawers.len(), 1);
    assert!(property_drawers.iter().any(|record| {
        record.context == "headline"
            && record.summary.get("properties") == Some(&OrgElementsIndexSummaryValue::Integer(2))
    }));
    let citation_references = doc.query_org_elements_index(
        &OrgElementsIndexQuery::new()
            .category(OrgElementsIndexCategory::Object)
            .kind("citation-reference")
            .property_eq(":key", "doe"),
    );
    assert_eq!(citation_references.len(), 1);
    let section_records = doc.query_org_elements_index(
        &OrgElementsIndexQuery::new()
            .category(OrgElementsIndexCategory::Element)
            .kind("section"),
    );
    assert_eq!(section_records.len(), 1);
    let diary_sexps = doc.query_org_elements_index(
        &OrgElementsIndexQuery::new()
            .category(OrgElementsIndexCategory::Element)
            .kind("diary-sexp"),
    );
    assert_eq!(diary_sexps.len(), 1);
    assert_eq!(
        diary_sexps[0].summary.get("raw"),
        Some(&OrgElementsIndexSummaryValue::Text(
            "%%(org-anniversary 1956 5 14)\n".to_string()
        ))
    );

    let records = doc.org_elements_index();
    let mut official_element_like = string_set(UPSTREAM_ORG_ELEMENT_ALL_ELEMENTS);
    official_element_like.insert("org-data".to_string());
    let official_objects = string_set(UPSTREAM_ORG_ELEMENT_ALL_OBJECTS);
    let official_affiliated_keywords = string_set(UPSTREAM_ORG_ELEMENT_AFFILIATED_KEYWORDS);
    let official_standard_properties = string_set(UPSTREAM_ORG_ELEMENT_STANDARD_PROPERTIES);

    let current_element_like = records
        .iter()
        .filter(|record| {
            matches!(
                record.category,
                OrgElementsIndexCategory::Document
                    | OrgElementsIndexCategory::Section
                    | OrgElementsIndexCategory::Element
                    | OrgElementsIndexCategory::Property
                    | OrgElementsIndexCategory::Keyword
            )
        })
        .map(|record| record.kind.as_str().to_string())
        .collect::<BTreeSet<_>>();
    let current_objects = records
        .iter()
        .filter(|record| record.category == OrgElementsIndexCategory::Object)
        .map(|record| record.kind.as_str().to_string())
        .collect::<BTreeSet<_>>();
    let current_properties = records
        .iter()
        .flat_map(|record| record.properties.keys().cloned())
        .collect::<BTreeSet<_>>();
    let current_affiliated_keyword_keys = records
        .iter()
        .filter(|record| record.context == "affiliatedKeyword")
        .filter_map(|record| match record.summary.get("key") {
            Some(OrgElementsIndexSummaryValue::Text(key)) => Some(key.to_string()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let current_affiliated_fields = records
        .iter()
        .filter(|record| record.affiliated.name.is_some())
        .map(|_| "name".to_string())
        .collect::<BTreeSet<_>>();
    let missing_element_like_kinds = difference(&official_element_like, &current_element_like);
    let orgize_specific_element_like_kinds =
        difference(&current_element_like, &official_element_like);
    let missing_object_kinds = difference(&official_objects, &current_objects);
    let orgize_specific_object_kinds = difference(&current_objects, &official_objects);
    let missing_affiliated_keyword_records = difference(
        &official_affiliated_keywords,
        &current_affiliated_keyword_keys,
    );
    let missing_standard_properties =
        difference(&official_standard_properties, &current_properties);
    assert_eq!(
        missing_standard_properties,
        string_vec(ORG_ELEMENT_INTENTIONALLY_UNMAPPED_STANDARD_PROPERTIES)
    );

    insta::assert_snapshot!(
        "org_element_kind_alignment",
        serde_json::to_string_pretty(&serde_json::json!({
            "missingElementLikeKinds": missing_element_like_kinds,
            "orgizeSpecificElementLikeKinds": orgize_specific_element_like_kinds,
            "missingObjectKinds": missing_object_kinds,
            "orgizeSpecificObjectKinds": orgize_specific_object_kinds,
            "missingAffiliatedKeywordRecords": missing_affiliated_keyword_records,
        }))
        .unwrap()
    );

    insta::assert_snapshot!(
        "org_element_standard_property_gap",
        serde_json::to_string_pretty(&serde_json::json!({
            "presentStandardProperties": intersection(
                &official_standard_properties,
                &current_properties
            ),
            "intentionallyUnmappedStandardProperties": missing_standard_properties,
        }))
        .unwrap()
    );

    let payload = serde_json::json!({
        "baseline": {
            "source": "bzg/org-mode b470d81 org-element.el",
            "elements": UPSTREAM_ORG_ELEMENT_ALL_ELEMENTS,
            "objects": UPSTREAM_ORG_ELEMENT_ALL_OBJECTS,
            "greaterElements": UPSTREAM_ORG_ELEMENT_GREATER_ELEMENTS,
            "recursiveObjects": UPSTREAM_ORG_ELEMENT_RECURSIVE_OBJECTS,
            "affiliatedKeywords": UPSTREAM_ORG_ELEMENT_AFFILIATED_KEYWORDS,
            "standardProperties": UPSTREAM_ORG_ELEMENT_STANDARD_PROPERTIES,
        },
        "current": {
            "elementLikeKinds": current_element_like.iter().collect::<Vec<_>>(),
            "objectKinds": current_objects.iter().collect::<Vec<_>>(),
            "affiliatedFields": current_affiliated_fields.iter().collect::<Vec<_>>(),
            "affiliatedKeywordRecords": current_affiliated_keyword_keys.iter().collect::<Vec<_>>(),
            "propertyKeys": current_properties.iter().collect::<Vec<_>>(),
        },
        "gaps": {
            "missingElementLikeKinds": missing_element_like_kinds,
            "orgizeSpecificElementLikeKinds": orgize_specific_element_like_kinds,
            "missingObjectKinds": missing_object_kinds,
            "orgizeSpecificObjectKinds": orgize_specific_object_kinds,
            "missingAffiliatedKeywordRecords": missing_affiliated_keyword_records,
            "missingStandardProperties": missing_standard_properties,
        },
    });

    insta::assert_snapshot!(serde_json::to_string_pretty(&payload).unwrap());
}

#[test]
fn semantic_ast_projects_upstream_org_element_defconsts_match_checked_in_baseline() {
    let upstream = upstream_org_element_defconsts();
    assert_eq!(
        upstream.all_elements,
        string_vec(UPSTREAM_ORG_ELEMENT_ALL_ELEMENTS)
    );
    assert_eq!(
        upstream.greater_elements,
        string_vec(UPSTREAM_ORG_ELEMENT_GREATER_ELEMENTS)
    );
    assert_eq!(
        upstream.all_objects,
        string_vec(UPSTREAM_ORG_ELEMENT_ALL_OBJECTS)
    );
    assert_eq!(
        upstream.recursive_objects,
        string_vec(UPSTREAM_ORG_ELEMENT_RECURSIVE_OBJECTS)
    );
    assert_eq!(
        upstream.affiliated_keywords,
        string_vec(UPSTREAM_ORG_ELEMENT_AFFILIATED_KEYWORDS)
    );

    insta::assert_snapshot!(
        serde_json::to_string_pretty(&serde_json::json!({
            "source": "bzg/org-mode .data/org-mode/lisp/org-element.el",
            "allElements": upstream.all_elements,
            "greaterElements": upstream.greater_elements,
            "allObjects": upstream.all_objects,
            "recursiveObjects": upstream.recursive_objects,
            "affiliatedKeywords": upstream.affiliated_keywords,
        }))
        .unwrap()
    );
}

#[test]
fn semantic_ast_projects_org_guide_real_document_org_elements_regression_has_snapshot() {
    let doc = Org::parse(include_str!("../../../.data/org-mode/doc/org-guide.org")).document();

    assert_clean_projection(&doc);
    let records = doc.org_elements_index();
    let selected_kind_counts = selected_kind_counts(
        &records,
        &[
            "org-data",
            "headline",
            "section",
            "paragraph",
            "src-block",
            "plain-list",
            "table",
            "link",
            "timestamp",
        ],
    );
    let top_level_headlines = records
        .iter()
        .filter(|record| {
            record.category == OrgElementsIndexCategory::Section
                && record.kind.as_str() == "headline"
                && record.outline_path.len() == 1
        })
        .filter_map(|record| match record.summary.get("title") {
            Some(OrgElementsIndexSummaryValue::Text(title)) => Some(title.clone()),
            _ => None,
        })
        .take(12)
        .collect::<Vec<_>>();

    insta::assert_snapshot!(
        serde_json::to_string_pretty(&serde_json::json!({
            "source": ".data/org-mode/doc/org-guide.org",
            "recordCount": records.len(),
            "selectedKindCounts": selected_kind_counts,
            "topLevelHeadlines": top_level_headlines,
        }))
        .unwrap()
    );
}

fn graph_query_snapshot_records(
    records: Vec<&orgize::ast::OrgElementsIndexRecord<orgize::ast::ParsedAnnotation>>,
) -> Vec<Value> {
    records
        .into_iter()
        .map(|record| {
            serde_json::json!({
                "id": record.id.as_usize(),
                "parentId": record.parent_id.map(|id| id.as_usize()),
                "category": record.category.as_str(),
                "kind": record.kind.as_str(),
                "rawValue": snapshot_summary_value(record.properties.get(":raw-value")),
                "path": snapshot_summary_value(record.properties.get(":path")),
                "outlinePath": record.outline_path,
            })
        })
        .collect()
}

fn snapshot_summary_value(value: Option<&OrgElementsIndexSummaryValue>) -> Value {
    match value {
        Some(OrgElementsIndexSummaryValue::Null) | None => Value::Null,
        Some(OrgElementsIndexSummaryValue::Bool(value)) => Value::Bool(*value),
        Some(OrgElementsIndexSummaryValue::Integer(value)) => serde_json::json!(value),
        Some(OrgElementsIndexSummaryValue::Text(value)) => serde_json::json!(value),
        Some(OrgElementsIndexSummaryValue::StringList(value)) => serde_json::json!(value),
    }
}

fn string_set(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| value.to_string()).collect()
}

fn string_vec(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

fn difference(left: &BTreeSet<String>, right: &BTreeSet<String>) -> Vec<String> {
    left.difference(right).cloned().collect()
}

fn intersection(left: &BTreeSet<String>, right: &BTreeSet<String>) -> Vec<String> {
    left.intersection(right).cloned().collect()
}

fn selected_kind_counts(
    records: &[OrgElementsIndexRecord<ParsedAnnotation>],
    selected_kinds: &[&str],
) -> BTreeMap<String, usize> {
    selected_kinds
        .iter()
        .map(|kind| {
            (
                (*kind).to_string(),
                records
                    .iter()
                    .filter(|record| record.kind.as_str() == *kind)
                    .count(),
            )
        })
        .collect()
}

#[derive(Debug)]
struct UpstreamOrgElementDefconsts {
    all_elements: Vec<String>,
    greater_elements: Vec<String>,
    all_objects: Vec<String>,
    recursive_objects: Vec<String>,
    affiliated_keywords: Vec<String>,
}

fn upstream_org_element_defconsts() -> UpstreamOrgElementDefconsts {
    let source = include_str!("../../../.data/org-mode/lisp/org-element.el");
    UpstreamOrgElementDefconsts {
        all_elements: elisp_defconst_quoted_list(source, "org-element-all-elements"),
        greater_elements: elisp_defconst_quoted_list(source, "org-element-greater-elements"),
        all_objects: elisp_defconst_quoted_list(source, "org-element-all-objects"),
        recursive_objects: elisp_defconst_quoted_list(source, "org-element-recursive-objects"),
        affiliated_keywords: elisp_defconst_quoted_list(source, "org-element-affiliated-keywords"),
    }
}

fn elisp_defconst_quoted_list(source: &str, name: &str) -> Vec<String> {
    let marker = format!("(defconst {name}");
    let defconst = source
        .split_once(&marker)
        .unwrap_or_else(|| panic!("missing upstream defconst `{name}`"))
        .1;
    let body_start = defconst
        .find("'(")
        .unwrap_or_else(|| panic!("missing quoted list for upstream defconst `{name}`"))
        + 2;
    let body = quoted_list_body(&defconst[body_start..]);
    elisp_list_values(body)
}

fn quoted_list_body(source: &str) -> &str {
    let mut depth = 1usize;
    for (index, ch) in source.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return &source[..index];
                }
            }
            _ => {}
        }
    }
    source
}

fn elisp_list_values(source: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut chars = source.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_whitespace() {
            continue;
        }
        if ch == '"' {
            let mut value = String::new();
            while let Some(ch) = chars.next() {
                match ch {
                    '"' => break,
                    '\\' => {
                        if let Some(escaped) = chars.next() {
                            value.push(escaped);
                        }
                    }
                    _ => value.push(ch),
                }
            }
            values.push(value);
            continue;
        }
        let mut value = ch.to_string();
        while let Some(next) = chars.peek().copied() {
            if next.is_whitespace() || next == '(' || next == ')' {
                break;
            }
            value.push(next);
            chars.next();
        }
        values.push(value);
    }
    values
}

const UPSTREAM_ORG_ELEMENT_ALL_ELEMENTS: &[&str] = &[
    "babel-call",
    "center-block",
    "clock",
    "comment",
    "comment-block",
    "diary-sexp",
    "drawer",
    "dynamic-block",
    "example-block",
    "export-block",
    "fixed-width",
    "footnote-definition",
    "headline",
    "horizontal-rule",
    "inlinetask",
    "item",
    "keyword",
    "latex-environment",
    "node-property",
    "paragraph",
    "plain-list",
    "planning",
    "property-drawer",
    "quote-block",
    "section",
    "special-block",
    "src-block",
    "table",
    "table-row",
    "verse-block",
];

const UPSTREAM_ORG_ELEMENT_ALL_OBJECTS: &[&str] = &[
    "bold",
    "citation",
    "citation-reference",
    "code",
    "entity",
    "export-snippet",
    "footnote-reference",
    "inline-babel-call",
    "inline-src-block",
    "italic",
    "line-break",
    "latex-fragment",
    "link",
    "macro",
    "radio-target",
    "statistics-cookie",
    "strike-through",
    "subscript",
    "superscript",
    "table-cell",
    "target",
    "timestamp",
    "underline",
    "verbatim",
];

const UPSTREAM_ORG_ELEMENT_GREATER_ELEMENTS: &[&str] = &[
    "center-block",
    "drawer",
    "dynamic-block",
    "footnote-definition",
    "headline",
    "inlinetask",
    "item",
    "plain-list",
    "property-drawer",
    "quote-block",
    "section",
    "special-block",
    "table",
    "org-data",
];

const UPSTREAM_ORG_ELEMENT_RECURSIVE_OBJECTS: &[&str] = &[
    "bold",
    "citation",
    "footnote-reference",
    "italic",
    "link",
    "subscript",
    "radio-target",
    "strike-through",
    "superscript",
    "table-cell",
    "underline",
];

const UPSTREAM_ORG_ELEMENT_AFFILIATED_KEYWORDS: &[&str] = &[
    "CAPTION", "DATA", "HEADER", "HEADERS", "LABEL", "NAME", "PLOT", "RESNAME", "RESULT",
    "RESULTS", "SOURCE", "SRCNAME", "TBLNAME",
];

const UPSTREAM_ORG_ELEMENT_STANDARD_PROPERTIES: &[&str] = &[
    ":begin",
    ":post-affiliated",
    ":contents-begin",
    ":contents-end",
    ":end",
    ":post-blank",
    ":secondary",
    ":mode",
    ":granularity",
    ":cached",
    ":org-element--cache-sync-key",
    ":robust-begin",
    ":robust-end",
    ":true-level",
    ":buffer",
    ":deferred",
    ":structure",
    ":parent",
];

const ORG_ELEMENT_INTENTIONALLY_UNMAPPED_STANDARD_PROPERTIES: &[&str] = &[
    ":buffer",
    ":cached",
    ":deferred",
    ":granularity",
    ":mode",
    ":org-element--cache-sync-key",
    ":robust-begin",
    ":robust-end",
    ":secondary",
    ":structure",
];

#[test]
fn semantic_ast_projects_org_element_selector_uses_affiliated_name() {
    let doc = Org::parse(
        r#"#+name: plan_contract_graph
#+begin_src mermaid
flowchart LR
  P["PLAN_POLICY.org"] --> S["_sdd_template.org"]
#+end_src

#+name: image-name
#+caption: Flowhub diagram
[[file:diagram.png]]

#+name: table-name
| Key | Value |
| a   | b     |
"#,
    )
    .document();

    assert_clean_projection(&doc);

    let selector = OrgElementSelector::parse_plist(
        r##"(:org-element (:type src-block :name "plan_contract_graph" :language "mermaid"))"##,
    )
    .expect("selector should parse");
    let graph_records = doc.select_org_elements(&selector);
    assert_eq!(graph_records.len(), 1);
    assert_eq!(graph_records[0].kind.as_str(), "src-block");
    assert_eq!(
        graph_records[0].affiliated.name.as_deref(),
        Some("plan_contract_graph")
    );
    assert_eq!(
        graph_records[0].summary.get("language"),
        Some(&OrgElementsIndexSummaryValue::Text("mermaid".to_string()))
    );

    let image_records = doc.query_org_elements_index(
        &OrgElementsIndexQuery::new()
            .category(OrgElementsIndexCategory::Element)
            .kind("paragraph")
            .affiliated_name("image-name"),
    );
    assert_eq!(image_records.len(), 1);

    let table_records = doc.query_org_elements_index(
        &OrgElementsIndexQuery::new()
            .category(OrgElementsIndexCategory::Element)
            .kind("table")
            .affiliated_name("table-name"),
    );
    assert_eq!(table_records.len(), 1);

    let index_json: Value =
        serde_json::from_str(&doc.org_elements_index_json()).expect("index JSON should parse");
    assert!(index_json.as_array().expect("index").iter().any(|record| {
        record["kind"] == "src-block"
            && record["affiliated"]["name"] == "plan_contract_graph"
            && record["summary"]["language"] == "mermaid"
    }));
}

#[test]
fn semantic_ast_projects_org_element_selector_rejects_invalid_plists() {
    assert_eq!(
        OrgElementSelector::parse_plist(
            r##"(:org-element :type src-block :name "plan_contract_graph")"##,
        )
        .unwrap_err(),
        OrgElementSelectorParseError::InvalidShape
    );
    assert_eq!(
        OrgElementSelector::parse_plist(r##"(:org-element (:name "missing-type"))"##).unwrap_err(),
        OrgElementSelectorParseError::MissingType
    );
    assert_eq!(
        OrgElementSelector::parse_plist(r#"(:org-element (:type src-block :unknown true))"#)
            .unwrap_err(),
        OrgElementSelectorParseError::UnknownKey(":unknown".to_string())
    );
}

#[cfg(feature = "datafusion-sql")]
#[tokio::test]
async fn semantic_ast_projects_org_elements_sql_query_uses_datafusion() {
    let doc = Org::parse(
        r#"#+name: plan_contract_graph
#+begin_src mermaid
flowchart LR
  P["PLAN_POLICY.org"] --> S["_sdd_template.org"]
#+end_src

* Policy Entry
See [[https://example.test][example]].
"#,
    )
    .document();

    assert_clean_projection(&doc);

    let batches = doc
        .org_elements_sql_query(
            r#"
SELECT kind, affiliated_name, language, source_start_line
FROM org_elements
WHERE affiliated_name = 'plan_contract_graph'
ORDER BY ordinal
"#,
        )
        .await
        .expect("DataFusion SQL query should run");
    assert_eq!(batches.len(), 1);
    let batch = &batches[0];
    assert_eq!(batch.num_rows(), 1);

    let kind = batch
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("kind should be a UTF-8 column");
    let affiliated_name = batch
        .column(1)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("affiliated_name should be a UTF-8 column");
    let language = batch
        .column(2)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("language should be a UTF-8 column");
    let source_start_line = batch
        .column(3)
        .as_any()
        .downcast_ref::<Int64Array>()
        .expect("source_start_line should be an int64 column");
    assert_eq!(kind.value(0), "src-block");
    assert_eq!(affiliated_name.value(0), "plan_contract_graph");
    assert_eq!(language.value(0), "mermaid");
    assert_eq!(source_start_line.value(0), 1);
}
