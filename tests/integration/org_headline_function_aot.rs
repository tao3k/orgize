//! The Scheme-authored Org TODO functions must compile and behave in Rust.

include!("../../languages/org/v1/modules/org-elements/generated/todo_directive.rs");
include!("../../languages/org/v1/modules/org-elements/generated/todo_state_from_directives.rs");
include!("../../languages/org/v1/modules/org-elements/generated/todo_keyword_from_directives.rs");
include!("../../languages/org/v1/modules/org-elements/generated/headline_content_after_todo.rs");

#[test]
fn scheme_headline_ir_compiles_to_rust_function() {
    let ir = include_str!(
        "../../languages/org/v1/modules/org-elements/generated/headline_content_after_todo.ir.json"
    );
    gerbil_scheme_rust_ir::compile_function_json(ir).unwrap();
}

macro_rules! check_todo_state_aot {
    ($($title:expr, $directives:expr => $expected:expr),+ $(,)?) => {
        $(
            let source: &[&str] = $directives;
            let directives: Vec<String> = source.iter().copied().map(String::from).collect();
            assert_eq!(todo_state_from_directives($title, &directives), $expected,
                       "title: {:?}, directives: {:?}", $title, directives);
        )+
    };
}

#[test]
fn scheme_todo_state_aot_uses_document_directives() {
    check_todo_state_aot!(
        "TODO Work", &[] => "todo",
        "DONE Work", &[] => "done",
        "WAIT Work", &[] => "",
        "WAIT Work", &["WAIT(w) | DONE(d)"] => "todo",
        "DONE Work", &["WAIT(w) | DONE(d)"] => "done",
        "TODO Work", &["WAIT(w) | DONE(d)"] => "",
        "HOLD Work", &["WAIT(w) | DONE(d)", "HOLD(h) | FINISHED(f)"] => "todo",
        "FINISHED Work", &["WAIT(w) | DONE(d)", "HOLD(h) | FINISHED(f)"] => "done",
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

#[test]
fn scheme_todo_keyword_value_aot_uses_file_local_declarations() {
    let directives = vec!["WAIT(w) | DONE(d)".to_string()];
    assert_eq!(
        todo_keyword_from_directives("WAIT Review", &directives),
        "WAIT"
    );
    assert_eq!(todo_keyword_from_directives("TODO prose", &directives), "");
    assert_eq!(
        todo_keyword_from_directives("DONE Child", &directives),
        "DONE"
    );
}

#[test]
fn scheme_headline_content_aot_preserves_remaining_text() {
    let directives = vec!["WAIT(w) | DONE(d)".to_string()];
    macro_rules! check_content {
        ($($title:expr => $expected:expr),+ $(,)?) => {
            $(assert_eq!(headline_content_after_todo($title, &directives), $expected);)+
        };
    }
    check_content!(
        "  WAIT   [#A] Parent :work:  " => "[#A] Parent :work:",
        "WAIT" => "",
        "TODO is ordinary text" => "TODO is ordinary text",
        "DONE\tévidence  :研究:" => "évidence  :研究:",
    );
}
