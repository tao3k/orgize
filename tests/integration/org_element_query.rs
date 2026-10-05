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
fn named_query_observation_binds_graph_local_ids_to_exact_source() {
    use orgize::{
        org_aot::{org_event_parser_digest, org_graph_spec, parse_org_aot},
        org_aot_edit::org_source_digest,
    };

    let first_source = "* TODO Remember\n";
    let first = parse_org_aot(first_source).expect("first Scheme-AOT parse");
    let first_observation = first
        .query_named_source_observation("tasks.open", 0)
        .expect("named query on first source");
    assert_eq!(first.to_org(), first_source);
    assert_eq!(
        first_observation.source_digest(),
        org_source_digest(first_source)
    );
    assert_eq!(first_observation.source_bytes(), first_source.len());
    assert_eq!(first_observation.parser_digest(), org_event_parser_digest());
    assert_eq!(
        first_observation.graph_digest(),
        org_graph_spec().projection_digest
    );
    assert_eq!(first_observation.rule().id, "tasks.open");
    assert_eq!(first_observation.scope_id(), 0);
    assert_eq!(first_observation.matches().len(), 1);
    assert!(first_observation.matches_source(first_source));
    assert!(!first_observation.matches_source("* TODO Remember\r\n"));
    let selected = first_observation.matches()[0];
    assert!(first_source[selected.start_byte..selected.end_byte].contains("TODO Remember"));

    let later_source = "* Prelude\n* TODO Remember\n";
    let later = parse_org_aot(later_source).expect("later Scheme-AOT parse");
    let later_observation = later
        .query_named_source_observation("tasks.open", 0)
        .expect("named query on later source");
    assert!(!first_observation.matches_source(later_source));
    assert_ne!(
        first_observation.source_digest(),
        later_observation.source_digest()
    );
    assert_ne!(
        first_observation.matches()[0].id,
        later_observation.matches()[0].id
    );
    assert!(later_observation.matches()[0].start_byte > first_observation.matches()[0].start_byte);
}

#[test]
fn named_query_observation_retains_effective_parse_configuration() {
    use orgize::{
        ParseConfig,
        org_aot::{parse_org_aot, parse_org_aot_with_config},
    };

    let source = "* WAIT Task\n";
    let default = parse_org_aot(source).expect("default parse");
    let default_observation = default
        .query_named_source_observation("tasks.open", 0)
        .expect("default query");
    let mut config = ParseConfig::default();
    config.todo_keywords = (vec!["WAIT".into()], vec!["DONE".into()]);
    let configured = parse_org_aot_with_config(source, &config).expect("configured parse");
    let configured_observation = configured
        .query_named_source_observation("tasks.open", 0)
        .expect("configured query");
    assert_eq!(
        default_observation.source_digest(),
        configured_observation.source_digest()
    );
    assert!(default_observation.matches().is_empty());
    assert_eq!(configured_observation.matches().len(), 1);
    assert_eq!(
        configured_observation.effective_config().todo_keywords.0,
        vec!["WAIT".to_string()]
    );
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
    let source = "#+TODO: WAIT | DONE\n* Team\n** WAIT [#A] Review patch :work:\nEvidence [cite:@doe2020]\n** DONE Review release\n** WAIT Audit\n";
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
        .find(|record| record.field("title") == Some("WAIT [#A] Review patch :work:"))
        .expect("review headline");
    assert_eq!(
        document.query_with_pack(
            &customer_query_plan::QUERIES,
            "customer.active-review",
            team.id,
        ),
        Ok(vec![review.id])
    );
    let citation_reference = document
        .records()
        .iter()
        .find(|record| {
            record.kind == "citation-reference" && record.field("key") == Some("doe2020")
        })
        .expect("Scheme citation reference is available to custom Element queries");
    assert_eq!(
        document.query_with_pack(&customer_query_plan::QUERIES, "customer.cited-evidence", 0),
        Ok(vec![citation_reference.id])
    );
    for id in [
        "customer.normalized-title",
        "customer.priority",
        "customer.tagged",
        "customer.raw-value",
        "customer.todo-contains",
    ] {
        assert_eq!(
            document.query_with_pack(&customer_query_plan::QUERIES, id, team.id),
            Ok(vec![review.id]),
            "customer query {id} uses the Scheme-AOT headline projection"
        );
    }
}
