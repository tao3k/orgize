use super::PropertyNativePlan;

#[test]
fn property_plan_admission_rejects_malformed_presence_and_shape() {
    let inputs = || vec![("Key".into(), "Value".into())];
    for rows in [
        vec![],
        vec![vec!["key".into()]],
        vec![vec!["key".into(), "maybe".into(), String::new()]],
        vec![vec!["key".into(), "false".into(), "unexpected".into()]],
    ] {
        assert!(std::panic::catch_unwind(|| PropertyNativePlan::admit(inputs(), rows)).is_err());
    }
    let plan = PropertyNativePlan::admit(
        inputs(),
        vec![vec![
            "Key_ALL".into(),
            "false".into(),
            String::new(),
            "two words".into(),
        ]],
    );
    assert!(plan.fact("Key", "Value").descriptor_name.is_none());
    assert_eq!(plan.fact("Key", "Value").tokens, ["two words"]);
}
