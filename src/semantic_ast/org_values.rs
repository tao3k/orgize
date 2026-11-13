//! Typed admission of the native property/lifecycle/metadata value owner.
pub(super) fn rows(name: &str, fields: &[&str]) -> Vec<Vec<String>> {
    let mut request = vec![name];
    request.extend_from_slice(fields);
    crate::org_aot::native_semantic_rows(23, &request).expect("native value owner")
}
pub(super) fn optional(name: &str, value: &str) -> Option<Vec<String>> {
    optional_fields(name, &[value])
}
pub(super) fn optional_fields(name: &str, fields: &[&str]) -> Option<Vec<String>> {
    let mut values = rows(name, fields);
    assert!(values.len() <= 1, "native optional row count");
    values.pop()
}
pub(super) fn scalar(name: &str, fields: &[&str]) -> String {
    let mut values = rows(name, fields);
    assert_eq!(values.len(), 1, "native scalar row count");
    let mut row = values.pop().unwrap();
    assert_eq!(row.len(), 1, "native scalar arity");
    row.pop().unwrap()
}
pub(super) fn words(value: &str) -> Vec<String> {
    let mut values = rows("words", &[value]);
    assert_eq!(values.len(), 1, "native words row count");
    values.pop().unwrap()
}
