use orgize::Org;

fn named_source_block_template_is_ast_owned_and_fail_closed() {
    let source = r#"
#+NAME: call
#+BEGIN_SRC codex-collaboration
collaboration.list_agents({ path_prefix: "{{{ROOT}}}" })
#+END_SRC
"#;
    let document = Org::parse(source).document();
    assert_eq!(
        document
            .render_named_source_block_template("call", "codex-collaboration", [("ROOT", "/root")],)
            .unwrap(),
        "collaboration.list_agents({ path_prefix: \"/root\" })"
    );
    assert!(
        document
            .render_named_source_block_template("call", "text", [("ROOT", "/root")])
            .is_err()
    );
    assert!(
        document
            .render_named_source_block_template("call", "codex-collaboration", [])
            .is_err()
    );
}

fn duplicate_named_source_block_is_rejected() {
    let source = r#"
#+NAME: call
#+BEGIN_SRC text
one
#+END_SRC
#+NAME: call
#+BEGIN_SRC text
two
#+END_SRC
"#;
    let document = Org::parse(source).document();
    assert!(
        document
            .render_named_source_block_template("call", "text", [])
            .is_err()
    );
}

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[
    (
        "named_source_block_template::named_source_block_template_is_ast_owned_and_fail_closed",
        named_source_block_template_is_ast_owned_and_fail_closed,
    ),
    (
        "named_source_block_template::duplicate_named_source_block_is_rejected",
        duplicate_named_source_block_is_rejected,
    ),
];
