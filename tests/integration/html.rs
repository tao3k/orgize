use orgize::{Org, export::HtmlExportOptions};

#[test]
fn headline_anchor_callback_uses_scheme_projected_title() {
    let rendered = Org::parse("* A & B\n")
        .try_to_html_with_headline_anchor(|title| format!("section-{title}"))
        .expect("graph-backed HTML");
    assert_eq!(
        rendered,
        "<main><h1><a id=\"section-A &amp; B\" href=\"#section-A &amp; B\">A &amp; B</a></h1></main>"
    );
}

#[test]
fn emphasis() {
    insta::assert_snapshot!(
        Org::parse("*bold*, /italic/,\n_underlined_, =verbatim= and ~code~").to_html(),
        @r###"
    <main><section><p><b>bold</b>, <i>italic</i>,
    <u>underlined</u>, <code>verbatim</code> and <code>code</code></p></section></main>
    "###
    );
}

#[test]
fn link() {
    insta::assert_snapshot!(
        Org::parse("Visit[[http://example.com][link1]]or[[http://example.com][link1]].").to_html(),
        @r###"<main><section><p>Visit<a href="http://example.com">link1</a>or<a href="http://example.com">link1</a>.</p></section></main>"###
    );

    insta::assert_snapshot!(
        Org::parse("Visit <https://example.com/path>.").to_html(),
        @r###"<main><section><p>Visit <a href="https://example.com/path">https://example.com/path</a>.</p></section></main>"###
    );

    insta::assert_snapshot!(
        Org::parse("Visit https://example.com/path.").to_html(),
        @r###"<main><section><p>Visit <a href="https://example.com/path">https://example.com/path</a>.</p></section></main>"###
    );
}

