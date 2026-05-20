use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{
        AgendaDate, AgendaMatchQuery, AgendaQuery, AgendaWorkspaceBuilder,
        AgendaWorkspaceCommandKind, AgendaWorkspaceMatchCommand, AgendaWorkspaceQuery,
        SparseTreeQuery,
    },
};
use serde_json::Value;

#[test]
fn semantic_ast_projects_tag_vocabulary_groups_and_exclusive_sets() {
    let doc = Org::parse(
        r#"#+TAGS: { @work(w) @home(h) @tennisclub(t) } laptop(l) pc(p)
#+TAGS: [ GTD : Control Persp ]
#+TAGS: [ Control : Context Task ]
#+TAGS: { Context : @Home @Work @Call }

* TODO Follow up :@Work:Task:
"#,
    )
    .document();

    assert_clean_projection(&doc);
    assert!(
        doc.tag_definitions
            .iter()
            .all(|definition| !matches!(definition.name.as_str(), "{" | "}" | "[" | "]" | ":"))
    );

    let work = doc
        .tag_definitions
        .iter()
        .find(|definition| definition.name == "@work")
        .expect("@work tag definition");
    assert_eq!(work.shortcut.as_deref(), Some("w"));
    assert!(!work.is_group);
    assert!(
        work.group
            .as_ref()
            .is_some_and(|group| { group.name.is_none() && group.exclusive })
    );

    let gtd = doc
        .tag_definitions
        .iter()
        .find(|definition| definition.name == "GTD")
        .expect("GTD group tag definition");
    assert!(gtd.is_group);
    assert!(gtd.group.is_none());

    let control = doc
        .tag_definitions
        .iter()
        .find(|definition| definition.name == "Control" && !definition.is_group)
        .expect("Control member tag definition");
    assert!(
        control
            .group
            .as_ref()
            .is_some_and(|group| { group.name.as_deref() == Some("GTD") && !group.exclusive })
    );

    let home = doc
        .tag_definitions
        .iter()
        .find(|definition| definition.name == "@Home")
        .expect("@Home member tag definition");
    assert!(
        home.group
            .as_ref()
            .is_some_and(|group| { group.name.as_deref() == Some("Context") && group.exclusive })
    );

    let payload: Value = serde_json::from_str(&doc.org_elements_json()).expect("Org elements JSON");
    assert_eq!(payload["tagDefinitions"][0]["group"]["exclusive"], true);
    assert_eq!(payload["tagDefinitions"][5]["isGroup"], true);
    assert_eq!(payload["tagDefinitions"][6]["group"]["name"], "GTD");

    insta::assert_debug_snapshot!(
        "semantic_ast__tag_vocabulary_groups",
        doc.to_bare().tag_definitions
    );
}

