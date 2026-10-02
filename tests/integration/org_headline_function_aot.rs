//! The Scheme-authored Org TODO functions must compile and behave in Rust.

include!(concat!(env!("OUT_DIR"), "/todo_directive_p.rs"));
include!(concat!(env!("OUT_DIR"), "/todo_state_from_directives.rs"));
include!(concat!(env!("OUT_DIR"), "/todo_keyword_from_directives.rs"));
include!(concat!(env!("OUT_DIR"), "/headline_content_after_todo.rs"));
include!(concat!(env!("OUT_DIR"), "/headline_source_title.rs"));
include!(concat!(env!("OUT_DIR"), "/planning_key_kind.rs"));
include!(concat!(env!("OUT_DIR"), "/priority_token_p.rs"));
include!(concat!(env!("OUT_DIR"), "/headline_display_title.rs"));
include!(concat!(env!("OUT_DIR"), "/headline_comment_p.rs"));
include!(concat!(env!("OUT_DIR"), "/memory_headline_state.rs"));
include!(concat!(env!("OUT_DIR"), "/org_image_link_p.rs"));
include!(concat!(env!("OUT_DIR"), "/org_link_kind.rs"));
include!(concat!(env!("OUT_DIR"), "/org_link_target_key.rs"));
include!(concat!(env!("OUT_DIR"), "/org_link_protocol.rs"));
include!(concat!(env!("OUT_DIR"), "/org_link_protocol_path.rs"));
include!(concat!(env!("OUT_DIR"), "/org_link_file_path.rs"));
include!(concat!(env!("OUT_DIR"), "/org_link_search.rs"));
include!(concat!(env!("OUT_DIR"), "/org_link_file_path_kind.rs"));
include!(concat!(env!("OUT_DIR"), "/org_link_search_kind.rs"));
include!(concat!(env!("OUT_DIR"), "/org_link_search_value.rs"));

