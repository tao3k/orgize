//! Native content boundaries shared by tables, macros and keyword headers.

use crate::{Org, ast::ElementData};

pub(super) fn public_macro_expansion_and_property_calls_are_native() {
    for (template, expected) in [
        ("", ""),
        ("literal λ", "literal λ"),
        ("$1/$2/$9", "λ,a/β/"),
        ("$0::$1::$0", "λ,a, β::λ,a::λ,a, β"),
        ("$$/$x/$", "$/$x/$"),
        ("$10 $01 $$$1", "λ,a0 λ,a, β1 $λ,a"),
    ] {
        let source = format!("#+MACRO: x {template}\n{{{{{{x(λ\\,a, β)}}}}}}\n");
        let document = Org::parse(&source).document();
        assert_eq!(
            document.macro_expansions()[0].value.as_deref(),
            Some(expected)
        );
        assert_eq!(
            crate::org_aot::expand_native_macro_fields(8, &[template, "λ,a", "β"]).unwrap(),
            expected
        );
    }
    assert_eq!(
        crate::org_aot::expand_native_macro_fields(8, &["$0/$1"]).unwrap(),
        "/"
    );
    for (source, expected) in [
        ("λ/{{{x(a\\,b,c)}}}/β", "λ/a,b/c/a,b, c/β"),
        ("* {{{x( a , b )}}}\r\nβ", "* a/b/a, b\r\nβ"),
        ("{{{missing(x)}}} {{{x(abc)", "{{{missing(x)}}} {{{x(abc)"),
        ("={{{x(a,b)}}}=", "={{{x(a,b)}}}="),
    ] {
        assert_eq!(
            crate::org_aot::expand_native_macro_fields(9, &[source, "x", "old", "x", "$1/$2/$0"])
                .unwrap(),
            expected
        );
    }
    // Counted UTF-8 fields preserve NUL, rather than treating input as C text.
    assert_eq!(
        crate::org_aot::expand_native_macro_fields(8, &["λ\0$1", "β\0"]).unwrap(),
        "λ\0β\0"
    );
    assert!(crate::org_aot::expand_native_macro_fields(8, &[]).is_err());
    assert!(crate::org_aot::expand_native_macro_fields(9, &["text", "unpaired"]).is_err());
    assert_eq!(
        crate::org_aot::expand_native_macro_fields(8, &["$1", "owner survived"]).unwrap(),
        "owner survived"
    );
    for count in [1_000, 10_000] {
        let template = "$1".repeat(count);
        let mut fields = vec!["x"; count + 1];
        fields[0] = &template;
        assert_eq!(
            crate::org_aot::expand_native_macro_fields(8, &fields).unwrap(),
            "x".repeat(count)
        );
    }
}

pub(super) fn public_table_content_and_formula_extents_are_native() {
    for space in [
        " ", "\t", "\u{85}", "\u{a0}", "\u{1680}", "\u{2000}", "\u{200a}", "\u{2028}", "\u{2029}",
        "\u{202f}", "\u{205f}", "\u{3000}",
    ] {
        let source = format!(
            "|{space}λ{space}|{space}|\r\n#+TBLFM: {space}$1{space}={space}$2{space};{space}N{space}::{space}$3{space}={space}@2{space}\r\n"
        );
        let parsed = Org::parse(&source);
        assert_eq!(parsed.to_org(), source);
        let cells = parsed
            .records()
            .iter()
            .filter(|record| record.kind == "table-cell")
            .collect::<Vec<_>>();
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].field("content"), Some("λ"));
        assert_eq!(cells[1].field("content"), Some(""));
        for record in parsed.records() {
            if let Some(field) = record.fields.iter().find(|field| field.name == "content") {
                assert_eq!(
                    &source[usize::from(field.range.start())..usize::from(field.range.end())],
                    field.value
                );
            }
        }
        let document = parsed.document();
        assert!(
            document.diagnostics.is_empty(),
            "{:?}",
            document.diagnostics
        );
        let table = document
            .children
            .iter()
            .find_map(|element| match &element.data {
                ElementData::Table(table) => Some(table),
                _ => None,
            })
            .expect("native table");
        assert_eq!(table.parsed_formulas.len(), 1);
        let assignments = &table.parsed_formulas[0].assignments;
        assert_eq!(assignments.len(), 2);
        assert_eq!(assignments[0].lhs, "$1");
        assert_eq!(assignments[0].rhs, "$2");
        assert_eq!(assignments[0].flags, ["N"]);
        assert_eq!(assignments[1].lhs, "$3");
        assert_eq!(assignments[1].rhs, "@2");
    }
}

