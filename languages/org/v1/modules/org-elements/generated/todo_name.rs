pub fn todo_name(token: &str) -> &str {
    token.split_once("(").map_or(token, |(head, _)| head)
}
