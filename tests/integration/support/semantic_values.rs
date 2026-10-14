//! Test-only probes of the existing native Scheme value ABI.

fn scalar(name: &str, fields: &[&str]) -> String {
    let mut request = Vec::with_capacity(fields.len() + 1);
    request.push(name);
    request.extend_from_slice(fields);
    let mut rows = orgize::c_ffi::project_native_semantic_rows(16, &request)
        .expect("initialized native value owner");
    assert_eq!(rows.len(), 1, "native value row count");
    let mut row = rows.pop().unwrap();
    assert_eq!(row.len(), 1, "native scalar count");
    row.pop().unwrap()
}

fn boolean(name: &str, fields: &[&str]) -> bool {
    match scalar(name, fields).as_str() {
        "true" => true,
        "false" => false,
        _ => panic!("native boolean"),
    }
}

fn todo_scalar(
    name: &str,
    title: &str,
    directives: &[String],
    configured_todo: &[String],
    configured_done: &[String],
) -> String {
    let mut fields = vec![title.to_owned(), directives.len().to_string()];
    fields.extend_from_slice(directives);
    fields.push(configured_todo.len().to_string());
    fields.extend_from_slice(configured_todo);
    fields.push(configured_done.len().to_string());
    fields.extend_from_slice(configured_done);
    scalar(name, &fields.iter().map(String::as_str).collect::<Vec<_>>())
}

pub(super) fn todo_directive_p(key: &str) -> bool {
    boolean("todo-directive?", &[key])
}

pub(super) fn todo_state_from_directives(
    title: &str,
    directives: &[String],
    configured_todo: &[String],
    configured_done: &[String],
) -> String {
    todo_scalar(
        "todo-state-from-directives",
        title,
        directives,
        configured_todo,
        configured_done,
    )
}

pub(super) fn todo_keyword_from_directives(
    title: &str,
    directives: &[String],
    configured_todo: &[String],
    configured_done: &[String],
) -> String {
    todo_scalar(
        "todo-keyword-from-directives",
        title,
        directives,
        configured_todo,
        configured_done,
    )
}

pub(super) fn headline_content_after_todo(
    title: &str,
    directives: &[String],
    configured_todo: &[String],
    configured_done: &[String],
) -> String {
    todo_scalar(
        "headline-content-after-todo",
        title,
        directives,
        configured_todo,
        configured_done,
    )
}

pub(super) fn headline_source_title(title_body: &str, todo_keyword: &str) -> String {
    scalar("headline-source-title", &[title_body, todo_keyword])
}

pub(super) fn planning_key_kind(key: &str) -> String {
    scalar("planning-key-kind", &[key])
}

pub(super) fn priority_token_p(word: &str) -> bool {
    boolean("priority-token?", &[word])
}

pub(super) fn headline_display_title(content: &str, has_tags: bool) -> String {
    scalar("headline-display-title", &[content, &has_tags.to_string()])
}

pub(super) fn headline_comment_p(display_title: &str) -> bool {
    boolean("headline-comment?", &[display_title])
}

pub(super) fn memory_headline_state(
    todo_type: &str,
    closed: bool,
    planned: bool,
    archived: bool,
) -> String {
    scalar(
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
    boolean("org-image-link?", &[target])
}

pub(super) fn org_link_kind(path: &str) -> String {
    scalar("org-link-kind", &[path])
}

pub(super) fn org_link_protocol(path: &str) -> String {
    scalar("org-link-protocol", &[path])
}

pub(super) fn org_link_target_key(path: &str) -> String {
    scalar("org-link-target-key", &[path])
}

pub(super) fn org_link_protocol_path(path: &str) -> String {
    scalar("org-link-protocol-path", &[path])
}

pub(super) fn org_link_file_path(path: &str) -> String {
    scalar("org-link-file-path", &[path])
}

pub(super) fn org_link_search(path: &str) -> String {
    scalar("org-link-search", &[path])
}

pub(super) fn org_link_file_path_kind(path: &str) -> String {
    scalar("org-link-file-path-kind", &[path])
}

pub(super) fn org_link_search_kind(search: &str) -> String {
    scalar("org-link-search-kind", &[search])
}

pub(super) fn org_link_search_value(search: &str) -> String {
    scalar("org-link-search-value", &[search])
}
