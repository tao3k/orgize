//! Scheme-owned inline source blocks and Babel calls in the Rowan Element graph.

macro_rules! check_org_inline_code {
    ($record:expr, $source:expr, $kind:expr, $literal:expr, {$($name:literal => $value:expr),+ $(,)?}) => {{
        let record = $record;
        let range = record.range;
        assert_eq!(record.kind, $kind);
        assert_eq!(
            &$source[usize::from(range.start())..usize::from(range.end())],
            $literal
        );
        $(assert_eq!(record.field($name), $value);)+
    }};
}

#[test]
fn scheme_inline_code_projects_balanced_fields_and_exact_source() {
    let source = "src_rust[:exports code]{a{b}c} call_task[x](a(b))[z]\n";
    let document = orgize::org_aot::parse_org_aot(source).expect("Scheme inline code parses");
    assert_eq!(document.syntax().to_string(), source);
    let records = document.records();
    let source_block = records
        .iter()
        .find(|record| record.kind == "inline-src-block")
        .expect("inline source block");
    let babel_call = records
        .iter()
        .find(|record| record.kind == "inline-babel-call")
        .expect("inline Babel call");
    check_org_inline_code!(source_block, source, "inline-src-block", "src_rust[:exports code]{a{b}c}", {
        "language" => Some("rust"),
        "parameters" => Some(":exports code"),
        "value" => Some("a{b}c")
    });
    check_org_inline_code!(babel_call, source, "inline-babel-call", "call_task[x](a(b))[z]", {
        "call" => Some("task"),
        "inside-header" => Some("x"),
        "arguments" => Some("a(b)"),
        "end-header" => Some("z")
    });
}

#[test]
fn scheme_inline_code_preserves_empty_body_and_recovers_incomplete_forms() {
    let source = "src_go{} call_run()\nsrc_rust{unfinished\ncall_bad[x](unfinished\n";
    let document = orgize::org_aot::parse_org_aot(source).expect("incomplete inline code parses");
    assert_eq!(document.syntax().to_string(), source);
    let records = document.records();
    assert_eq!(
        records
            .iter()
            .filter(|record| record.kind == "inline-src-block")
            .count(),
        1
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.kind == "inline-babel-call")
            .count(),
        1
    );
    let source_block = records
        .iter()
        .find(|record| record.kind == "inline-src-block")
        .expect("empty inline source block");
    check_org_inline_code!(source_block, source, "inline-src-block", "src_go{}", {
        "language" => Some("go"),
        "parameters" => None,
        "value" => Some("")
    });
    let call = records
        .iter()
        .find(|record| record.kind == "inline-babel-call")
        .expect("empty inline Babel call");
    check_org_inline_code!(call, source, "inline-babel-call", "call_run()", {
        "call" => Some("run"),
        "inside-header" => None,
        "arguments" => Some(""),
        "end-header" => None
    });
}
