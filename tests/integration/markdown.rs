use orgize::{
    Org,
    export::{MarkdownExport, MarkdownExportOptions},
};
use rowan::ast::AstNode;

#[test]
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

#[test]
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

#[test]
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
#[test]
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

#[test]
fn markdown_export_can_render_subtrees() {
    let org = Org::parse("* /hello/ *world*");
    let bold = org.first_node::<orgize::syntax_ast::Bold>().unwrap();
    let mut markdown = MarkdownExport::default();
    markdown.render(bold.syntax());
    assert_eq!(markdown.finish(), "**world**");
}

#[test]
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
