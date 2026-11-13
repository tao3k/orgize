use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{BlockKind, ElementData, MarkupKind, ObjectData, TargetKind, TodoState},
};

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

fn semantic_ast_title_objects_use_aot_spans_when_todo_text_repeats() {
    let doc = Org::parse("* TODO TODO *Bold* TODO :work:\n").document();
    let section = &doc.sections[0];
    assert_eq!(section.raw_title, "TODO *Bold* TODO ");
    let bold = section
        .title
        .iter()
        .find(|object| {
            matches!(
                object.data,
                ObjectData::Markup {
                    kind: MarkupKind::Bold,
                    ..
                }
            )
        })
        .expect("Scheme-owned title markup");
    assert_eq!(u32::from(bold.ann.range.start()), 12);

    let doc = Org::parse("*************** TODO TODO *Bold* TODO :work:\n").document();
    let inlinetask = doc
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::Inlinetask(task) => Some(task),
            _ => None,
        })
        .expect("Scheme-owned inlinetask");
    assert_eq!(inlinetask.raw_title, "TODO *Bold* TODO ");
    let bold = inlinetask
        .title
        .iter()
        .find(|object| {
            matches!(
                object.data,
                ObjectData::Markup {
                    kind: MarkupKind::Bold,
                    ..
                }
            )
        })
        .expect("Scheme-owned inlinetask title markup");
    assert_eq!(u32::from(bold.ann.range.start()), 26);
}

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

fn semantic_ast_property_value_annotations_use_scheme_graph_field_spans() {
    let doc = Org::parse("* H\n:PROPERTIES:\n:EFFORT: tomorrow\n:END:\n").document();
    let property = &doc.sections[0].properties[0];
    assert_eq!(property.key, "EFFORT");
    assert_eq!(property.value, "tomorrow");
    assert_eq!(u32::from(property.ann.range.start()), 26);
    assert_eq!(u32::from(property.ann.range.end()), 34);

    let id_doc = Org::parse("* H\n:PROPERTIES:\n:ID: shared\n:END:\n").document();
    let id_target = id_doc
        .targets
        .iter()
        .find(|target| target.key == "id:shared")
        .expect("ID target");
    assert_eq!(u32::from(id_target.ann.range.start()), 22);
    assert_eq!(u32::from(id_target.ann.range.end()), 28);
}

fn semantic_ast_reports_scheme_classified_missing_internal_link_targets() {
    let doc = Org::parse("[[fn:missing]]\n").document();
    assert!(doc.diagnostics.iter().any(|diagnostic| {
        diagnostic.message == "internal link target `fn:missing` was not found"
    }));
}

fn semantic_ast_projects_section_body_from_aot_child_ranges() {
    let doc = Org::parse("* H\nBody\n").document();
    let body = doc.sections[0]
        .body_ann
        .as_ref()
        .expect("section body from Scheme graph");
    assert_eq!(u32::from(body.range.start()), 4);
    assert_eq!(u32::from(body.range.end()), 9);
}

fn semantic_ast_reuses_scheme_title_objects_for_document_targets() {
    let doc = Org::parse("* TODO Heading :work:\n:PROPERTIES:\n:CUSTOM_ID: heading-id\n:END:\n")
        .document();
    let headline = doc
        .targets
        .iter()
        .find(|target| target.kind == TargetKind::Headline)
        .expect("headline target");
    assert_eq!(headline.raw, "Heading");
    assert!(
        headline.alias.iter().any(
            |object| matches!(&object.data, ObjectData::Plain(text) if text.contains("Heading"))
        )
    );
    let custom = doc
        .targets
        .iter()
        .find(|target| target.kind == TargetKind::CustomId)
        .expect("CUSTOM_ID target");
    assert_eq!(custom.raw, "heading-id");
    assert_eq!(custom.alias, headline.alias);
}