macro_rules! check_todo_state_aot {
    ($($title:expr, $directives:expr => $expected:expr),+ $(,)?) => {
        let configured_todo = vec!["TODO".to_string()];
        let configured_done = vec!["DONE".to_string()];
        $(
            let source: &[&str] = $directives;
            let directives: Vec<String> = source.iter().copied().map(String::from).collect();
            assert_eq!(todo_state_from_directives(
                           $title, &directives, &configured_todo, &configured_done), $expected,
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
fn scheme_todo_state_aot_uses_config_only_without_file_directives() {
    let configured_todo = vec!["WAIT".to_string()];
    let configured_done = vec!["FINISHED".to_string()];
    assert_eq!(
        todo_state_from_directives("WAIT Work", &[], &configured_todo, &configured_done),
        "todo"
    );
    assert_eq!(
        todo_state_from_directives("FINISHED Work", &[], &configured_todo, &configured_done),
        "done"
    );
    assert_eq!(
        todo_state_from_directives(
            "WAIT Work",
            &["HOLD | DONE".to_string()],
            &configured_todo,
            &configured_done,
        ),
        ""
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
    let configured_todo = vec!["TODO".to_string()];
    let configured_done = vec!["DONE".to_string()];
    assert_eq!(
        todo_keyword_from_directives(
            "WAIT Review",
            &directives,
            &configured_todo,
            &configured_done
        ),
        "WAIT"
    );
    assert_eq!(
        todo_keyword_from_directives(
            "TODO prose",
            &directives,
            &configured_todo,
            &configured_done
        ),
        ""
    );
    assert_eq!(
        todo_keyword_from_directives(
            "DONE Child",
            &directives,
            &configured_todo,
            &configured_done
        ),
        "DONE"
    );
}

#[test]
fn scheme_headline_content_aot_preserves_remaining_text() {
    let directives = vec!["WAIT(w) | DONE(d)".to_string()];
    macro_rules! check_content {
        ($($title:expr => $expected:expr),+ $(,)?) => {
            let configured_todo = vec!["TODO".to_string()];
            let configured_done = vec!["DONE".to_string()];
            $(assert_eq!(headline_content_after_todo(
                $title, &directives, &configured_todo, &configured_done), $expected);)+
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
fn scheme_source_title_aot_preserves_tag_boundary_whitespace() {
    macro_rules! check_source_title_aot {
        ($($body:expr, $todo:expr => $expected:expr),+ $(,)?) => {
            $(assert_eq!(headline_source_title($body, $todo), $expected);)+
        };
    }
    check_source_title_aot!(
        "TODO [#A] *Inline* task ", "TODO" => "*Inline* task ",
        "  Plain  ", "" => "Plain  ",
        "DONE\t[#2] Review", "DONE" => "Review",
    );
}

#[test]
fn scheme_planning_aot_classifies_declared_keys() {
    macro_rules! check_planning_aot {
        ($function:ident; $($input:expr => $expected:expr),+ $(,)?) => {
            $(assert_eq!($function($input), $expected, "input: {:?}", $input);)+
        };
    }
    check_planning_aot!(planning_key_kind;
        "scheduled" => "scheduled",
        "DEADLINE" => "deadline",
        "Closed" => "closed",
        "CLOCK" => "",
    );
}

#[test]
fn scheme_headline_display_title_aot_projects_decorations() {
    macro_rules! check_priority_token {
        ($($token:expr => $expected:expr),+ $(,)?) => {
            $(assert_eq!(priority_token_p($token), $expected, "priority token: {:?}", $token);)+
        };
    }
    check_priority_token!(
        "[#A]" => true,
        "[#064]" => true,
        "[#65]" => false,
        "[#+1]" => false,
        "[#a]" => false,
        "[#É]" => false,
        "[#A]junk]" => false,
    );
    macro_rules! check_display_title {
        ($($content:expr, $has_tags:expr => $expected:expr),+ $(,)?) => {
            $(
                assert_eq!(headline_display_title($content, $has_tags), $expected,
                           "headline content: {:?}", $content);
            )+
        };
    }
    check_display_title!(
        "[#A] Parent :work:urgent:", true => "Parent",
        "Child :work:", true => "Child",
        "TODO is ordinary text", false => "TODO is ordinary text",
        "Task :work:sdd:", true => "Task",
        "Task", false => "Task",
        "Plan :bad::", false => "Plan :bad::",
        "[#AB] Plan", false => "[#AB] Plan",
        "[#65] Plan", false => "[#65] Plan",
    );
}

#[test]
fn scheme_memory_headline_state_aot_classifies_admitted_elements() {
    macro_rules! check_state {
        ($($todo:expr, $closed:expr, $planned:expr, $archived:expr => $expected:expr),+ $(,)?) => {
            $(assert_eq!(memory_headline_state($todo, $closed, $planned, $archived), $expected);)+
        };
    }
    check_state!(
        "todo", false, false, false => "current",
        "done", false, false, false => "closed",
        "", true, false, false => "closed",
        "", false, true, false => "current",
        "", false, false, false => "background",
        "todo", false, false, true => "archived",
    );
}

#[test]
fn scheme_comment_marker_aot_keeps_org_headline_case_rule() {
    macro_rules! check_comment {
        ($($title:expr => $expected:expr),+ $(,)?) => {
            $(assert_eq!(headline_comment_p($title), $expected, "title: {:?}", $title);)+
        };
    }
    check_comment!(
        "COMMENT Hidden" => true,
        "COMMENT" => true,
        "comment Visible" => false,
        "COMMENTARY Visible" => false,
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

#[test]
fn scheme_org_link_path_aot_classifies_internal_and_protocol_forms() {
    assert_eq!(org_link_kind("*Heading"), "headline");
    assert_eq!(org_link_kind("#custom"), "custom-id");
    assert_eq!(org_link_kind("id:local"), "id");
    assert_eq!(org_link_kind("fn:note"), "footnote");
    assert_eq!(org_link_kind("coderef:init"), "code-ref");
    assert_eq!(org_link_kind("target-one"), "fuzzy");
    assert_eq!(org_link_kind("https://example.org"), "uri");
    assert_eq!(org_link_target_key("*Heading"), "Heading");
    assert_eq!(org_link_target_key("id:local::*Heading"), "id:local");
    assert_eq!(org_link_protocol("https://example.org"), "https");
    assert_eq!(
        org_link_protocol_path("https://example.org"),
        "//example.org"
    );
    assert_eq!(
        org_link_file_path("file:notes/demo.org::*Heading"),
        "notes/demo.org"
    );
    assert_eq!(org_link_search("file:notes/demo.org::*Heading"), "*Heading");
    assert_eq!(org_link_file_path_kind("/tmp/demo.org"), "absolute");
    assert_eq!(org_link_file_path_kind("notes/demo.org"), "relative");
    assert_eq!(org_link_search_kind("*Heading"), "headline");
    assert_eq!(org_link_search_kind("255"), "line-number");
    assert_eq!(org_link_search_value("*Heading"), "Heading");
}
