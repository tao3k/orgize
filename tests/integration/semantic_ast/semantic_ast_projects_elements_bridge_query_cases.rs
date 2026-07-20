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

use super::semantic_ast_projects_elements_bridge_fixtures::graph_query_snapshot_records;

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
use super::semantic_ast_projects_elements_bridge_fixtures::snapshot_summary_value;
