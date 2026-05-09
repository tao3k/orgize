use crate::semantic_ast::support::assert_clean_projection;
use orgize::Org;

#[test]
fn semantic_ast_covers_current_lossless_projection_surface() {
    let fixtures = [
        "#+TITLE: Demo\n",
        r#"* TODO Heading :tag:
DEADLINE: <2026-05-01 Fri> SCHEDULED: <2026-04-30 Thu> CLOSED: [2026-04-29 Wed]
:PROPERTIES:
:CUSTOM_ID: id
:END:
Body.
"#,
        r#"Paragraph with *bold* /italic/ _underline_ +strike+ ~code~ =verbatim= H_2 x^2 <2026-04-30 Thu> [2026-04-30 Thu] <%%(diary-date 4 30)> @@html:<span>@@ \alpha $x$ <<target>> <<<radio>>> {{{macro(1\,a, two)}}} [fn:note:See /inner/] [cite:@doe2020] src_rust[:exports code]{let x = 1;} call_square(4) [50%]\\
Next.
"#,
        r#"#+ATTR_HTML: :class compact
| A | B |
|---+---|
| 1 | 2 |
#+TBLFM: $1=$2
"#,
        r#"#+begin_quote
quoted
#+end_quote

#+begin_src rust
fn main() {}
#+end_src

#+begin_export html
<b>x</b>
#+end_export
"#,
        r#":DRAWER:
inside
:END:

[fn:note] Footnote body

# comment
: fixed
-----
\begin{equation}
x
\end{equation}
"#,
        "- [X] item\n- term :: description\n",
        "  +---+\n  | a |\n  +---+\n",
    ];

    for fixture in fixtures {
        let doc = Org::parse(fixture).document();
        assert_clean_projection(&doc);
    }
}
