//! Native semantic plans and public consumers share the admitted owner.
use crate::{Org, ast::ElementData};

pub(super) fn block_lines_and_dynamic_content_are_native() {
    let value = "\tλ (ref:ok)\r\n\tβ\r";
    let source = ",\tλ (ref:ok)\r\n\tβ\r";
    let rows = crate::org_aot::native_semantic_rows(10, &[value, source, "(ref:%s)", "4", "false"])
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        &rows[0][0..8],
        [
            "1",
            ",\tλ (ref:ok)",
            "\tλ (ref:ok)",
            "λ (ref:ok)",
            "\tλ",
            "λ",
            "4",
            "\r\n"
        ]
    );
    assert_eq!(rows[0][9].parse::<usize>().unwrap(), ",\tλ (ref:ok)".len());
    assert_eq!(&rows[0][10..], ["4", "12", "ok", "(ref:ok)"]);
    assert_eq!(
        rows[1][8].parse::<usize>().unwrap(),
        ",\tλ (ref:ok)\r\n".len()
    );
    assert_eq!(rows[1][7], "\r");
    for (pattern, line, name) in [
        ("bad pattern", "λ (ref:ok)", "ok"),
        ("[%s]", "λ [bad!] [good:_1]", "good:_1"),
        ("%s", "λ good", "good"),
        ("(%s)", "(bad!) (good)", "good"),
        ("[%s]", "[bad", ""),
    ] {
        let rows =
            crate::org_aot::native_semantic_rows(10, &[line, line, pattern, "0", "true"]).unwrap();
        assert_eq!(rows[0][12], name);
        assert_eq!(rows[0][6], "0");
    }
    assert!(crate::org_aot::native_semantic_rows(10, &["", "", "%s", "-1", "false"]).is_err());
    assert!(crate::org_aot::native_semantic_rows(10, &["", "", "%s", "4", "maybe"]).is_err());
    assert!(
        crate::org_aot::native_semantic_rows(10, &["", "", "%s", "4", "true"])
            .unwrap()
            .is_empty()
    );
    let rows = crate::org_aot::native_semantic_rows(
        10,
        &["  α\n\n  β", "  α\n\n  β", "(ref:%s)", "4", "false"],
    )
    .unwrap();
    assert!(rows.iter().all(|row| row[6] == "0"));
    for (text, expected) in [
        ("open\r\n \u{a0}\r\n", vec![vec!["1", "false"]]),
        ("open\r\nλ\r\n\r\n", vec![vec!["2", "true"]]),
        ("", vec![vec!["0", "false"]]),
    ] {
        assert_eq!(
            crate::org_aot::native_semantic_rows(11, &[text]).unwrap(),
            expected
        );
    }
    let parsed = Org::parse("#+begin_src text\r\n\tλ (ref:ok)\r\n\tβ\r\n#+end_src\r\n");
    let document = parsed.document();
    let block = document
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::Block(block) => Some(block),
            _ => None,
        })
        .unwrap();
    assert_eq!(block.lines.len(), 2);
    assert_eq!(block.lines[0].normalized_value_without_code_ref, "λ");
    assert_eq!(block.lines[0].code_ref.as_ref().unwrap().column, 4);
    assert_eq!(block.lines[0].ann.raw, "\tλ (ref:ok)");
    for count in [1_000, 10_000] {
        let content = "  λ (ref:x)\r\n".repeat(count);
        let rows = crate::org_aot::native_semantic_rows(
            10,
            &[&content, &content, "(ref:%s)", "4", "false"],
        )
        .unwrap();
        assert_eq!(rows.len(), count);
        assert_eq!(rows[count - 1][0], count.to_string());
        assert_eq!(
            rows[count - 1][8].parse::<usize>().unwrap(),
            (count - 1) * "  λ (ref:x)\r\n".len()
        );
    }
}

pub(super) fn keyword_prescan_is_one_native_plan() {
    let document = Org::parse(
        "#+filetags: :α:β:α:\n#+options: H:2 H:3 -:nil e:YES\n#+property: X λ β\n#+select_tags: α β\n#+exclude_tags: skip\n#+link: Docs https://example.org/%s\n"
    ).document();
    assert_eq!(document.filetags, ["α", "β"]);
    assert_eq!(document.export_settings.headline_levels, Some(3));
    assert_eq!(document.export_settings.special_strings, Some(false));
    assert_eq!(document.export_settings.expand_entities, Some(true));
    assert_eq!(document.export_settings.select_tags, ["α", "β"]);
    assert_eq!(document.export_settings.exclude_tags, ["skip"]);
    assert_eq!(
        document
            .properties
            .iter()
            .find(|p| p.key == "X")
            .unwrap()
            .value,
        "λ β"
    );
    assert_eq!(document.link_abbreviations[0].name, "docs");
    assert_eq!(
        document.link_abbreviations[0].replacement,
        "https://example.org/%s"
    );
    let facts =
        crate::org_aot::native_semantic_rows(12, &["lınk", "x y", "OPTIONS", "H: -:maybe e:nIl"])
            .unwrap();
    assert_eq!(facts[0], ["index", "0"]);
    assert_eq!(facts[1], ["route", ""]);
    assert!(facts.iter().any(|row| row == &["-", ""]));
    assert!(facts.iter().any(|row| row == &["e", "false"]));
    assert!(crate::org_aot::native_semantic_rows(12, &["unpaired"]).is_err());
    assert!(crate::org_aot::native_semantic_rows(12, &["TITLE", "owner survived\0λ"]).is_ok());
}

pub(super) fn dir_recognition_is_native_and_host_order_is_preserved() {
    let rows =
        crate::org_aot::native_semantic_rows(13, &[" λ/$(printf ')')/$(echo (nested))/$(unclosed"])
            .unwrap();
    assert_eq!(
        rows,
        vec![
            vec!["literal", " λ/", ""],
            vec!["command", "printf ')'", "$(printf ')')"],
            vec!["literal", "/", ""],
            vec!["command", "echo (nested)", "$(echo (nested))"],
            vec!["literal", "/$(unclosed", ""],
        ]
    );
    let rows =
        crate::org_aot::native_semantic_rows(14, &["λ/$A-._1/\u{24}{β}/\u{24}{}/$/$unfinished{"])
            .unwrap();
    assert_eq!(
        rows,
        vec![
            vec!["literal", "λ/", ""],
            vec!["environment", "A-._1", "$A-._1"],
            vec!["literal", "/", ""],
            vec!["environment", "β", "\u{24}{β}"],
            vec!["literal", "/", ""],
            vec!["environment", "", "\u{24}{}"],
            vec!["literal", "/$/", ""],
            vec!["environment", "unfinished", "$unfinished"],
            vec!["literal", "{", ""],
        ]
    );
    assert_eq!(
        crate::org_aot::expand_native_macro_fields(15, &[" \u{a0}{{{x}}} \r\n", "x", "  λ  "])
            .unwrap(),
        "  λ  "
    );
    assert_eq!(
        crate::org_aot::native_semantic_rows(14, &["λ\0\u{24}{MISSING}"]).unwrap()[0][1],
        "λ\0"
    );
    assert!(crate::org_aot::native_semantic_rows(13, &["x", "extra"]).is_err());
    assert_eq!(
        crate::org_aot::native_semantic_rows(13, &["after"]).unwrap(),
        vec![vec!["literal", "after", ""]]
    );
}
