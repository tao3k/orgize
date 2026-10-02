use orgize::{Org, export::LatexExportOptions};

#[test]
fn latex_export_escapes_text_and_renders_inline_markup() {
    insta::assert_snapshot!(Org::parse(
        "* Heading & 100%\nText _ # $ & with *bold*, /em/, _under_, =verb=, ~code~, x^2, y_1, and \\\\alpha.\n"
    )
    .to_latex());
}

#[test]
fn latex_export_renders_structural_blocks_lists_tables_and_links() {
    insta::assert_snapshot!(
        Org::parse(
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
        .to_latex()
    );
}

#[test]
fn latex_table_formula_is_metadata_not_a_row() {
    let org = Org::parse("| Name | Value |\n|------+-------|\n| alpha | 1 |\n#+TBLFM: @2$2=1\n");
    let table = org
        .records()
        .iter()
        .find(|record| record.kind == "table")
        .expect("Scheme must classify the table");
    insta::assert_snapshot!(org.try_latex_record(table.id).unwrap(), @r"
\begin{tabular}{ll}
Name & Value \\
\hline
alpha & 1 \\
\end{tabular}
");
}

#[test]
fn latex_footnote_definition_preserves_its_label_and_body() {
    let org = Org::parse("A [fn:bench].\n\n[fn:bench] Note.\n");
    let definition = org
        .records()
        .iter()
        .find(|record| record.kind == "footnote-definition")
        .expect("Scheme must classify the footnote definition");
    insta::assert_snapshot!(org.try_latex_record(definition.id).unwrap(), @r###"
\begin{quote}\textsuperscript{bench} Note.

\end{quote}
"###);
}

#[test]
fn latex_export_preserves_latex_specific_input() {
    insta::assert_snapshot!(
        Org::parse(
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
        .to_latex()
    );
}

#[test]
fn latex_export_can_render_subtrees() {
    let org = Org::parse("* /hello/ *world*");
    let bold = org
        .records()
        .iter()
        .find(|record| record.kind == "bold")
        .unwrap();
    assert_eq!(org.try_latex_record(bold.id).unwrap(), r"\textbf{world}");
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
