pub fn todo_keyword_from_directives(title: &str, directives: &[String]) -> String {
    if todo_state_from_directives(title, directives).is_empty() { String::new() } else { title.split_whitespace().next().unwrap_or("").to_owned() }
}
