use orgize::{Org, export::MarkdownExportOptions};

#[test]
fn explicit_startup_precedes_parallel_markdown_cases() {
    // SAFETY: standalone fixture initializes before application workers.
    unsafe { orgize::initialize_native_runtime() }.expect("native exporter startup");
    let cases: &[(&str, fn())] = &[
        (
            "markdown_export_renders_core_document_shapes",
            markdown_export_renders_core_document_shapes,
        ),
        (
            "markdown_export_renders_blocks_tables_and_markdown_exports",
            markdown_export_renders_blocks_tables_and_markdown_exports,
        ),
        (
            "markdown_export_renders_properties_as_key_value_table",
            markdown_export_renders_properties_as_key_value_table,
        ),
        #[cfg(feature = "md")]
        (
            "markdown_export_properties_parse_as_gfm_table",
            markdown_export_properties_parse_as_gfm_table,
        ),
        (
            "markdown_export_can_render_subtrees",
            markdown_export_can_render_subtrees,
        ),
        (
            "markdown_footnote_reference_and_definition_share_a_label",
            markdown_footnote_reference_and_definition_share_a_label,
        ),
        (
            "markdown_export_options_control_special_strings_and_entities",
            markdown_export_options_control_special_strings_and_entities,
        ),
    ];
    std::thread::scope(|scope| {
        for &(name, case) in cases {
            scope.spawn(move || {
                case();
                println!("native-export case={name} OK");
            });
        }
    });
    println!(
        "startup-native exporter=markdown cases={} complete OK",
        cases.len()
    );
}

fn markdown_export_renders_core_document_shapes() {
    insta::assert_snapshot!(
        Org::parse(
            r#"
* Title
Paragraph with *bold*, /italic/, =verbatim=, ~code~, \alpha{}, and <2026-05-11 Mon>.

Visit [[https://example.com?a=1&b=2][Example]] and [[file:plot.png]].

Hard line\\
break.

#+begin_quote
Quoted
text.
#+end_quote

+ first
+ second
"#
        )
        .to_markdown()
    );
}

fn markdown_export_renders_blocks_tables_and_markdown_exports() {
    insta::assert_snapshot!(
        Org::parse(
            r#"
#+begin_src rust
fn main() {
    println!("hello");
}
#+end_src

#+begin_example
,* escaped headline
#+end_example

#+begin_export markdown
**raw markdown**
#+end_export

@@md:inline markdown@@ and @@html:<span>ignored</span>@@.

| Name | Count |
|------+-------|
| one  |     1 |
| two  |     2 |

| Plain | Table |
| no    | rule  |
"#
        )
        .to_markdown()
    );
}

fn markdown_export_renders_properties_as_key_value_table() {
    let rendered = Org::parse(
        r#"
* Task
:PROPERTIES:
:CUSTOM_ID: task-1
:Effort: 1:00
:OWNER: tao|bar
:END:

Body.
"#,
    )
    .to_markdown();

    assert!(
        rendered.contains(
            "| Key | Value |\n\
             | --- | --- |\n\
             | CUSTOM_ID | task-1 |\n\
             | Effort | 1:00 |\n\
             | OWNER | tao\\|bar |"
        ),
        "{rendered}"
    );
    assert!(!rendered.contains(":PROPERTIES:"), "{rendered}");
}

#[cfg(feature = "md")]
fn markdown_export_properties_parse_as_gfm_table() {
    let rendered = Org::parse(
        r#"
* Task
:PROPERTIES:
:CUSTOM_ID: task-1
:Effort: 1:00
:END:
"#,
    )
    .to_markdown();

    let arena = comrak::Arena::new();
    let mut options = comrak::Options::default();
    options.extension.table = true;
    let _ = comrak::parse_document(&arena, &rendered, &options);
}

fn markdown_export_can_render_subtrees() {
    let org = Org::parse("* /hello/ *world*");
    let bold = org
        .records()
        .iter()
        .position(|record| record.kind == "bold")
        .expect("Scheme AOT graph should contain the bold Object");
    assert_eq!(org.try_markdown_record(bold).unwrap(), "**world**");
}

fn markdown_footnote_reference_and_definition_share_a_label() {
    insta::assert_snapshot!(Org::parse("A [fn:bench].\n\n[fn:bench] Note.\n").to_markdown(), @r###"
A [^bench].

[^bench]: Note.
"###);
}

fn markdown_export_options_control_special_strings_and_entities() {
    let org = Org::parse(r#"a -- b --- c... don't \- \alpha{}"#);
    let rendered = org.to_markdown_with_options(MarkdownExportOptions {
        special_strings: true,
        expand_entities: false,
    });

    assert!(rendered.contains('\u{2013}'));
    assert!(rendered.contains('\u{2014}'));
    assert!(rendered.contains('\u{2026}'));
    assert!(rendered.contains("don\u{2019}t"));
    assert!(rendered.contains("\\alpha{}"));
    assert!(org.to_markdown().contains('α'));
}
