use crate::ast::{
    ORG_ELEMENTS_QUERY_EXPRESSION_EXAMPLES, ORG_ELEMENTS_QUERY_EXPRESSION_SURFACE_GUIDE,
    OrgElementQueryPredicate, OrgElementsIndexCategory, OrgElementsIndexRelation,
    org_elements_index_query_from_expr_str,
};

#[test]
fn parses_complete_index_query_expression_surface() {
    let query = org_elements_index_query_from_expr_str(
        r#"
(org-elements-query
  (category element)
  (kind src-block)
  (affiliated-name "build")
  (context "src-block")
  (outline-path-prefix ("Plan" "Evidence"))
  (outline-path-exact-len 2)
  (property :CUSTOM_ID "abc")
  (property-contains TAGS "work")
  (summary language "shell")
  (summary-contains text "trace")
  (child-of 1 2)
  (descendant-of (3 4))
  (ancestor-of 5)
  (at 6)
  (predicate
    (or
      (kind link)
      (= (summary hasText) t)))
  (limit 7))
"#,
    )
    .expect("query expression should parse");

    assert_eq!(query.category, Some(OrgElementsIndexCategory::Element));
    assert_eq!(
        query.kind.as_ref().map(|kind| kind.as_str()),
        Some("src-block")
    );
    assert_eq!(query.affiliated_name.as_deref(), Some("build"));
    assert_eq!(query.context.as_deref(), Some("src-block"));
    assert_eq!(query.outline_path_prefix, vec!["Plan", "Evidence"]);
    assert_eq!(query.outline_path_exact_len, Some(2));
    assert_eq!(query.property_equals.len(), 1);
    assert_eq!(query.property_contains.len(), 1);
    assert_eq!(query.summary_equals.len(), 1);
    assert_eq!(query.summary_contains.len(), 1);
    assert_eq!(query.relations.len(), 4);
    assert!(matches!(
        query.relations[0],
        OrgElementsIndexRelation::ChildOf(_)
    ));
    assert!(matches!(
        query.relations[1],
        OrgElementsIndexRelation::DescendantOf(_)
    ));
    assert!(matches!(
        query.relations[2],
        OrgElementsIndexRelation::AncestorOf(_)
    ));
    assert!(matches!(
        query.relations[3],
        OrgElementsIndexRelation::At(_)
    ));
    assert!(matches!(query.predicate, OrgElementQueryPredicate::Any(_)));
    assert_eq!(query.limit, Some(7));
}

#[test]
fn parses_org_element_ast_relation_aliases() {
    let query = org_elements_index_query_from_expr_str(
        r#"
(org-elements-query
  (contents-of 1)
  (within-contents-of 2)
  (lineage-of 3)
  (headline :contents-of 4 :within-contents-of 5 :lineage-of 6))
"#,
    )
    .expect("Org Element API relation aliases should parse");

    assert_eq!(
        query.kind.as_ref().map(|kind| kind.as_str()),
        Some("headline")
    );
    assert_eq!(query.relations.len(), 6);
    assert!(matches!(
        query.relations[0],
        OrgElementsIndexRelation::ChildOf(_)
    ));
    assert!(matches!(
        query.relations[1],
        OrgElementsIndexRelation::DescendantOf(_)
    ));
    assert!(matches!(
        query.relations[2],
        OrgElementsIndexRelation::AncestorOf(_)
    ));
    assert!(matches!(
        query.relations[3],
        OrgElementsIndexRelation::ChildOf(_)
    ));
    assert!(matches!(
        query.relations[4],
        OrgElementsIndexRelation::DescendantOf(_)
    ));
    assert!(matches!(
        query.relations[5],
        OrgElementsIndexRelation::AncestorOf(_)
    ));
}

#[test]
fn parses_every_agent_facing_query_surface_example() {
    for example in ORG_ELEMENTS_QUERY_EXPRESSION_EXAMPLES {
        org_elements_index_query_from_expr_str(example)
            .unwrap_or_else(|err| panic!("guide example should parse: {example}: {err}"));
    }
}

#[test]
fn guide_mentions_discoverability_anchors_for_agent_search() {
    let guide = ORG_ELEMENTS_QUERY_EXPRESSION_SURFACE_GUIDE.join("\n");
    assert!(guide.contains("relation"));
    assert!(guide.contains("contents"));
    assert!(guide.contains("lineage"));
    assert!(guide.contains("secondary"));
    assert!(guide.contains("json index"));
}
