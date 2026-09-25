//! The Scheme-authored Org TODO functions must compile and behave in Rust.

include!(concat!(env!("OUT_DIR"), "/todo_directive_p.rs"));
include!(concat!(env!("OUT_DIR"), "/todo_state_from_directives.rs"));
include!(concat!(env!("OUT_DIR"), "/todo_keyword_from_directives.rs"));
include!(concat!(env!("OUT_DIR"), "/headline_content_after_todo.rs"));
include!(concat!(env!("OUT_DIR"), "/headline_display_title.rs"));
include!(concat!(env!("OUT_DIR"), "/org_image_link_p.rs"));

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

#[test]
fn scheme_headline_display_title_aot_projects_decorations() {
    macro_rules! check_display_title {
        ($($content:expr => $expected:expr),+ $(,)?) => {
            $(
                assert_eq!(headline_display_title($content), $expected,
                           "headline content: {:?}", $content);
            )+
        };
    }
    check_display_title!(
        "[#A] Parent :work:urgent:" => "Parent",
        "Child :work:" => "Child",
        "TODO is ordinary text" => "TODO is ordinary text",
        "Task :work:sdd:" => "Task",
        "Task" => "Task",
    );
}

#[test]
fn scheme_org_image_link_aot_classifies_targets() {
    macro_rules! check_image_target {
        ($($target:expr => $expected:expr),+ $(,)?) => {
            $(assert_eq!(org_image_link_p($target), $expected, "target: {:?}", $target);)+
        };
    }
    check_image_target!(
        "diagram.svg" => true,
        "photo.jpeg" => true,
        "diagram.svg?size=2" => false,
        "notes.org" => false,
    );
}
