//! Semantic citation parsing helpers.

pub(super) fn citation_style(head: &str) -> (String, String) {
    let Some(rest) = head.strip_prefix("cite/") else {
        return ("nil".into(), String::new());
    };
    let mut parts = rest.split('/');
    let style = parts.next().unwrap_or("nil").to_string();
    let variant = parts.collect::<Vec<_>>().join("/");
    (style, variant)
}

pub(super) fn citation_key_range(reference: &str) -> Option<(usize, usize)> {
    let at = reference.find('@')?;
    let start = at + 1;
    let end = reference[start..]
        .find(char::is_whitespace)
        .map(|position| start + position)
        .unwrap_or(reference.len());

    (start < end).then_some((start, end))
}
