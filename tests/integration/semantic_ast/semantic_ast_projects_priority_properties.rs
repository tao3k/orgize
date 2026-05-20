use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{
        AgendaDate, AgendaQuery, PriorityProfile, PriorityRangeStatus, PriorityValue,
        PropertyAllowedValueScope, PropertyInheritancePolicy,
    },
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
    assert_eq!(parent.priority.org_priority_score(), Some(2_000));
    assert_eq!(parent.priority.range_status(), PriorityRangeStatus::InRange);
    assert!(
        parent
            .effective_properties
            .iter()
            .any(|property| property.key == "Effort" && property.value == "2h")
    );

    let child = &parent.subsections[0];
    assert!(child.priority.is_default());
    assert_eq!(child.priority.effective_text(), "B");
    assert_eq!(child.priority.org_priority_score(), Some(1_000));
    assert_eq!(child.priority.range_status(), PriorityRangeStatus::InRange);
    assert!(
        child
            .effective_properties
            .iter()
            .any(|property| property.key == "Owner" && property.value == "Sarah")
    );

    let numeric = &doc.sections[1];
    assert_eq!(numeric.priority.raw_cookie(), Some("2"));
    assert_eq!(numeric.priority.effective_text(), "2");
    assert_eq!(numeric.priority.org_priority_score(), Some(65_000));
    assert_eq!(
        numeric.priority.range_status(),
        PriorityRangeStatus::OutOfRange
    );

    let numeric_profile = PriorityProfile::new(
        PriorityValue::Numeric(0),
        PriorityValue::Numeric(10),
        PriorityValue::Numeric(5),
    )
    .expect("valid numeric priority profile");
    assert_eq!(
        numeric.priority.score_with_profile(&numeric_profile),
        Some(8_000)
    );
    assert_eq!(
        numeric.priority.range_status_with_profile(&numeric_profile),
        PriorityRangeStatus::InRange
    );

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

#[test]
fn semantic_ast_projects_two_digit_numeric_priority() {
    let doc = Org::parse("* TODO [#64] Numeric high\n").document();
    assert_clean_projection(&doc);

    let section = &doc.sections[0];
    assert_eq!(section.priority.raw_cookie(), Some("64"));
    assert_eq!(section.priority.effective_text(), "64");
    assert_eq!(section.priority.org_priority_score(), Some(3_000));
    assert_eq!(
        section.priority.range_status(),
        PriorityRangeStatus::OutOfRange
    );
}

#[test]
fn semantic_ast_projects_property_profile_allowed_values() {
    let doc = Org::parse(
        r#"#+PROPERTY: Effort_ALL 0 0:30 "1 hour"
#+PROPERTY: Status_all todo done
* Project
:PROPERTIES:
:Owner_ALL: "Sarah Connor" Jim ""
:Owner: Sarah Connor
:END:
** Child
:PROPERTIES:
:Owner: Jim
:END:
"#,
    )
    .document();
    assert_clean_projection(&doc);

    let profile = doc.property_profile();
    assert_eq!(profile.inheritance, PropertyInheritancePolicy::All);
    assert!(
        profile
            .inherited_keys
            .iter()
            .any(|key| key.eq_ignore_ascii_case("Owner"))
    );

    let fixed = profile
        .allowed_values
        .iter()
        .find(|record| record.descriptor_key == "VISIBILITY_ALL")
        .expect("fixed Org visibility values");
    assert_eq!(fixed.scope, PropertyAllowedValueScope::FixedGlobal);
    assert_eq!(fixed.values, ["folded", "children", "content", "all"]);

    let effort = profile
        .allowed_values
        .iter()
        .find(|record| record.descriptor_key == "Effort_ALL")
        .expect("document Effort_ALL values");
    assert_eq!(effort.scope, PropertyAllowedValueScope::Document);
    assert_eq!(effort.property, "Effort");
    assert_eq!(effort.values, ["0", "0:30", "1 hour"]);

    let status = profile
        .allowed_values
        .iter()
        .find(|record| record.descriptor_key == "Status_all")
        .expect("document Status_all values");
    assert_eq!(status.scope, PropertyAllowedValueScope::Document);
    assert_eq!(status.property, "Status");
    assert_eq!(status.values, ["todo", "done"]);

    let owner = profile
        .allowed_values
        .iter()
        .find(|record| record.descriptor_key == "Owner_ALL")
        .expect("section Owner_ALL values");
    assert!(matches!(
        &owner.scope,
        PropertyAllowedValueScope::Section { title, .. } if title == "Project"
    ));
    assert_eq!(owner.values, ["Sarah Connor", "Jim", ""]);
}