#[test]
fn semantic_ast_expands_tag_groups_for_agenda_sparse_workspace_and_clocktable_match() {
    let doc = Org::parse(
        r#"#+TAGS: [ GTD : Control Persp ]
#+TAGS: [ Control : Context Task ]

#+BEGIN: clocktable :scope file :match "+GTD" :maxlevel 1
#+END:

* TODO Deep task :Task:
SCHEDULED: <2026-05-20 Wed>
CLOCK: [2026-05-20 Wed 09:00]--[2026-05-20 Wed 09:30] =>  0:30
* TODO Direct control :Control:
SCHEDULED: <2026-05-20 Wed>
CLOCK: [2026-05-20 Wed 10:00]--[2026-05-20 Wed 10:30] =>  0:30
* TODO Perspective :Persp:
SCHEDULED: <2026-05-20 Wed>
CLOCK: [2026-05-20 Wed 11:00]--[2026-05-20 Wed 11:30] =>  0:30
* TODO Outside :other:
SCHEDULED: <2026-05-20 Wed>
CLOCK: [2026-05-20 Wed 12:00]--[2026-05-20 Wed 12:30] =>  0:30
"#,
    )
    .document();
    assert_clean_projection(&doc);

    let day = AgendaDate::new(2026, 5, 20);
    let agenda_titles = doc
        .agenda_entries(
            &AgendaQuery::single_day(day)
                .match_expression("+GTD")
                .expect("valid group tag match"),
        )
        .into_iter()
        .map(|entry| entry.raw_title)
        .collect::<Vec<_>>();
    assert_eq!(
        agenda_titles,
        ["Deep task", "Direct control", "Perspective"]
    );

    let excluded_titles = doc
        .agenda_entries(&AgendaQuery::single_day(day).exclude_tag("GTD"))
        .into_iter()
        .map(|entry| entry.raw_title)
        .collect::<Vec<_>>();
    assert_eq!(excluded_titles, ["Outside"]);

    let alltags_property_titles = doc
        .agenda_entries(
            &AgendaQuery::single_day(day)
                .match_expression(r#"ALLTAGS={GTD}"#)
                .expect("valid group special-property match"),
        )
        .into_iter()
        .map(|entry| entry.raw_title)
        .collect::<Vec<_>>();
    assert_eq!(
        alltags_property_titles,
        ["Deep task", "Direct control", "Perspective"]
    );

    let tags_property_titles = doc
        .agenda_entries(
            &AgendaQuery::single_day(day)
                .match_expression(r#"TAGS={Control}"#)
                .expect("valid local tag group special-property match"),
        )
        .into_iter()
        .map(|entry| entry.raw_title)
        .collect::<Vec<_>>();
    assert_eq!(tags_property_titles, ["Deep task", "Direct control"]);

    let not_gtd_property_titles = doc
        .agenda_entries(
            &AgendaQuery::single_day(day)
                .match_expression(r#"ALLTAGS<>{GTD}"#)
                .expect("valid negative group special-property match"),
        )
        .into_iter()
        .map(|entry| entry.raw_title)
        .collect::<Vec<_>>();
    assert_eq!(not_gtd_property_titles, ["Outside"]);

    let sparse_titles = doc
        .sparse_tree_projection(
            &SparseTreeQuery::new()
                .match_expression("+Control")
                .expect("valid nested group tag match"),
        )
        .cards
        .into_iter()
        .map(|card| card.title)
        .collect::<Vec<_>>();
    assert_eq!(sparse_titles, ["Deep task", "Direct control"]);

    let sparse_alltags_property_titles = doc
        .sparse_tree_projection(
            &SparseTreeQuery::new()
                .match_expression(r#"ALLTAGS={GTD}"#)
                .expect("valid sparse group special-property match"),
        )
        .cards
        .into_iter()
        .map(|card| card.title)
        .collect::<Vec<_>>();
    assert_eq!(
        sparse_alltags_property_titles,
        ["Deep task", "Direct control", "Perspective"]
    );

    let mut builder = AgendaWorkspaceBuilder::new();
    builder.add_document("tag-groups.org", &doc);
    let workspace = builder.finish(&AgendaWorkspaceQuery::new().command(
        "gtd",
        AgendaWorkspaceCommandKind::Match(AgendaWorkspaceMatchCommand::new(
            AgendaMatchQuery::parse("+GTD").expect("valid workspace group match"),
        )),
    ));
    let workspace_titles = workspace.commands[0]
        .cards
        .iter()
        .map(|card| card.title.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        workspace_titles,
        ["Deep task", "Direct control", "Perspective"]
    );

    let clock_plans = doc.clock_table_plans();
    let clock_rows = clock_plans[0]
        .rows
        .iter()
        .map(|row| row.title.as_str())
        .collect::<Vec<_>>();
    assert_eq!(clock_rows, ["Deep task", "Direct control", "Perspective"]);

    insta::assert_debug_snapshot!(
        "semantic_ast__tag_vocabulary_group_match_expansion",
        (
            agenda_titles,
            excluded_titles,
            alltags_property_titles,
            tags_property_titles,
            not_gtd_property_titles,
            sparse_titles,
            sparse_alltags_property_titles,
            workspace_titles,
            clock_rows
        )
    );
}
