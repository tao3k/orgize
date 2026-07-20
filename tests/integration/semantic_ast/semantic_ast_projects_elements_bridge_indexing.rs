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

use super::semantic_ast_projects_elements_bridge_fixtures::{
    ORG_ELEMENT_INTENTIONALLY_UNMAPPED_STANDARD_PROPERTIES, difference,
    graph_query_snapshot_records, selected_kind_counts, string_set, string_vec,
};

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
    assert!(
        property_drawers
            .iter()
            .any(|record| record.context == "document")
    );
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
use super::semantic_ast_projects_elements_bridge_fixtures::{
    UPSTREAM_ORG_ELEMENT_AFFILIATED_KEYWORDS, UPSTREAM_ORG_ELEMENT_ALL_ELEMENTS,
    UPSTREAM_ORG_ELEMENT_ALL_OBJECTS, UPSTREAM_ORG_ELEMENT_STANDARD_PROPERTIES, intersection,
};
use super::semantic_ast_projects_elements_bridge_fixtures::{
    UPSTREAM_ORG_ELEMENT_GREATER_ELEMENTS, UPSTREAM_ORG_ELEMENT_RECURSIVE_OBJECTS,
};