fn semantic_ast_inherits_headline_tags_from_the_aot_graph() {
    let doc = Org::parse("* Parent :work:\n** Child :urgent:\n").document();
    let parent = &doc.sections[0];
    let child = &parent.subsections[0];
    assert_eq!(parent.tags, ["work"]);
    assert_eq!(child.tags, ["urgent"]);
    assert_eq!(child.effective_tags, ["work", "urgent"]);
}

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

    let list = section
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::List(list) => Some(list),
            _ => None,
        })
        .expect("plain list element");
    assert_eq!(
        list.items
            .iter()
            .map(|item| item.bullet.as_str())
            .collect::<Vec<_>>(),
        ["- ", "- "]
    );
    let ElementData::Paragraph(description_body) = &list.items[1].children[0].data else {
        panic!("description list body is a paragraph");
    };
    assert!(matches!(
        &description_body[0].data,
        ObjectData::Plain(value) if value == "item two\n"
    ));
    let source_block = section
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::Block(block) if block.kind == BlockKind::Source => Some(block),
            _ => None,
        })
        .expect("source block element");
    assert_eq!(source_block.name, None, "unnamed block has no NAME");

    insta::with_settings!({snapshot_path => "../../snapshots", prepend_module_to_snapshot => false}, {
        insta::assert_debug_snapshot!("semantic_ast__semantic_bare_ast", doc.to_bare());
    });
}

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[
    (
        "semantic_ast::semantic_ast_projection_and_bare_snapshot::semantic_ast_projects_headline_title_objects_from_the_aot_graph",
        semantic_ast_projects_headline_title_objects_from_the_aot_graph,
    ),
    (
        "semantic_ast::semantic_ast_projection_and_bare_snapshot::semantic_ast_title_objects_use_aot_spans_when_todo_text_repeats",
        semantic_ast_title_objects_use_aot_spans_when_todo_text_repeats,
    ),
    (
        "semantic_ast::semantic_ast_projection_and_bare_snapshot::semantic_ast_projects_rich_keyword_objects_from_the_aot_graph",
        semantic_ast_projects_rich_keyword_objects_from_the_aot_graph,
    ),
    (
        "semantic_ast::semantic_ast_projection_and_bare_snapshot::semantic_ast_projects_attribute_tokens_from_the_scheme_aot_graph",
        semantic_ast_projects_attribute_tokens_from_the_scheme_aot_graph,
    ),
    (
        "semantic_ast::semantic_ast_projection_and_bare_snapshot::semantic_ast_property_value_annotations_use_scheme_graph_field_spans",
        semantic_ast_property_value_annotations_use_scheme_graph_field_spans,
    ),
    (
        "semantic_ast::semantic_ast_projection_and_bare_snapshot::semantic_ast_reports_scheme_classified_missing_internal_link_targets",
        semantic_ast_reports_scheme_classified_missing_internal_link_targets,
    ),
    (
        "semantic_ast::semantic_ast_projection_and_bare_snapshot::semantic_ast_projects_section_body_from_aot_child_ranges",
        semantic_ast_projects_section_body_from_aot_child_ranges,
    ),
    (
        "semantic_ast::semantic_ast_projection_and_bare_snapshot::semantic_ast_reuses_scheme_title_objects_for_document_targets",
        semantic_ast_reuses_scheme_title_objects_for_document_targets,
    ),
    (
        "semantic_ast::semantic_ast_projection_and_bare_snapshot::semantic_ast_inherits_headline_tags_from_the_aot_graph",
        semantic_ast_inherits_headline_tags_from_the_aot_graph,
    ),
    (
        "semantic_ast::semantic_ast_projection_and_bare_snapshot::semantic_ast_projects_planning_timestamp_fields_from_scheme_objects",
        semantic_ast_projects_planning_timestamp_fields_from_scheme_objects,
    ),
    (
        "semantic_ast::semantic_ast_projection_and_bare_snapshot::semantic_ast_projects_clock_timestamp_and_duration_from_scheme_graph",
        semantic_ast_projects_clock_timestamp_and_duration_from_scheme_graph,
    ),
    (
        "semantic_ast::semantic_ast_projection_and_bare_snapshot::semantic_ast_inherits_graph_properties_with_child_override",
        semantic_ast_inherits_graph_properties_with_child_override,
    ),
    (
        "semantic_ast::semantic_ast_projection_and_bare_snapshot::semantic_ast_projects_document_targets_from_scheme_graph_records",
        semantic_ast_projects_document_targets_from_scheme_graph_records,
    ),
    (
        "semantic_ast::semantic_ast_projection_and_bare_snapshot::semantic_ast_projection_and_bare_snapshot",
        semantic_ast_projection_and_bare_snapshot,
    ),
];
