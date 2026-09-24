pub fn todo_keyword_matches_p(title: &str, directives: &[String], expected: &str) -> bool {
    let state = todo_state_from_directives(title, directives);
    let candidate = title.split_whitespace().next().unwrap_or("");
    ((state == "todo") || (state == "done")) && (candidate == expected)
}
