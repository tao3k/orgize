//! Scheme-AOT inlinetask Elements and source-backed recovery.

use orgize::org_aot::parse_org_aot;
use orgize::org_element_query::{
    OrgElementFieldMatch, OrgElementPropertyRule, OrgElementQueryPack, OrgElementQueryRule,
    OrgElementRelation,
};

#[test]
fn closed_inlinetask_projects_element_body_and_next_outline() {
    let source = "* Parent\n*************** TODO Inline :work:\nSCHEDULED: <2026-05-10 Sun>\n:PROPERTIES:\n:CUSTOM_ID: inline-task\n:END:\nBody [[https://example.com][link]].\n*************** END\n* Next\n";
    let document = parse_org_aot(source).expect("Scheme-AOT inlinetask parses");
    assert_eq!(document.syntax().to_string(), source);
    let records = document.records();
    let task = records
        .iter()
        .find(|record| record.kind == "inlinetask")
        .expect("inlinetask Element");
    assert_eq!(task.field("markers"), Some("***************"));
    assert_eq!(task.field("title"), Some("TODO Inline :work:"));
    assert_eq!(task.values("tag").collect::<Vec<_>>(), ["work"]);
    assert_eq!(document.headline_todo_type(task.id), Some("todo"));
    assert_eq!(
        document.headline_todo_keyword(task.id).as_deref(),
        Some("TODO")
    );
    assert_eq!(
        document.headline_display_title(task.id).as_deref(),
        Some("Inline")
    );
    let parent_id = task.parent_id.expect("parent section");
    assert_eq!(records[parent_id].kind, "headline");
    let pack = OrgElementQueryPack {
        graph_digest: orgize::org_aot::org_graph_spec().projection_digest,
        rules: &[OrgElementQueryRule {
            id: "tasks.inline",
            node_kind: "inlinetask",
            groups: &[&[OrgElementPropertyRule {
                name: "todo-type",
                value: "todo",
                matcher: OrgElementFieldMatch::Exact,
            }]],
            relation: OrgElementRelation::ChildOf,
            target_scope: true,
        }],
    };
    assert_eq!(
        document.query_with_pack(&pack, "tasks.inline", parent_id),
        Ok(vec![task.id])
    );
    assert!(
        task.child_ids
            .iter()
            .any(|id| records[*id].kind == "planning")
    );
    assert!(
        task.child_ids
            .iter()
            .any(|id| records[*id].kind == "property-drawer")
    );
    assert!(
        task.child_ids
            .iter()
            .any(|id| records[*id].kind == "paragraph")
    );
    assert!(
        records
            .iter()
            .any(|record| { record.kind == "headline" && record.field("title") == Some("Next") })
    );
}

#[test]
fn unclosed_inlinetask_does_not_consume_following_paragraph() {
    let source = "*************** Note\nAfter text.\n";
    check_org_aot_element!(source, "inlinetask", "title" => "Note");
    let document = parse_org_aot(source).expect("unclosed inlinetask recovers");
    let records = document.records();
    let task = records
        .iter()
        .find(|record| record.kind == "inlinetask")
        .unwrap();
    let paragraph = records
        .iter()
        .find(|record| record.kind == "paragraph")
        .unwrap();
    assert_eq!(paragraph.parent_id, task.parent_id);
    assert!(usize::from(task.range.end()) <= usize::from(paragraph.range.start()));
}

#[test]
fn unclosed_inlinetask_retains_affiliated_planning_and_properties() {
    let source = "*************** TODO Note\nSCHEDULED: <2026-05-10 Sun>\n:PROPERTIES:\n:CUSTOM_ID: note\n:END:\nAfter text.\n";
    check_org_aot_element!(source, "inlinetask", "title" => "TODO Note");
    let document = parse_org_aot(source).expect("unclosed inlinetask parses");
    let records = document.records();
    let task = records
        .iter()
        .find(|record| record.kind == "inlinetask")
        .expect("inlinetask Element");
    assert!(
        task.child_ids
            .iter()
            .any(|id| records[*id].kind == "planning")
    );
    assert!(
        task.child_ids
            .iter()
            .any(|id| records[*id].kind == "property-drawer")
    );
    let paragraph = records
        .iter()
        .find(|record| record.kind == "paragraph")
        .expect("following paragraph");
    assert_eq!(paragraph.parent_id, task.parent_id);
    assert!(usize::from(task.range.end()) <= usize::from(paragraph.range.start()));
}

#[test]
fn inlinetask_inherits_file_local_todo_vocabulary() {
    let source = "#+TODO: NEXT WAIT | DONE CANCELLED\n* Parent\n*************** NEXT Review\n*************** END\n";
    check_org_aot_element!(source, "inlinetask", "title" => "NEXT Review");
    let document = parse_org_aot(source).expect("custom TODO inlinetask parses");
    let task = document
        .records()
        .iter()
        .find(|record| record.kind == "inlinetask")
        .expect("inlinetask Element");
    assert_eq!(document.headline_todo_type(task.id), Some("todo"));
    assert_eq!(
        document.headline_todo_keyword(task.id).as_deref(),
        Some("NEXT")
    );
    assert_eq!(
        document.headline_display_title(task.id).as_deref(),
        Some("Review")
    );
    let pack = OrgElementQueryPack {
        graph_digest: orgize::org_aot::org_graph_spec().projection_digest,
        rules: &[OrgElementQueryRule {
            id: "tasks.custom",
            node_kind: "inlinetask",
            groups: &[&[OrgElementPropertyRule {
                name: "todo-keyword",
                value: "NEXT",
                matcher: OrgElementFieldMatch::Exact,
            }]],
            relation: OrgElementRelation::Any,
            target_scope: false,
        }],
    };
    assert_eq!(
        document.query_with_pack(&pack, "tasks.custom", 0),
        Ok(vec![task.id])
    );
}
