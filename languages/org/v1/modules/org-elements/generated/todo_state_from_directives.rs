pub fn todo_state_from_directives(title: &str, directives: &[String]) -> &'static str {
    let candidate = title.split_whitespace().next().unwrap_or("");
    if candidate.is_empty() { "" } else if directives.is_empty() { if candidate == "TODO" { "todo" } else if candidate == "DONE" { "done" } else { "" } } else if directives.iter().map(String::as_str).any(|directive| {
    let open_side = directive.split_once("|").map_or(directive, |(head, _)| head);
    open_side.split_whitespace().any(|word| candidate == word.split_once("(").map_or(word, |(head, _)| head))
}) { "todo" } else if directives.iter().map(String::as_str).any(|directive| {
    let done_side = directive.split_once("|").map_or("", |(_, tail)| tail);
    done_side.split_whitespace().any(|word| candidate == word.split_once("(").map_or(word, |(head, _)| head))
}) { "done" } else { "" }
}
