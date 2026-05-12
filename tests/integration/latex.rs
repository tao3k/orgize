use orgize::{
    export::{LatexExport, LatexExportOptions},
    Org,
};
use rowan::ast::AstNode;

#[test]
fn latex_export_escapes_text_and_renders_inline_markup() {
    insta::assert_snapshot!(Org::parse(
        "* Heading & 100%\nText _ # $ & with *bold*, /em/, _under_, =verb=, ~code~, x^2, y_1, and \\\\alpha.\n"
    )
    .to_latex());
}

#[test]
fn latex_export_renders_structural_blocks_lists_tables_and_links() {
    insta::assert_snapshot!(Org::parse(
        r#"
* Export
Visit [[https://example.com?a=1&b=2][Example & Docs]] and [[file:plot.png]].

#+begin_quote
Quoted *text*.
#+end_quote

#+begin_src rust
fn main() {
    println!("hello_{}", 1);
}
#+end_src

+ plain item
+ second item

| Name | Count |
|------+-------|
| one  |     1 |
| two  |     2 |
"#
    )
    .to_latex());
}

#[test]
fn latex_export_preserves_latex_specific_input() {
    insta::assert_snapshot!(Org::parse(
        r#"
Inline $a_b$ and @@latex:\LaTeX{}@@ plus \alpha{}.

#+begin_export latex
\begin{equation}
e^{i\pi}+1=0
\end{equation}
#+end_export

\begin{align}
a &= b + c
\end{align}

See [cite:@doe2026; @roe2026 p. 42] on <2026-05-10 Sun>.
"#
    )
    .to_latex());
}

#[test]
fn latex_export_can_render_subtrees() {
    let org = Org::parse("* /hello/ *world*");
    let bold = org.first_node::<orgize::syntax_ast::Bold>().unwrap();
    let mut latex = LatexExport::default();
    latex.render(bold.syntax());
    assert_eq!(latex.finish(), r"\textbf{world}");
}

#[test]
fn latex_export_options_control_special_strings_and_entities() {
    let org = Org::parse(r#"a -- b --- c... don't \- \alpha{}"#);
    let rendered = org.to_latex_with_options(LatexExportOptions {
        special_strings: true,
        expand_entities: false,
    });

    assert!(rendered.contains('\u{2013}'));
    assert!(rendered.contains('\u{2014}'));
    assert!(rendered.contains('\u{2026}'));
    assert!(rendered.contains("don\u{2019}t"));
    assert!(rendered.contains(r"\textbackslash{}alpha\{\}"));
    assert!(org.to_latex().contains(r"\alpha"));
}
