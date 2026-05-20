use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{ElementData, ObjectData, RepeaterKind, TimeUnit, WarningKind},
};

#[test]
fn semantic_ast_projects_timestamp_metadata() {
    let doc = Org::parse("SCHEDULED: <2003-09-16 Tue 09:39-10:39 +1w --2d>\n").document();

    assert_clean_projection(&doc);
    let timestamp = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects
            .iter()
            .find_map(|object| match &object.data {
                ObjectData::Timestamp(timestamp) => Some(timestamp),
                _ => None,
            })
            .expect("timestamp object"),
        other => panic!("expected paragraph, got {other:#?}"),
    };

    let start = timestamp.start.as_ref().expect("timestamp start");
    assert_eq!(start.year, 2003);
    assert_eq!(start.month, 9);
    assert_eq!(start.day, 16);
    assert_eq!(start.day_name.as_deref(), Some("Tue"));
    assert_eq!(start.hour, Some(9));
    assert_eq!(start.minute, Some(39));

    let end = timestamp.end.as_ref().expect("timestamp range end");
    assert_eq!(end.year, 2003);
    assert_eq!(end.month, 9);
    assert_eq!(end.day, 16);
    assert_eq!(end.day_name.as_deref(), Some("Tue"));
    assert_eq!(end.hour, Some(10));
    assert_eq!(end.minute, Some(39));

    let repeater = timestamp.repeater.as_ref().expect("timestamp repeater");
    assert_eq!(repeater.kind, RepeaterKind::Cumulate);
    assert_eq!(repeater.value, 1);
    assert_eq!(repeater.unit, TimeUnit::Week);

    let warning = timestamp.warning.as_ref().expect("timestamp warning");
    assert_eq!(warning.kind, WarningKind::First);
    assert_eq!(warning.value, 2);
    assert_eq!(warning.unit, TimeUnit::Day);
}
