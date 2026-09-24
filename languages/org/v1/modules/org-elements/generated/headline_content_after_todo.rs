pub fn headline_content_after_todo(title: &str, directives: &[String]) -> String {
    if todo_keyword_from_directives(title, directives).is_empty() {
        title.trim().to_owned()
    } else {
        title
            .trim()
            .split_once(char::is_whitespace)
            .unwrap_or_default()
            .1
            .trim()
            .to_owned()
    }
}