pub(super) fn public_macro_definitions_and_escaped_arguments_are_native() {
    for (args, expected) in [
        ("", vec![]),
        (" , \t, ", vec![]),
        ("42\\,A, Fix", vec!["42,A", "Fix"]),
        ("a\\\\,b", vec!["a\\", "b"]),
        ("a\\x, λ", vec!["a\\x", "λ"]),
        ("a\\", vec!["a\\"]),
        ("\u{3000}λ\u{a0},\u{2003}β\u{202f}", vec!["λ", "β"]),
    ] {
        let source =
            format!("#+MACRO: \u{3000}issue\u{a0}$1 / $2\u{2003}\r\n{{{{{{issue({args})}}}}}}\r\n");
        let parsed = Org::parse(&source);
        assert_eq!(parsed.to_org(), source);
        let keyword = parsed
            .records()
            .iter()
            .find(|record| record.kind == "keyword")
            .unwrap();
        assert_eq!(keyword.field("macro-name"), Some("issue"));
        assert_eq!(keyword.field("macro-template"), Some("$1 / $2\u{2003}"));
        let actual = parsed
            .records()
            .iter()
            .filter(|record| record.kind == "macro-argument")
            .map(|record| record.field("value").unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{source}");
        let document = parsed.document();
        assert!(
            document.diagnostics.is_empty(),
            "{:?}",
            document.diagnostics
        );
        assert_eq!(document.macro_definitions[0].name, "issue");
        assert_eq!(document.macro_definitions[0].template, "$1 / $2\u{2003}");
        let expansions = document.macro_expansions();
        assert_eq!(expansions.len(), 1);
        assert_eq!(expansions[0].arguments, expected);
    }
    for invalid in ["", "1bad template", "bad! template", "λ template"] {
        let source = format!("#+MACRO: {invalid}\n");
        let parsed = Org::parse(&source);
        assert_eq!(parsed.to_org(), source);
        let document = parsed.document();
        assert!(document.macro_definitions.is_empty());
        assert_eq!(document.diagnostics.len(), 1);
    }
}

pub(super) fn public_keyword_and_include_headers_are_decoded_from_content() {
    for (parameters, expected) in [
        (
            ":class compact :width \"10 em\"",
            vec![
                ("class", Some("compact"), ":class compact"),
                ("width", Some("10 em"), ":width \"10 em\""),
            ],
        ),
        (
            ":empty \"\" :missing :width 2 :width 3",
            vec![
                ("empty", Some(""), ":empty \"\""),
                ("missing", None, ":missing"),
                ("width", Some("2"), ":width 2"),
                ("width", Some("3"), ":width 3"),
            ],
        ),
        (
            ":dir \"λ :not-key\" :single 'literal'",
            vec![
                ("dir", Some("λ :not-key"), ":dir \"λ :not-key\""),
                ("single", Some("'literal'"), ":single 'literal'"),
            ],
        ),
        (
            ":dir \"unterminated",
            vec![("dir", Some("\"unterminated"), ":dir \"unterminated")],
        ),
        (
            ":class compact extra :width \"10 em\" tail",
            vec![
                ("class", Some("compact"), ":class compact extra"),
                ("width", Some("10 em"), ":width \"10 em\" tail"),
            ],
        ),
        (
            ":value \"a\\\"b :not-key\" :exports both",
            vec![
                (
                    "value",
                    Some("a\\\"b :not-key"),
                    ":value \"a\\\"b :not-key\"",
                ),
                ("exports", Some("both"), ":exports both"),
            ],
        ),
    ] {
        for prefix in ["#+ATTR_HTML:", "#+INCLUDE: \"./λ file.org\" src org"] {
            let source = format!("{prefix} {parameters} \t\r\nText\r\n");
            let parsed = Org::parse(&source);
            assert_eq!(parsed.to_org(), source);
            let record = parsed
                .records()
                .iter()
                .find(|record| record.kind == "keyword")
                .expect("native keyword");
            let content = record.values("argument-content").collect::<Vec<_>>();
            assert_eq!(
                content,
                expected.iter().filter_map(|arg| arg.1).collect::<Vec<_>>(),
                "{source}"
            );
            for field in record.fields.iter().filter(|field| {
                matches!(
                    field.name,
                    "argument-key" | "argument-content" | "argument-raw"
                )
            }) {
                assert_eq!(
                    &source[usize::from(field.range.start())..usize::from(field.range.end())],
                    field.value
                );
            }
            let document = parsed.document();
            assert!(
                document.diagnostics.is_empty(),
                "{:?}",
                document.diagnostics
            );
            if prefix.starts_with("#+INCLUDE") {
                let include = &document.includes[0];
                assert_eq!(include.path, "./λ file.org");
                assert_eq!(include.arguments, ["src", "org"]);
                assert_eq!(
                    include
                        .options
                        .iter()
                        .map(|arg| (arg.key.as_str(), arg.value.as_deref(), arg.raw.as_str()))
                        .collect::<Vec<_>>(),
                    expected
                );
            } else {
                let paragraph = document
                    .children
                    .iter()
                    .find(|element| matches!(element.data, ElementData::Paragraph(_)))
                    .expect("paragraph following native ATTR_HTML");
                let attributes = &paragraph.affiliated_keywords[0].attributes;
                assert_eq!(
                    attributes
                        .iter()
                        .map(|arg| (arg.key.as_str(), arg.value.as_deref(), arg.raw.as_str()))
                        .collect::<Vec<_>>(),
                    expected
                );
            }
        }
    }
}
