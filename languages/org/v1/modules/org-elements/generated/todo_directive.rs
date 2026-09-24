pub fn todo_directive_p(key: &str) -> bool {
    key.eq_ignore_ascii_case("TODO") || key.eq_ignore_ascii_case("SEQ_TODO") || key.eq_ignore_ascii_case("TYP_TODO")
}
