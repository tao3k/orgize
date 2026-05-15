use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{AgendaDate, AgendaQuery},
    Org,
};

#[test]
fn semantic_ast_projects_priority_effort_and_effective_properties() {
    let doc = Org::parse(
        r#"#+PROPERTY: Effort 2h
* TODO [#A] Parent
:PROPERTIES:
:Owner: Sarah
:END:
** TODO Child
SCHEDULED: <2026-05-15 Fri>
* TODO [#2] Numeric
"#,
    )
    .document();

    assert_clean_projection(&doc);
    assert_eq!(doc.properties[0].key, "Effort");
    assert_eq!(
        doc.properties[0].duration.as_ref().unwrap().total_seconds,
        7_200
    );

    let parent = &doc.sections[0];
    assert_eq!(parent.priority.raw_cookie(), Some("A"));
    assert_eq!(parent.priority.effective_text(), "A");
    assert!(parent
        .effective_properties
        .iter()
        .any(|property| property.key == "Effort" && property.value == "2h"));

    let child = &parent.subsections[0];
    assert!(child.priority.is_default());
    assert_eq!(child.priority.effective_text(), "B");
    assert!(child
        .effective_properties
        .iter()
        .any(|property| property.key == "Owner" && property.value == "Sarah"));

    let numeric = &doc.sections[1];
    assert_eq!(numeric.priority.raw_cookie(), Some("2"));
    assert_eq!(numeric.priority.effective_text(), "2");

    let query = AgendaQuery::single_day(AgendaDate::new(2026, 5, 15))
        .match_expression(r#"Owner="Sarah"+Effort>=120+PRIORITY="B""#)
        .expect("valid inherited property match");
    let entries = doc.to_bare().agenda_entries(&query);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].raw_title, "Child");

    insta::with_settings!({snapshot_path => "../../snapshots", prepend_module_to_snapshot => false}, {
        insta::assert_debug_snapshot!("semantic_ast__semantic_priority_properties", doc.to_bare());
    });
}

#[test]
fn semantic_ast_projects_clock_duration_metadata() {
    let doc = Org::parse("* Work\nCLOCK: [2003-09-16 Tue 09:39] =>  1:02\n").document();
    assert_clean_projection(&doc);

    let clock = doc.sections[0]
        .children
        .iter()
        .find_map(|element| match &element.data {
            orgize::ast::ElementData::Clock(clock) => Some(clock),
            _ => None,
        })
        .expect("clock element");

    assert_eq!(clock.duration.as_deref(), Some("1:02"));
    assert_eq!(clock.parsed_duration.as_ref().unwrap().total_seconds, 3_720);
}
