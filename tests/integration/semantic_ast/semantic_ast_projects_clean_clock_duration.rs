use super::*;

#[test]
fn semantic_ast_projects_clean_clock_duration() {
    let doc = Org::parse("* Work\nCLOCK: [2003-09-16 Tue 09:39] =>  1:00\n").document();

    assert!(doc.diagnostics.is_empty());
    let clock = doc.sections[0]
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::Clock(clock) => Some(clock),
            _ => None,
        })
        .expect("clock element");

    assert!(clock.value.is_some());
    let timestamp = clock.value.as_ref().unwrap();
    assert_eq!(timestamp.start.as_ref().unwrap().hour, Some(9));
    assert_eq!(timestamp.start.as_ref().unwrap().minute, Some(39));
    assert_eq!(clock.duration.as_deref(), Some("1:00"));
    assert!(clock.raw.contains("=>  1:00"));
}
