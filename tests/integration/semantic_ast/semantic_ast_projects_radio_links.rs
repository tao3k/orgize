use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{ElementData, LinkTarget, MarkupKind, ObjectData, TargetKind},
    config::RadioLinkProjection,
    Org, ParseConfig,
};

#[test]
fn semantic_ast_projects_radio_links_from_plain_text() {
    let doc = Org::parse("<<<Radio Target>>> links Radio Target, not Radio Targets.").document();

    assert_clean_projection(&doc);
    let objects = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects,
        other => panic!("expected paragraph, got {other:#?}"),
    };

    assert!(objects.iter().any(
        |object| matches!(&object.data, ObjectData::RadioTarget(value) if value == "Radio Target")
    ));

    let links = objects
        .iter()
        .filter_map(|object| match &object.data {
            ObjectData::Link(link) => Some((object, link)),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(links.len(), 1);
    let (object, link) = links[0];
    assert_eq!(object.ann.raw, "Radio Target");
    assert_eq!(link.path(), "Radio Target");
    assert!(matches!(
        &link.target,
        LinkTarget::Internal(target) if target == "Radio Target"
    ));
    assert!(link.has_description());
    assert_eq!(link.raw_description, "Radio Target");
    assert_eq!(link.description.len(), 1);
    assert!(matches!(
        &link.description[0].data,
        ObjectData::Plain(value) if value == "Radio Target"
    ));

    let plain = objects
        .iter()
        .filter_map(|object| match &object.data {
            ObjectData::Plain(value) => Some(value.as_str()),
            _ => None,
        })
        .collect::<String>();
    assert!(plain.contains("Radio Targets"));
}

#[test]
fn semantic_ast_projects_opt_in_radio_links_across_parsed_objects() {
    let doc = ParseConfig {
        radio_link_projection: RadioLinkProjection::Semantic,
        ..Default::default()
    }
    .parse(r#"<<<*Radio*>>> See *Radio* and \alpha, not Radio."#)
    .document();

    assert_clean_projection(&doc);
    let objects = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects,
        other => panic!("expected paragraph, got {other:#?}"),
    };

    let links = objects
        .iter()
        .filter_map(|object| match &object.data {
            ObjectData::Link(link) => Some((object.ann.raw.as_str(), link)),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(links.len(), 1);
    let (raw, link) = links[0];
    assert_eq!(raw, "*Radio*");
    assert_eq!(link.path(), "*Radio*");
    assert_eq!(link.raw_description, "*Radio*");
    assert!(matches!(
        &link.target,
        LinkTarget::Internal(target) if target == "*Radio*"
    ));
    assert!(matches!(
        &link.description[0].data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    ));

    assert!(objects.iter().any(|object| {
        matches!(
            &object.data,
            ObjectData::Plain(value) if value.contains("not Radio")
        )
    }));
}

#[test]
fn semantic_ast_deduplicates_radio_targets_for_projection() {
    let doc = Org::parse("<<<Alpha>>> <<<Alpha>>>\nAlpha Alphabet Alpha").document();

    assert_clean_projection(&doc);
    assert_eq!(
        doc.targets
            .iter()
            .filter(|target| target.kind == TargetKind::RadioTarget)
            .count(),
        2
    );

    let links = doc
        .children
        .iter()
        .filter_map(|element| match &element.data {
            ElementData::Paragraph(objects) => Some(objects),
            _ => None,
        })
        .flatten()
        .filter_map(|object| match &object.data {
            ObjectData::Link(link) => Some(link),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(links.len(), 2);
    assert!(links.iter().all(|link| matches!(
        &link.target,
        LinkTarget::Internal(target) if target == "Alpha"
    )));
}

#[test]
fn semantic_ast_prefers_longest_radio_target_at_same_position() {
    let doc = Org::parse("<<<Alpha>>> <<<Alpha Beta>>>\nAlpha Beta Alpha").document();

    assert_clean_projection(&doc);
    let links = doc
        .children
        .iter()
        .filter_map(|element| match &element.data {
            ElementData::Paragraph(objects) => Some(objects),
            _ => None,
        })
        .flatten()
        .filter_map(|object| match &object.data {
            ObjectData::Link(link) => Some(link),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(links.len(), 2);
    assert!(matches!(
        &links[0].target,
        LinkTarget::Internal(target) if target == "Alpha Beta"
    ));
    assert!(matches!(
        &links[1].target,
        LinkTarget::Internal(target) if target == "Alpha"
    ));
}

#[test]
fn semantic_ast_projects_multiple_radio_links_across_object_run_slices() {
    let doc = ParseConfig {
        radio_link_projection: RadioLinkProjection::Semantic,
        ..Default::default()
    }
    .parse("<<<*One*>>> <<<~Two~>>>\nBefore *One* middle ~Two~ after")
    .document();

    assert_clean_projection(&doc);
    let links = doc
        .children
        .iter()
        .filter_map(|element| match &element.data {
            ElementData::Paragraph(objects) => Some(objects),
            _ => None,
        })
        .flatten()
        .filter_map(|object| match &object.data {
            ObjectData::Link(link) => Some(link),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(links.len(), 2);
    assert!(matches!(
        &links[0].description[0].data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    ));
    assert!(matches!(
        &links[1].description[0].data,
        ObjectData::Code(value) if value == "Two"
    ));
}