#[test]
fn section_and_headline() {
    insta::assert_snapshot!(
        Org::parse(r#"
* title 1
section 1
** title 2
section 2
* title 3
section 3
* title 4
section 4
"#).to_html(),
        @r###"
    <main><h1>title 1</h1><section><p>section 1
    </p></section><h2>title 2</h2><section><p>section 2
    </p></section><h1>title 3</h1><section><p>section 3
    </p></section><h1>title 4</h1><section><p>section 4
    </p></section></main>
    "###
    );
}

#[test]
fn list() {
    insta::assert_snapshot!(
        Org::parse(r#"
+ 1

+ 2

  - 3

  - 4

+ 5
"#).to_html(),
        @r###"
    <main><section><ul><li><p>1
    </p></li><li><p>2
    </p><ul><li><p>3
    </p></li><li><p>4
    </p></li></ul></li><li><p>5
    </p></li></ul></section></main>
    "###
    );
}

#[test]
fn snippet() {
    insta::assert_snapshot!(
        Org::parse("@@html:<del>@@delete this@@html:</del>@@").to_html(),
        @"<main><section><p><del>delete this</del></p></section></main>"
    );
}

#[test]
fn html_export_block() {
    let rendered = Org::parse(
        r#"
#+begin_export html
<div class="videoWrapper"><iframe src="https://www.youtube.com/embed/vb1-lHR7kRM"></iframe></div>
#+end_export

#+begin_export latex
\LaTeX{}
#+end_export
"#,
    )
    .to_html();

    assert!(rendered.contains(r#"<div class="videoWrapper"><iframe src="https://www.youtube.com/embed/vb1-lHR7kRM"></iframe></div>"#));
    assert!(!rendered.contains(r#"\LaTeX{}"#));
}

#[test]
fn html_source_block_preserves_language_and_safe_data_attributes() {
    let rendered = Org::parse(
        r#"
#+name: browser-contribution
#+attr_html: :data-poo-flow browser-contribution :onclick alert(1) :style display:none
#+begin_src scheme
(display 1)
#+end_src
"#,
    )
    .to_html();

    assert!(rendered.contains(
        r#"<pre class="src src-scheme" data-poo-flow="browser-contribution"><code class="language-scheme">"#,
    ));
    assert!(!rendered.contains("onclick="));
    assert!(!rendered.contains("style="));
}

#[test]
fn paragraphs() {
    insta::assert_snapshot!(
        Org::parse(r#"
* title

paragraph 1

paragraph 2

paragraph 3

paragraph 4
"#).to_html(),
        @r###"
    <main><h1>title</h1><section><p>paragraph 1
    </p><p>paragraph 2
    </p><p>paragraph 3
    </p><p>paragraph 4
    </p></section></main>
    "###
    );
}

#[test]
fn table() {
    // don't has table header
    insta::assert_snapshot!(
        Org::parse(r#"
|-----+-----+-----|
|   0 |   1 |   2 |
|   4 |   5 |   6 |
|-----+-----+-----|
"#).to_html(),
        @"<main><section><table><tbody><tr><td>0</td><td>1</td><td>2</td></tr><tr><td>4</td><td>5</td><td>6</td></tr></tbody></table></section></main>"
    );

    // has table header
    insta::assert_snapshot!(
        Org::parse(r#"
|   0 |   1 |   2 |
|-----+-----+-----|
|   4 |   5 |   6 |
|-----+-----+-----|
"#).to_html(),
        @"<main><section><table><thead><tr><td>0</td><td>1</td><td>2</td></tr></thead><tbody><tr><td>4</td><td>5</td><td>6</td></tr></tbody></table></section></main>"
    );

    // has two table body
    insta::assert_snapshot!(
        Org::parse(r#"
|   0 |   1 |   2 |
|-----+-----+-----|
|   4 |   5 |   6 |
|-----+-----+-----|
|   7 |   8 |   9 |
"#).to_html(),
        @"<main><section><table><thead><tr><td>0</td><td>1</td><td>2</td></tr></thead><tbody><tr><td>4</td><td>5</td><td>6</td></tr></tbody><tbody><tr><td>7</td><td>8</td><td>9</td></tr></tbody></table></section></main>"
    );

    // multiple row rule
    insta::assert_snapshot!(
        Org::parse(r#"
|   0 |   1 |   2 |
|-----+-----+-----|
|-----+-----+-----|
|   4 |   5 |   6 |
"#).to_html(),
        @"<main><section><table><thead><tr><td>0</td><td>1</td><td>2</td></tr></thead><tbody><tr><td>4</td><td>5</td><td>6</td></tr></tbody></table></section></main>"
    );

    // empty
    insta::assert_snapshot!(
        Org::parse(r#"
|-----+-----+-----|
|-----+-----+-----|
"#).to_html(),
        @"<main><section><table></table></section></main>"
    );

    insta::assert_snapshot!(
        Org::parse(r#"
|
|-
|
|-
|
"#).to_html(),
        @"<main><section><table><thead><tr></tr></thead><tbody><tr></tr></tbody><tbody><tr></tr></tbody></table></section></main>"
    );
}

#[test]
fn table_formula_is_metadata_not_html_row() {
    insta::assert_snapshot!(
        Org::parse("| Name | Value |\n|------+-------|\n| alpha | 1 |\n#+TBLFM: @2$2=1\n").to_html(),
        @"<main><section><table><thead><tr><td>Name</td><td>Value</td></tr></thead><tbody><tr><td>alpha</td><td>1</td></tr></tbody></table></section></main>"
    );
}

#[test]
fn inline_source_uses_scheme_classified_language_and_escaped_body() {
    insta::assert_snapshot!(
        Org::parse("before src_rust{let x = 1 < 2;} after").to_html(),
        @r#"<main><section><p>before <code class="src src-rust">let x = 1 &lt; 2;</code> after</p></section></main>"#
    );
}

#[test]
fn inline_babel_call_is_escaped_source_text() {
    insta::assert_snapshot!(
        Org::parse("call_square(1 < 2)").to_html(),
        @"<main><section><p>call_square(1 &lt; 2)</p></section></main>"
    );
}

#[test]
fn footnote_reference_and_definition_share_an_anchor() {
    insta::assert_snapshot!(
        Org::parse("A [fn:bench].\n\n[fn:bench] Note.\n").to_html(),
        @r###"
<main><section><p>A <sup class="footnote-reference"><a href="#fn-bench">bench</a></sup>.
</p><aside class="footnote" id="fn-bench"><p> Note.
</p></aside></section></main>
"###
    );
}

#[test]
fn line_break() {
    insta::assert_debug_snapshot!(
        Org::parse("aa\\\\\nbb").to_html(),
        @r###""<main><section><p>aa<br/>bb</p></section></main>""###
    );
}

#[test]
fn html_export_options_control_special_strings_and_entities() {
    let org = Org::parse(r#"a -- b --- c... don't \- \alpha{}"#);
    let rendered = org.to_html_with_options(HtmlExportOptions {
        special_strings: true,
        expand_entities: false,
    });

    assert!(rendered.contains('\u{2013}'));
    assert!(rendered.contains('\u{2014}'));
    assert!(rendered.contains('\u{2026}'));
    assert!(rendered.contains("don\u{2019}t"));
    assert!(rendered.contains("\\alpha{}"));
    assert!(org.to_html().contains("&alpha;"));
}
