//! Scheme-owned named Org Element queries and consumer AOT packs.

#[path = "customer_query_plan.rs"]
mod customer_query_plan;

macro_rules! check_org_aot_query {
    ($document:expr, $query:literal, $scope:expr => [$($record:expr),* $(,)?]) => {{
        assert_eq!(
            $document.query_named($query, $scope),
            Ok(vec![$($record.id),*]),
            "Scheme-AOT query {} selected unexpected Org Elements",
            $query
        );
    }};
}

#[test]
fn tagged_element_queries_inherit_custom_todo_state_and_scope() {
    let source = "#+SEQ_TODO: WAIT(w) | DONE(d)\n* Group\n** WAIT First\n** WAIT Review\n** WAIT Audit\n** DONE Child :work:\n* Other\n** WAIT Remote\n";
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("custom queries run over Scheme-AOT Element properties");
    let group = document
        .records()
        .iter()
        .find(|record| record.kind == "headline" && record.field("title") == Some("Group"))
        .expect("group headline");
    let first = document
        .records()
        .iter()
        .find(|record| record.field("title") == Some("WAIT First"))
        .expect("first task");
    let done = document
        .records()
        .iter()
        .find(|record| record.field("title") == Some("DONE Child :work:"))
        .expect("done task");
    let review = document
        .records()
        .iter()
        .find(|record| record.field("title") == Some("WAIT Review"))
        .expect("review task");
    let audit = document
        .records()
        .iter()
        .find(|record| record.field("title") == Some("WAIT Audit"))
        .expect("audit task");
    check_org_aot_query!(document, "tasks.open", group.id => [first, review, audit]);
    check_org_aot_query!(document, "tasks.waiting", group.id => [first, review, audit]);
    check_org_aot_query!(document, "tasks.review-or-audit", group.id => [review, audit]);
    check_org_aot_query!(document, "tasks.done", group.id => [done]);
    check_org_aot_query!(document, "headlines.child", 0 => [done]);
    assert_eq!(
        document.query_named("missing", 0),
        Err(orgize::org_element_query::OrgElementQueryError::UnknownQuery)
    );
}

#[test]
fn todo_keyword_query_obeys_file_local_declarations() {
    let source = "#+SEQ_TODO: HOLD | FINISHED\n* HOLD Review\n* WAIT is plain text\n";
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("file-local TODO declarations are parsed from Org Elements");
    check_org_aot_query!(document, "tasks.waiting", 0 => []);
    let hold = document
        .records()
        .iter()
        .find(|record| record.field("title") == Some("HOLD Review"))
        .expect("custom TODO headline");
    check_org_aot_query!(document, "tasks.open", 0 => [hold]);
}

#[test]
fn scheme_event_aot_shares_headline_queries_with_structural_entrypoint() {
    let source = "#+SEQ_TODO: HOLD | FINISHED\n* HOLD Review\n* WAIT is plain text\n";
    let structural = orgize::org_aot::parse_org_aot(source).expect("structural baseline parses");
    let events = orgize::org_aot::parse_org_event_aot(source)
        .expect("Scheme event AOT projects through the production query path");
    let structural_headlines: Vec<_> = structural
        .records()
        .iter()
        .filter(|record| record.kind == "headline")
        .map(|record| record.field("title"))
        .collect();
    let event_headlines: Vec<_> = events
        .records()
        .iter()
        .filter(|record| record.kind == "headline")
        .map(|record| record.field("title"))
        .collect();
    assert_eq!(event_headlines, structural_headlines);
    assert_eq!(
        events.query_named("tasks.open", 0),
        structural.query_named("tasks.open", 0)
    );
    assert_eq!(events.query_named("tasks.waiting", 0), Ok(Vec::new()));
}

#[test]
fn supplied_element_query_packs_fail_closed_before_scanning() {
    use orgize::org_element_query::{
        OrgElementFieldMatch, OrgElementPropertyRule, OrgElementQueryError, OrgElementQueryPack,
        OrgElementQueryRule, OrgElementRelation, org_element_query_pack,
    };

    let document = orgize::org_aot::parse_org_aot("* Task\n").expect("valid Org document");
    let stale = OrgElementQueryPack {
        graph_digest: "sha256:outdated",
        rules: org_element_query_pack().rules,
    };
    assert_eq!(
        document.query_with_pack(&stale, "tasks.open", 0),
        Err(OrgElementQueryError::StaleGraph)
    );
    let unsupported = OrgElementQueryPack {
        graph_digest: org_element_query_pack().graph_digest,
        rules: &[OrgElementQueryRule {
            id: "headline.normalized-title",
            node_kind: "headline",
            groups: &[&[OrgElementPropertyRule {
                name: "title",
                value: "Task",
                matcher: OrgElementFieldMatch::Exact,
            }]],
            relation: OrgElementRelation::Any,
            target_scope: false,
        }],
    };
    assert_eq!(
        document.query_with_pack(&unsupported, "headline.normalized-title", 0),
        Err(OrgElementQueryError::UnsupportedField)
    );
    let invalid = OrgElementQueryPack {
        graph_digest: org_element_query_pack().graph_digest,
        rules: &[OrgElementQueryRule {
            id: "headline.invalid",
            node_kind: "headline",
            groups: &[&[OrgElementPropertyRule {
                name: "not-a-projected-property",
                value: "value",
                matcher: OrgElementFieldMatch::Exact,
            }]],
            relation: OrgElementRelation::Any,
            target_scope: false,
        }],
    };
    assert_eq!(
        document.query_with_pack(&invalid, "headline.invalid", 0),
        Err(OrgElementQueryError::InvalidRule)
    );
}

#[test]
fn consumer_authored_scheme_query_pack_executes_without_gerbil() {
    let source = "#+TODO: WAIT | DONE\n* Team\n** WAIT Review patch\n** DONE Review release\n** WAIT Audit\n";
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("Cargo consumer parses from committed Scheme-AOT artifacts");
    let team = document
        .records()
        .iter()
        .find(|record| record.kind == "headline" && record.field("title") == Some("Team"))
        .expect("Team headline");
    let review = document
        .records()
        .iter()
        .find(|record| record.field("title") == Some("WAIT Review patch"))
        .expect("review headline");
    assert_eq!(
        document.query_with_pack(
            &customer_query_plan::QUERIES,
            "customer.active-review",
            team.id,
        ),
        Ok(vec![review.id])
    );
}
