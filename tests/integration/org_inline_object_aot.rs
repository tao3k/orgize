//! Scheme-owned inline Object events through generated Rust and Rowan.

macro_rules! check_org_aot_latex_fragments {
    ($source:expr => [$($value:expr),* $(,)?]) => {{
        let document = orgize::org_aot::parse_org_aot($source)
            .expect("Scheme LaTeX fragments build a lossless Rowan document");
        assert_eq!(document.syntax().to_string(), $source);
        let values: Vec<_> = document.records().iter()
            .filter(|record| record.kind == "latex-fragment")
            .map(|record| record.field("value"))
            .collect();
        assert_eq!(values, [$(Some($value)),*]);
    }};
}

#[test]
fn org_scheme_event_aot_projects_latex_math_fragments() {
    check_org_aot_latex_fragments!(
        "\\(x\\) \\[y\\] $$z$$ $w$\n"
        => ["\\(x\\)", "\\[y\\]", "$$z$$", "$w$"]
    );
    check_org_aot_latex_fragments!("$ x$ $x $ \\(open\n" => []);
    let source = "$unfinished [[id:x]]\n";
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("unclosed LaTeX candidate leaves later Objects parseable");
    assert_eq!(document.syntax().to_string(), source);
    assert!(
        document
            .records()
            .iter()
            .any(|record| record.kind == "link")
    );
}

#[test]
fn org_scheme_event_aot_projects_inline_code_and_verbatim_values() {
    let source = "a ~code~ =verb= [[id:x]] z\n";
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("Scheme inline Object strategy builds a lossless Rowan document");
    assert_eq!(document.syntax().to_string(), source);
    let code = document
        .records()
        .iter()
        .find(|record| record.kind == "code")
        .expect("code is a typed Object");
    assert_eq!(code.field("value"), Some("code"));
    let verbatim = document
        .records()
        .iter()
        .find(|record| record.kind == "verbatim")
        .expect("verbatim is a typed Object");
    assert_eq!(verbatim.field("value"), Some("verb"));
    assert!(
        document
            .records()
            .iter()
            .any(|record| record.kind == "link")
    );

    let negative = orgize::org_aot::parse_org_aot("x~y~ ~unclosed\n")
        .expect("invalid and unclosed markup remains source text");
    assert_eq!(negative.syntax().to_string(), "x~y~ ~unclosed\n");
    assert!(
        !negative
            .records()
            .iter()
            .any(|record| record.kind == "code" || record.kind == "verbatim")
    );
}

#[test]
fn org_scheme_event_aot_projects_target_and_radio_target_values() {
    let source = "a <<one two>> and <<<radio>>> z\n";
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("Scheme target Objects build a lossless Rowan document");
    assert_eq!(document.syntax().to_string(), source);
    let target = document
        .records()
        .iter()
        .find(|record| record.kind == "target")
        .expect("target is a typed Object");
    assert_eq!(target.field("value"), Some("one two"));
    let radio = document
        .records()
        .iter()
        .find(|record| record.kind == "radio-target")
        .expect("radio-target is a typed Object");
    assert_eq!(radio.field("value"), Some("radio"));

    let invalid = orgize::org_aot::parse_org_aot("x << bad>> and <<bad >>\n")
        .expect("invalid target borders remain lossless text");
    assert_eq!(invalid.syntax().to_string(), "x << bad>> and <<bad >>\n");
    assert!(
        !invalid
            .records()
            .iter()
            .any(|record| record.kind == "target" || record.kind == "radio-target")
    );
}

#[test]
fn org_scheme_event_aot_projects_statistics_cookies_with_exact_values() {
    let source = "a [50%] [2/3] [%] [/] z\n";
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("Scheme statistics cookies build a lossless Rowan document");
    assert_eq!(document.syntax().to_string(), source);
    let values: Vec<_> = document
        .records()
        .iter()
        .filter(|record| record.kind == "statistics-cookie")
        .map(|record| record.field("value"))
        .collect();
    assert_eq!(
        values,
        [Some("[50%]"), Some("[2/3]"), Some("[%]"), Some("[/]")]
    );

    let invalid = orgize::org_aot::parse_org_aot("[5] [5/a] [5%%] x\n")
        .expect("malformed statistics cookies remain source text");
    assert_eq!(invalid.syntax().to_string(), "[5] [5/a] [5%%] x\n");
    assert!(
        !invalid
            .records()
            .iter()
            .any(|record| record.kind == "statistics-cookie")
    );
}

#[test]
fn org_scheme_event_aot_projects_only_physical_line_end_breaks() {
    let source = "a\\\\  \r\nnext\n";
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("Scheme line-break strategy builds a lossless Rowan document");
    assert_eq!(document.syntax().to_string(), source);
    let line_break = document
        .records()
        .iter()
        .find(|record| record.kind == "line-break")
        .expect("physical end-of-line break is a typed Object");
    assert_eq!(line_break.range.start(), 1u32.into());
    assert_eq!(line_break.range.end(), 7u32.into());

    let invalid = orgize::org_aot::parse_org_aot("a\\\\ x\na\\\\\\\n")
        .expect("nonterminal and escaped pairs remain source text");
    assert_eq!(invalid.syntax().to_string(), "a\\\\ x\na\\\\\\\n");
    assert!(
        !invalid
            .records()
            .iter()
            .any(|record| record.kind == "line-break")
    );
}

#[test]
fn org_scheme_context_algorithm_aot_projects_inline_link_objects() {
    let source = "go [[https://a][α]] and [[id:b]]\n[[broken\n- [[file:x][item]]\n";
    let parsed = orgize::org_aot::parse_org_aot(source)
        .expect("Scheme inline links build a lossless Rowan tree");
    assert_eq!(parsed.syntax().to_string(), source);
    let links: Vec<_> = parsed
        .records()
        .iter()
        .filter(|record| record.kind == "link")
        .map(|record| (record.field("path"), record.field("description")))
        .collect();
    assert_eq!(
        links,
        [
            (Some("https://a"), Some("α")),
            (Some("id:b"), None),
            (Some("file:x"), Some("item"))
        ]
    );
}
