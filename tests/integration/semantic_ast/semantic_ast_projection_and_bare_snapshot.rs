use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{ElementData, MarkupKind, ObjectData, TodoState},
};

#[test]
fn semantic_ast_projects_headline_title_objects_from_the_aot_graph() {
    let doc = Org::parse("* TODO *Bold* title :work:\n").document();
    let section = &doc.sections[0];
    assert_eq!(section.raw_title, "*Bold* title ");
    assert!(section.title.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));
}

#[test]
fn semantic_ast_projects_rich_keyword_objects_from_the_aot_graph() {
    let doc = Org::parse("#+TITLE: *Demo* Doc\n").document();
    let title = doc
        .metadata
        .iter()
        .find(|keyword| keyword.key == "TITLE")
        .expect("TITLE keyword");
    assert!(title.parsed.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));
}

#[test]
fn semantic_ast_projects_attribute_tokens_from_the_scheme_aot_graph() {
    let doc = Org::parse("#+ATTR_HTML: :class compact :width \"10 em\"\nText\n").document();
    let paragraph = doc
        .children
        .iter()
        .find(|element| matches!(element.data, ElementData::Paragraph(_)))
        .expect("paragraph following ATTR_HTML");
    let attributes = &paragraph.affiliated_keywords[0].attributes;
    assert_eq!(attributes.len(), 2);
    assert_eq!(attributes[0].key, "class");
    assert_eq!(attributes[0].value.as_deref(), Some("compact"));
    assert_eq!(attributes[0].raw, ":class compact");
    assert_eq!(attributes[1].key, "width");
    assert_eq!(attributes[1].value.as_deref(), Some("10 em"));
    assert_eq!(attributes[1].raw, ":width \"10 em\"");
}

#[test]
fn semantic_ast_inherits_headline_tags_from_the_aot_graph() {
    let doc = Org::parse("* Parent :work:\n** Child :urgent:\n").document();
    let parent = &doc.sections[0];
    let child = &parent.subsections[0];
    assert_eq!(parent.tags, ["work"]);
    assert_eq!(child.tags, ["urgent"]);
    assert_eq!(child.effective_tags, ["work", "urgent"]);
}

#[test]
fn semantic_ast_projects_planning_timestamp_fields_from_scheme_objects() {
    let doc = Org::parse("* Work\nSCHEDULED: <2026-05-15 Fri 10:00-11:00 ++1w -2d>\n").document();
    let scheduled = doc.sections[0]
        .planning
        .scheduled
        .as_ref()
        .expect("Scheme-classified planning timestamp");
    assert_eq!(
        scheduled
            .start
            .as_ref()
            .map(|moment| (moment.year, moment.month, moment.day)),
        Some((2026, 5, 15))
    );
    assert_eq!(
        scheduled.start.as_ref().and_then(|moment| moment.hour),
        Some(10)
    );
    assert_eq!(
        scheduled.end.as_ref().and_then(|moment| moment.hour),
        Some(11)
    );
    assert!(scheduled.repeater.is_some());
    assert!(scheduled.warning.is_some());

    let range = Org::parse("* Range\nSCHEDULED: <2026-05-15 Fri>-<2026-05-16 Sat>\n").document();
    let scheduled_range = range.sections[0]
        .planning
        .scheduled
        .as_ref()
        .expect("Scheme-classified single-hyphen range");
    assert!(scheduled_range.is_range);
    assert_eq!(scheduled_range.end.as_ref().map(|end| end.day), Some(16));
}

#[test]
fn semantic_ast_projects_clock_timestamp_and_duration_from_scheme_graph() {
    let doc = Org::parse("* Work\nCLOCK: [2026-05-15 Fri 10:00] => 1:02\n").document();
    let clock = doc.sections[0]
        .children
        .iter()
        .find_map(|element| match &element.data {
            orgize::ast::ElementData::Clock(clock) => Some(clock),
            _ => None,
        })
        .expect("Scheme-classified clock element");
    assert_eq!(clock.duration.as_deref(), Some("1:02"));
    assert_eq!(
        clock
            .parsed_duration
            .as_ref()
            .map(|value| value.total_seconds),
        Some(3720)
    );
    assert_eq!(
        clock
            .value
            .as_ref()
            .and_then(|timestamp| timestamp.start.as_ref())
            .map(|start| start.hour),
        Some(Some(10))
    );
}

