//! UTF-8 bounded admission of the native Scheme radio matching plan.

pub(super) fn matches(source: &str, targets: &[String]) -> Vec<(usize, usize, usize)> {
    let mut fields = vec![source];
    fields.extend(targets.iter().map(String::as_str));
    crate::org_aot::native_semantic_rows(22, &fields)
        .expect("initialized native radio owner")
        .into_iter()
        .map(|row| {
            let [start, end, index]: [String; 3] = row.try_into().expect("native radio arity");
            let start: usize = start.parse().expect("native radio start");
            let end: usize = end.parse().expect("native radio end");
            let index: usize = index.parse().expect("native radio target");
            assert!(
                start < end
                    && end <= source.len()
                    && source.is_char_boundary(start)
                    && source.is_char_boundary(end)
                    && index < targets.len(),
                "native radio bounds"
            );
            (start, end, index)
        })
        .collect()
}
