//! The Scheme-authored Org TODO functions must compile and behave in Rust.

include!("../../languages/org/v1/modules/org-elements/generated/todo_name.rs");
include!("../../languages/org/v1/modules/org-elements/generated/todo_directive.rs");

macro_rules! check_todo_name_aot {
    ($($input:expr => $expected:expr),+ $(,)?) => {
        $(assert_eq!(todo_name($input), $expected, "input: {:?}", $input);)+
    };
}

#[test]
fn scheme_todo_name_aot_matches_org_declarations() {
    check_todo_name_aot!(
        "TODO(t)" => "TODO",
        "WAIT(w)" => "WAIT",
        "DONE" => "DONE",
        "(log)" => "",
        "" => "",
    );
}

#[test]
fn scheme_todo_directive_aot_is_case_insensitive() {
    macro_rules! check_todo_directive_aot {
        ($($key:expr => $expected:expr),+ $(,)?) => {
            $(assert_eq!(todo_directive_p($key), $expected, "key: {:?}", $key);)+
        };
    }
    check_todo_directive_aot!(
        "TODO" => true,
        "seq_todo" => true,
        "Typ_Todo" => true,
        "ＴＯＤＯ" => false,
        "TITLE" => false,
        "" => false,
    );
}
