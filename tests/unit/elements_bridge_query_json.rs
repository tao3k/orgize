use serde_json::json;

use crate::ast::{
    OrgElementId, OrgElementQueryPredicate, OrgElementsIndexCategory, OrgElementsIndexKind,
    OrgElementsIndexQuery, OrgElementsIndexRelation, OrgElementsIndexSummaryValue,
    org_elements_index_query_from_json_value, org_elements_index_query_to_json_value,
};

#[test]
fn parses_query_packet_into_ast() {
    let query = org_elements_index_query_from_json_value(&json!({
        "schemaVersion": 1,
        "category": "element",
        "kind": "src-block",
        "affiliatedName": "demo",
        "context": "Heading",
        "outlinePathPrefix": ["Heading"],
        "outlinePathExactLen": 1,
        "propertyEquals": [{ "key": ":CUSTOM_ID", "value": "demo" }],
        "propertyContains": [{ "key": "header-args", "needle": ":results" }],
        "summaryEquals": [{ "key": "language", "value": "rust" }],
        "summaryContains": [{ "key": "exports", "needle": "code" }],
        "relations": [{ "type": "childOf", "ids": [1, 2] }],
        "predicate": {
            "any": [
                { "kind": "src-block" },
                { "property": { "key": "CUSTOM_ID", "equals": "demo" } }
            ]
        },
        "limit": 3
    }))
    .expect("query packet should parse");

    assert_eq!(query.category, Some(OrgElementsIndexCategory::Element));
    assert_eq!(
        query.kind.as_ref().map(OrgElementsIndexKind::as_str),
        Some("src-block")
    );
    assert_eq!(query.affiliated_name.as_deref(), Some("demo"));
    assert_eq!(query.context.as_deref(), Some("Heading"));
    assert_eq!(query.outline_path_prefix, ["Heading"]);
    assert_eq!(query.outline_path_exact_len, Some(1));
    assert_eq!(query.limit, Some(3));

    assert_eq!(query.property_equals[0].key, ":CUSTOM_ID");
    assert_eq!(
        query.property_equals[0].value,
        OrgElementsIndexSummaryValue::Text("demo".to_string())
    );
    assert_eq!(query.property_contains[0].needle, ":results");
    assert_eq!(query.summary_equals[0].key, "language");
    assert_eq!(
        query.summary_equals[0].value,
        OrgElementsIndexSummaryValue::Text("rust".to_string())
    );
    assert_eq!(query.summary_contains[0].needle, "code");

    match &query.relations[..] {
        [OrgElementsIndexRelation::ChildOf(ids)] => {
            assert!(ids.contains(&OrgElementId::new(1)));
            assert!(ids.contains(&OrgElementId::new(2)));
        }
        relations => panic!("unexpected relations: {relations:?}"),
    }

    match &query.predicate {
        OrgElementQueryPredicate::Any(predicates) => {
            assert_eq!(predicates.len(), 2);
            assert!(matches!(
                &predicates[0],
                OrgElementQueryPredicate::Kind(kind) if kind.as_str() == "src-block"
            ));
            assert!(matches!(
                &predicates[1],
                OrgElementQueryPredicate::PropertyEquals(predicate)
                    if predicate.key == "CUSTOM_ID"
                        && predicate.value
                            == OrgElementsIndexSummaryValue::Text("demo".to_string())
            ));
        }
        predicate => panic!("unexpected predicate: {predicate:?}"),
    }
}

#[test]
fn renders_canonical_query_packet() {
    let query = OrgElementsIndexQuery::new()
        .category(OrgElementsIndexCategory::Object)
        .kind("link")
        .summary_contains("raw", "https")
        .predicate(OrgElementQueryPredicate::negate(
            OrgElementQueryPredicate::Context("archive".to_string()),
        ))
        .at_any([OrgElementId::new(4), OrgElementId::new(6)])
        .limit(2);

    assert_eq!(
        org_elements_index_query_to_json_value(&query),
        json!({
            "schemaVersion": 1,
            "category": "object",
            "kind": "link",
            "summaryContains": [{ "key": "raw", "needle": "https" }],
            "relations": [{ "type": "at", "ids": [4, 6] }],
            "predicate": { "not": { "context": "archive" } },
            "limit": 2
        })
    );
}