#[test]
fn semantic_ast_inherits_graph_properties_with_child_override() {
    let doc = Org::parse(
        "#+PROPERTY: Effort 2h\n* Parent\n:PROPERTIES:\n:Owner: Sarah\n:END:\n** Child\n:PROPERTIES:\n:Owner: Bob\n:END:\n",
    )
    .document();
    let child = &doc.sections[0].subsections[0];
    assert_eq!(child.properties.len(), 1);
    assert!(
        child
            .effective_properties
            .iter()
            .any(|property| { property.key == "Effort" && property.value == "2h" })
    );
    assert!(
        child
            .effective_properties
            .iter()
            .any(|property| { property.key == "Owner" && property.value == "Bob" })
    );
    assert!(
        !child
            .effective_properties
            .iter()
            .any(|property| { property.key == "Owner" && property.value == "Sarah" })
    );
}

#[test]
fn semantic_ast_projects_document_targets_from_scheme_graph_records() {
    let doc = Org::parse(
        "* Anchor Heading\n:PROPERTIES:\n:CUSTOM_ID: local\n:ID: global\n:END:\n<<named>> <<<radio>>>\n[fn:note] body\n",
    )
    .document();
    assert_eq!(doc.sections[0].anchor.as_deref(), Some("local"));
    let keys = doc
        .targets
        .iter()
        .map(|target| target.key.as_str())
        .collect::<Vec<_>>();
    for key in [
        "Anchor Heading",
        "#local",
        "id:global",
        "named",
        "radio",
        "fn:note",
    ] {
        assert!(keys.contains(&key), "missing AOT graph target {key}");
    }
}

#[test]
fn semantic_ast_projection_and_bare_snapshot() {
    let doc = Org::parse(
        r#"#+TITLE: Demo
* TODO Heading :work:
SCHEDULED: <2026-04-30 Thu>
:PROPERTIES:
:CUSTOM_ID: heading-id
:END:
Paragraph with *bold*, [[https://example.com][a link]], and <2026-04-30 Thu>.

- [X] item one
- tag :: item two

#+begin_src rust
fn main() {}
#+end_src
"#,
    )
    .document();

    assert_clean_projection(&doc);
    assert_eq!(doc.children.len(), 1);
    assert_eq!(doc.sections.len(), 1);

    let section = &doc.sections[0];
    assert_eq!(section.level, 1);
    assert_eq!(section.todo.as_ref().unwrap().state, TodoState::Todo);
    assert_eq!(section.raw_title, "Heading ");
    assert_eq!(section.tags, ["work"]);
    assert_eq!(section.anchor.as_deref(), Some("heading-id"));
    let scheduled = section.planning.scheduled.as_ref().unwrap();
    assert_eq!(scheduled.start.as_ref().unwrap().year, 2026);
    assert_eq!(scheduled.start.as_ref().unwrap().month, 4);
    assert_eq!(scheduled.start.as_ref().unwrap().day, 30);
    assert_eq!(
        scheduled.start.as_ref().unwrap().day_name.as_deref(),
        Some("Thu")
    );

    let paragraph = section
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::Paragraph(objects) => Some(objects),
            _ => None,
        })
        .expect("paragraph element");
    assert!(paragraph.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));
    assert!(
        paragraph
            .iter()
            .any(|object| matches!(object.data, ObjectData::Link(_)))
    );

    insta::with_settings!({snapshot_path => "../../snapshots", prepend_module_to_snapshot => false}, {
        insta::assert_debug_snapshot!("semantic_ast__semantic_bare_ast", doc.to_bare());
    });
}
