//! Production-only typed admission of native Scheme values.

fn scalar(name: &str, fields: &[&str]) -> String {
    let mut request = Vec::with_capacity(fields.len() + 1);
    request.push(name);
    request.extend_from_slice(fields);
    let mut rows =
        super::native_semantic_rows(16, &request).expect("initialized native value owner");
    assert_eq!(rows.len(), 1, "native value row count");
    let mut row = rows.pop().unwrap();
    assert_eq!(row.len(), 1, "native scalar count");
    row.pop().unwrap()
}

fn label(name: &str, fields: &[&str]) -> &'static str {
    match scalar(name, fields).as_str() {
        "" => "",
        "scheduled" => "scheduled",
        "deadline" => "deadline",
        "closed" => "closed",
        "archived" => "archived",
        "current" => "current",
        "background" => "background",
        "headline" => "headline",
        "custom-id" => "custom-id",
        "id" => "id",
        "footnote" => "footnote",
        "code-ref" => "code-ref",
        "fuzzy" => "fuzzy",
        "uri" => "uri",
        _ => panic!("native category"),
    }
}

pub(super) fn planning_key_kind(key: &str) -> &'static str {
    label("planning-key-kind", &[key])
}

pub(super) fn headline_anchor_slug(title: &str) -> String {
    scalar("headline-anchor-slug", &[title])
}

pub(super) fn memory_headline_state(
    todo_type: &str,
    closed: bool,
    planned: bool,
    archived: bool,
) -> &'static str {
    label(
        "memory-headline-state",
        &[
            todo_type,
            &closed.to_string(),
            &planned.to_string(),
            &archived.to_string(),
        ],
    )
}

pub(super) fn org_image_link_p(target: &str) -> bool {
    match scalar("org-image-link?", &[target]).as_str() {
        "true" => true,
        "false" => false,
        _ => panic!("native boolean"),
    }
}

pub(super) fn org_link_kind(path: &str) -> &'static str {
    label("org-link-kind", &[path])
}

pub(super) fn org_link_protocol(path: &str) -> String {
    scalar("org-link-protocol", &[path])
}

pub(super) fn org_link_protocol_path(path: &str) -> String {
    scalar("org-link-protocol-path", &[path])
}

pub(super) fn org_expand_link_abbreviation(
    replacement: &str,
    path: &str,
    encoded_path: &str,
) -> String {
    scalar(
        "org-expand-link-abbreviation",
        &[replacement, path, encoded_path],
    )
}
