//! Shared structural assertions for Scheme-AOT Org Elements tests.

macro_rules! check_org_aot_element {
    ($source:expr, $kind:expr, $field:expr => $value:expr) => {{
        let document = orgize::org_aot::parse_org_aot($source)
            .expect("Scheme-owned Org Element parser accepts the source");
        let elements: Vec<_> = document
            .records()
            .iter()
            .filter(|record| record.kind == $kind)
            .collect();
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].field($field), Some($value));
        assert_eq!(document.syntax().to_string(), $source);
    }};
}
