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
use super::semantic_ast_projects_elements_bridge_fixtures::selected_kind_counts;
