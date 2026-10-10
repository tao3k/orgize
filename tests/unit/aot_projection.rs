//! Public Scheme AOT parser and owned Element projection admissions.

use crate::{
    Org,
    ast::{ElementData, ObjectData},
};

#[path = "aot_projection/content.rs"]
mod content;
#[path = "aot_projection/lifecycle_values.rs"]
mod lifecycle_values;
#[path = "aot_projection/owner_plans.rs"]
mod owner_plans;
#[path = "aot_projection/source_values.rs"]
mod source_values;
#[path = "aot_projection/value_families.rs"]
mod value_families;
use content::{
    public_keyword_and_include_headers_are_decoded_from_content,
    public_macro_definitions_and_escaped_arguments_are_native,
    public_macro_expansion_and_property_calls_are_native,
    public_table_content_and_formula_extents_are_native,
};

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[
    (
        "source_values_preserve_presence_and_inert_syntax",
        source_values::source_values_preserve_presence_and_inert_syntax,
    ),
    (
        "native_clock_windows_cover_leaps_weeks_and_bounds",
        lifecycle_values::native_clock_windows_cover_leaps_weeks_and_bounds,
    ),
    (
        "native_property_tokens_and_progress_cookies_cover_quoting",
        lifecycle_values::native_property_tokens_and_progress_cookies_cover_quoting,
    ),
    (
        "native_protocol_plans_keep_parameter_presence_and_inert_intent",
        lifecycle_values::native_protocol_plans_keep_parameter_presence_and_inert_intent,
    ),
    (
        "lifecycle_values_preserve_public_numeric_domains",
        lifecycle_values::lifecycle_values_preserve_public_numeric_domains,
    ),
    (
        "native_unicode_lowercase_matches_host_reference",
        lifecycle_values::native_unicode_lowercase_matches_host_reference,
    ),
    (
        "native_metadata_time_plans_reject_bad_framing_and_recover",
        lifecycle_values::native_metadata_time_plans_reject_bad_framing_and_recover,
    ),
    (
        "value_families_drive_public_consumers",
        value_families::value_families_drive_public_consumers,
    ),
    (
        "native_value_batches_handle_ten_thousand_entries",
        value_families::native_value_batches_handle_ten_thousand_entries,
    ),
    (
        "native_value_framing_rejects_without_poisoning_owner",
        value_families::native_value_framing_rejects_without_poisoning_owner,
    ),
    (
        "block_lines_and_dynamic_content_are_native",
        owner_plans::block_lines_and_dynamic_content_are_native,
    ),
    (
        "keyword_prescan_is_one_native_plan",
        owner_plans::keyword_prescan_is_one_native_plan,
    ),
    (
        "dir_recognition_is_native_and_host_order_is_preserved",
        owner_plans::dir_recognition_is_native_and_host_order_is_preserved,
    ),
    (
        "public_macro_expansion_and_property_calls_are_native",
        public_macro_expansion_and_property_calls_are_native,
    ),
    (
        "public_table_content_and_formula_extents_are_native",
        public_table_content_and_formula_extents_are_native,
    ),
    (
        "public_macro_definitions_and_escaped_arguments_are_native",
        public_macro_definitions_and_escaped_arguments_are_native,
    ),
    (
        "public_keyword_and_include_headers_are_decoded_from_content",
        public_keyword_and_include_headers_are_decoded_from_content,
    ),
    (
        "public_header_argument_structure_is_owned_by_scheme",
        public_header_argument_structure_is_owned_by_scheme,
    ),
    (
        "public_source_switch_arguments_are_classified_by_scheme",
        public_source_switch_arguments_are_classified_by_scheme,
    ),
    (
        "public_citation_affix_ranges_are_classified_by_scheme",
        public_citation_affix_ranges_are_classified_by_scheme,
    ),
    (
        "public_citation_header_fields_are_classified_by_scheme",
        public_citation_header_fields_are_classified_by_scheme,
    ),
    (
        "public_timestamp_date_fields_are_classified_by_scheme",
        public_timestamp_date_fields_are_classified_by_scheme,
    ),
    (
        "public_timestamp_cookie_fields_are_classified_by_scheme",
        public_timestamp_cookie_fields_are_classified_by_scheme,
    ),
    (
        "public_parser_keeps_opaque_blocks_inside_list_items",
        public_parser_keeps_opaque_blocks_inside_list_items,
    ),
    (
        "public_parser_projects_quoted_emphasis_from_scheme_events",
        public_parser_projects_quoted_emphasis_from_scheme_events,
    ),
    (
        "aot_table_rows_and_cells_reach_the_owned_ast",
        aot_table_rows_and_cells_reach_the_owned_ast,
    ),
    (
        "named_source_block_uses_scheme_affiliation_in_owned_ast",
        named_source_block_uses_scheme_affiliation_in_owned_ast,
    ),
    (
        "nested_elements_reuse_the_engine_ancestor_projection",
        nested_elements_reuse_the_engine_ancestor_projection,
    ),
];

fn public_header_argument_structure_is_owned_by_scheme() {
    for (parameters, expected) in [
        (
            ":results output drawer :var x=1 :var y=2",
            vec![
                ("results", Some("output drawer"), ":results output drawer"),
                ("var", Some("x=1"), ":var x=1"),
                ("var", Some("y=2"), ":var y=2"),
            ],
        ),
        (
            ":var :exports both :empty",
            vec![
                ("var", None, ":var"),
                ("exports", Some("both"), ":exports both"),
                ("empty", None, ":empty"),
            ],
        ),
        (
            ":dir \"/tmp/λ :not-key\" :results output drawer",
            vec![
                (
                    "dir",
                    Some("\"/tmp/λ :not-key\""),
                    ":dir \"/tmp/λ :not-key\"",
                ),
                ("results", Some("output drawer"), ":results output drawer"),
            ],
        ),
        (
            ":var α=1 :bad! ignored :exports both",
            vec![
                ("var", Some("α=1 :bad! ignored"), ":var α=1 :bad! ignored"),
                ("exports", Some("both"), ":exports both"),
            ],
        ),
    ] {
        for (source, kind) in [
            (
                format!("#+begin_src rust {parameters} \t\r\nα\r\n#+end_src\r\n"),
                "src-block",
            ),
            (
                format!("#+begin: clocktable {parameters} \t\r\nα\r\n#+end:\r\n"),
                "dynamic-block",
            ),
            (
                format!("src_rust[{parameters} \t]{{α}}\r\n"),
                "inline-src-block",
            ),
            (format!("#+header: {parameters} \t\r\n"), "keyword"),
            (
                format!("* λ\r\n:PROPERTIES:\r\n:header-args: {parameters} \t\r\n:END:\r\n"),
                "node-property",
            ),
        ] {
            let parsed = Org::parse(&source);
            assert_eq!(parsed.to_org(), source);
            let record = parsed
                .records()
                .iter()
                .find(|record| record.kind == kind)
                .unwrap_or_else(|| panic!("native {kind} record: {source}"));
            assert_eq!(
                record.field("header-parameters"),
                Some(parameters),
                "{source}"
            );
            for field in record.fields.iter().filter(|field| {
                matches!(
                    field.name,
                    "argument-key" | "argument-value" | "argument-raw" | "header-parameters"
                )
            }) {
                assert_eq!(
                    &source[usize::from(field.range.start())..usize::from(field.range.end())],
                    field.value
                );
            }
            let keys = record
                .fields
                .iter()
                .filter(|field| field.name == "argument-key")
                .map(|field| field.value.as_str())
                .collect::<Vec<_>>();
            let raw = record
                .fields
                .iter()
                .filter(|field| field.name == "argument-raw")
                .map(|field| field.value.as_str())
                .collect::<Vec<_>>();
            let values = record
                .fields
                .iter()
                .filter(|field| field.name == "argument-value")
                .map(|field| field.value.as_str())
                .collect::<Vec<_>>();
            assert_eq!(
                keys,
                expected.iter().map(|arg| arg.0).collect::<Vec<_>>(),
                "{source}"
            );
            assert_eq!(
                raw,
                expected.iter().map(|arg| arg.2).collect::<Vec<_>>(),
                "{source}"
            );
            assert_eq!(
                values,
                expected.iter().filter_map(|arg| arg.1).collect::<Vec<_>>(),
                "{source}"
            );
            if matches!(kind, "src-block" | "dynamic-block") {
                let document = parsed.document();
                let ElementData::Block(block) = &document.children[0].data else {
                    panic!("native block");
                };
                assert_eq!(block.parameters.as_deref(), Some(parameters));
                assert_eq!(
                    block
                        .header_args
                        .iter()
                        .map(|arg| (arg.key.as_str(), arg.value.as_deref(), arg.raw.as_str()))
                        .collect::<Vec<_>>(),
                    expected
                );
            }
        }
    }
    for switches in ["-n -i +n 12", "-i :exports both -n 5", "-i -n 3"] {
        let source = format!("#+begin_src rust {switches} \t\r\nα\r\n#+end_src\r\n");
        let parsed = Org::parse(&source);
        assert_eq!(parsed.to_org(), source);
        let document = parsed.document();
        let ElementData::Block(block) = &document.children[0].data else {
            panic!("native block");
        };
        assert_eq!(block.switches.as_deref(), Some(switches));
        assert!(block.preserve_indentation);
    }
}

fn public_source_switch_arguments_are_classified_by_scheme() {
    use crate::ast::BlockLineNumberMode::{Continued, New};
    for (switches, numbering, label) in [
        (
            "-n 5 +n 12 -l \"λ:%s\"",
            Some((Continued, Some(12))),
            Some("λ:%s"),
        ),
        ("-n :exports both", Some((New, None)), None),
        ("-n invalid", Some((New, None)), None),
        ("-l \"\"", None, Some("")),
        ("-l \"unterminated", None, Some("\"unterminated")),
        ("-l 'quoted'", None, Some("'quoted'")),
        ("-l bare -l :exports both", None, None),
    ] {
        for (begin, end) in [("src rust", "src"), ("example", "example")] {
            let source = format!("#+begin_{begin} {switches}\r\nα\r\n#+end_{end}\r\n");
            let parsed = Org::parse(&source);
            assert_eq!(parsed.to_org(), source);
            let record = parsed
                .records()
                .iter()
                .find(|record| matches!(record.kind, "src-block" | "example-block"))
                .expect("native block record");
            for field in record
                .fields
                .iter()
                .filter(|field| field.name.starts_with("switch-"))
            {
                assert_eq!(
                    &source[usize::from(field.range.start())..usize::from(field.range.end())],
                    field.value,
                );
            }
            assert!(
                !parsed
                    .records()
                    .iter()
                    .any(|record| record.kind == "switch-value")
            );
            let document = parsed.document();
            let ElementData::Block(block) = &document.children[0].data else {
                panic!("native block");
            };
            assert_eq!(
                block
                    .line_numbering
                    .as_ref()
                    .map(|value| (value.mode.clone(), value.start)),
                numbering
            );
            assert_eq!(block.switch_options.label_format.as_deref(), label);
        }
    }
}

fn public_citation_affix_ranges_are_classified_by_scheme() {
    for leading in ["", " ", "\t  \t"] {
        // Non-ASCII spacing belongs to the content, not ASCII trivia.
        let source = format!(
            "α [cite:{leading}\u{a0}*global* ;{leading}/prefix/ @key {leading}\u{2003}*suffix* ;{leading}尾 ]\r\n"
        );
        let parsed = Org::parse(&source);
        assert_eq!(parsed.to_org(), source);
        let citation_record = parsed
            .records()
            .iter()
            .find(|record| record.kind == "citation")
            .expect("native citation");
        let reference_record = parsed
            .records()
            .iter()
            .find(|record| record.kind == "citation-reference")
            .expect("native reference");
        for (record, field, expected) in [
            (citation_record, "global-prefix-content", "\u{a0}*global* "),
            (citation_record, "global-suffix-content", "尾 "),
            (reference_record, "prefix-content", "/prefix/ "),
            (reference_record, "suffix-content", "\u{2003}*suffix* "),
        ] {
            assert_eq!(record.field(field), Some(expected));
            let range = record.field_range(field).expect("Scheme content range");
            assert_eq!(
                &source[usize::from(range.start())..usize::from(range.end())],
                expected
            );
        }
        assert_eq!(
            reference_record.field("prefix"),
            Some(format!("{leading}/prefix/ ").as_str())
        );
        let document = parsed.document();
        assert!(document.diagnostics.is_empty());
        let ElementData::Paragraph(objects) = &document.children[0].data else {
            panic!("citation paragraph");
        };
        let citation = objects
            .iter()
            .find_map(|object| match &object.data {
                ObjectData::Citation(citation) => Some(citation),
                _ => None,
            })
            .expect("typed citation");
        for (objects, expected) in [
            (&citation.prefix, "\u{a0}*global* "),
            (&citation.suffix, "尾 "),
            (&citation.references[0].prefix, "/prefix/ "),
            (&citation.references[0].suffix, "\u{2003}*suffix* "),
        ] {
            let raw: String = objects
                .iter()
                .map(|object| object.ann.raw.as_str())
                .collect();
            assert_eq!(raw, expected, "{leading:?}");
            for object in objects {
                assert_eq!(
                    &source[usize::from(object.ann.range.start())
                        ..usize::from(object.ann.range.end())],
                    object.ann.raw
                );
            }
        }
    }
    let source = "[cite: \t; \t@key \t; \t]\r\n";
    let parsed = Org::parse(source);
    assert_eq!(parsed.to_org(), source);
    for record in parsed
        .records()
        .iter()
        .filter(|record| matches!(record.kind, "citation" | "citation-reference"))
    {
        for field in [
            "global-prefix-content",
            "global-suffix-content",
            "prefix-content",
            "suffix-content",
        ] {
            assert!(record.field_range(field).is_none(), "{field}");
        }
    }
    let document = parsed.document();
    let ElementData::Paragraph(objects) = &document.children[0].data else {
        panic!("citation paragraph");
    };
    let citation = objects
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::Citation(citation) => Some(citation),
            _ => None,
        })
        .expect("whitespace-only citation");
    assert!(citation.prefix.is_empty() && citation.suffix.is_empty());
    assert_eq!(citation.references.len(), 1);
    assert!(citation.references[0].prefix.is_empty() && citation.references[0].suffix.is_empty());
}

fn public_citation_header_fields_are_classified_by_scheme() {
    for (head, style, variant) in [
        ("[cite:", None, None),
        ("[cite/text:", Some("text"), None),
        ("[cite/noauthor/bare:", Some("noauthor"), Some("bare")),
        (
            "[cite/text/bare/compact:",
            Some("text"),
            Some("bare/compact"),
        ),
        ("[cite/中文/α:", Some("中文"), Some("α")),
    ] {
        let source = format!("α {head}global *prefix* ; see /also/ @key p. *42*; suffix]\r\n");
        let parsed = Org::parse(&source);
        assert_eq!(parsed.to_org(), source);
        let record = parsed
            .records()
            .iter()
            .find(|record| record.kind == "citation")
            .expect("native citation");
        assert_eq!(record.field("head"), Some(head));
        assert_eq!(record.field("style"), style);
        assert_eq!(record.field("variant"), variant);
        for name in ["head", "style", "variant", "global-prefix", "global-suffix"] {
            if let Some(value) = record.field(name) {
                let range = record.field_range(name).expect("native field range");
                assert_eq!(
                    &source[usize::from(range.start())..usize::from(range.end())],
                    value,
                    "{head} {name}"
                );
            }
        }
        let document = parsed.document();
        assert!(document.diagnostics.is_empty(), "{head}");
        let ElementData::Paragraph(objects) = &document.children[0].data else {
            panic!("citation paragraph");
        };
        let citation = objects
            .iter()
            .find_map(|object| match &object.data {
                ObjectData::Citation(citation) => Some(citation),
                _ => None,
            })
            .expect("typed native citation");
        assert_eq!(citation.style, style.unwrap_or("nil"));
        assert_eq!(citation.variant, variant.unwrap_or_default());
        assert_eq!(citation.references.len(), 1);
        assert_eq!(citation.references[0].id, "key");
        assert!(
            citation
                .prefix
                .iter()
                .any(|object| matches!(object.data, ObjectData::Markup { .. }))
        );
        assert!(!citation.suffix.is_empty());
    }
    for body in [
        "[cite/:@key]",
        "[cite/text/:@key]",
        "[cite/text//bare:@key]",
        "[cite/text bare:@key]",
        "[cite/text:@ ]",
        "[cite/text:no key]",
        "[cite/text:@key",
        "[cite/text:\\@key]",
    ] {
        let source = format!("α {body}\r\n");
        let parsed = Org::parse(&source);
        assert_eq!(parsed.to_org(), source);
        assert!(
            !parsed
                .records()
                .iter()
                .any(|record| record.kind == "citation"),
            "{body}"
        );
    }
    // The scanner must reset style/variant state between adjacent candidates.
    let parsed = Org::parse("[cite/noauthor/bare:@a] [cite:@b] [cite/text:@c]\r\n");
    let records: Vec<_> = parsed
        .records()
        .iter()
        .filter(|record| record.kind == "citation")
        .collect();
    assert_eq!(records.len(), 3);
    assert_eq!(records[0].field("variant"), Some("bare"));
    assert_eq!(records[1].field("style"), None);
    assert_eq!(records[1].field("variant"), None);
    assert_eq!(records[2].field("style"), Some("text"));
    assert_eq!(records[2].field("variant"), None);
}

fn public_timestamp_date_fields_are_classified_by_scheme() {
    for (body, dates) in [
        ("<2026-09-23>", vec![("2026-09-23", "2026", "09", "23")]),
        ("[0000-00-00]", vec![("0000-00-00", "0000", "00", "00")]),
        (
            "<9999-99-99 Wed 10:00-11:00>",
            vec![("9999-99-99", "9999", "99", "99")],
        ),
        (
            "[2026-09-23]--[2027-10-24]",
            vec![
                ("2026-09-23", "2026", "09", "23"),
                ("2027-10-24", "2027", "10", "24"),
            ],
        ),
        (
            "<2026-09-23>-<2027-10-24>",
            vec![
                ("2026-09-23", "2026", "09", "23"),
                ("2027-10-24", "2027", "10", "24"),
            ],
        ),
        ("<2026-9-23>", vec![]),
        ("<2026-09-2>", vec![]),
        ("<20260-09-23>", vec![]),
        ("<2026/09/23>", vec![]),
        ("<2026-0x-23>", vec![]),
        ("<2026-09-23", vec![]),
    ] {
        let source = format!("α {body}\r\n");
        let parsed = Org::parse(&source);
        assert_eq!(parsed.to_org(), source);
        let records: Vec<_> = parsed
            .records()
            .iter()
            .filter(|record| record.kind == "timestamp")
            .collect();
        if dates.is_empty() {
            assert!(records.is_empty(), "{body}");
            continue;
        }
        assert_eq!(records.len(), 1, "{body}");
        let record = records[0];
        for (name, column) in [("date", 0), ("year", 1), ("month", 2), ("day", 3)] {
            let expected: Vec<_> = dates
                .iter()
                .map(|date| match column {
                    0 => date.0,
                    1 => date.1,
                    2 => date.2,
                    _ => date.3,
                })
                .collect();
            assert_eq!(
                record.values(name).collect::<Vec<_>>(),
                expected,
                "{body} {name}"
            );
            for field in record.fields.iter().filter(|field| field.name == name) {
                assert_eq!(
                    &source[usize::from(field.range.start())..usize::from(field.range.end())],
                    field.value,
                    "{body} {name}"
                );
            }
        }
        let document = parsed.document();
        let ElementData::Paragraph(objects) = &document.children[0].data else {
            panic!("timestamp paragraph");
        };
        let typed = objects
            .iter()
            .find_map(|object| match &object.data {
                ObjectData::Timestamp(timestamp) => Some(timestamp),
                _ => None,
            })
            .expect("typed timestamp");
        let start = typed.start.as_ref().expect("classified date");
        assert_eq!(
            (start.year, start.month, start.day),
            (
                dates[0].1.parse().unwrap(),
                dates[0].2.parse().unwrap(),
                dates[0].3.parse().unwrap()
            )
        );
        if dates.len() == 2 {
            let end = typed.end.as_ref().expect("second classified date");
            assert_eq!(
                (end.year, end.month, end.day),
                (
                    dates[1].1.parse().unwrap(),
                    dates[1].2.parse().unwrap(),
                    dates[1].3.parse().unwrap()
                )
            );
            assert!(typed.is_range);
        }
    }
}

fn assert_scheme_timestamp_clock_fields() {
    fn moment(value: &crate::ast::TimestampMoment) -> (u16, Option<&str>, Option<u8>, Option<u8>) {
        (
            value.year,
            value.day_name.as_deref(),
            value.hour,
            value.minute,
        )
    }
    for (body, start, end, range) in [
        (
            "<2026-09-23 Wed 9:05-10:06>",
            Some((2026, Some("Wed"), Some(9), Some(5))),
            Some((2026, Some("Wed"), Some(10), Some(6))),
            true,
        ),
        (
            "<2026-09-23 Wed>--<2027-10-24 Thu 10:06>",
            Some((2026, Some("Wed"), None, None)),
            Some((2027, Some("Thu"), Some(10), Some(6))),
            true,
        ),
        (
            "<2026-09-23 Wed 10:06>--<2027-10-24 Thu>",
            Some((2026, Some("Wed"), Some(10), Some(6))),
            Some((2027, Some("Thu"), None, None)),
            true,
        ),
        ("<2026-09-23 10::05>", None, None, false),
        (
            "<2026-09-23 10:06->",
            Some((2026, None, Some(10), Some(6))),
            None,
            true,
        ),
    ] {
        let source = format!("α {body}\r\n");
        let parsed = Org::parse(&source);
        assert_eq!(parsed.to_org(), source);
        let record = parsed
            .records()
            .iter()
            .find(|r| r.kind == "timestamp")
            .unwrap();
        for field in &record.fields {
            assert_eq!(
                &source[usize::from(field.range.start())..usize::from(field.range.end())],
                field.value,
                "{body} {}",
                field.name
            );
        }
        let document = parsed.document();
        let ElementData::Paragraph(objects) = &document.children[0].data else {
            panic!("timestamp paragraph");
        };
        let object = objects
            .iter()
            .find(|o| matches!(o.data, ObjectData::Timestamp(_)))
            .unwrap();
        let ObjectData::Timestamp(timestamp) = &object.data else {
            unreachable!()
        };
        assert_eq!(timestamp.start.as_ref().map(moment), start, "{body} start");
        assert_eq!(timestamp.end.as_ref().map(moment), end, "{body} end");
        assert_eq!(timestamp.is_range, range, "{body} range");
        let points: Vec<_> = record
            .fields
            .iter()
            .filter(|f| f.name == "point")
            .map(|f| f.range)
            .collect();
        assert_eq!(
            object.ann.timestamp_first_point_range,
            points.first().copied()
        );
        assert_eq!(
            object.ann.timestamp_second_point_range,
            points.get(1).copied()
        );
    }
}

fn public_timestamp_cookie_fields_are_classified_by_scheme() {
    assert_scheme_timestamp_clock_fields();
    for (cookie, field) in [
        ("+1h", Some("repeater")),
        ("++12d", Some("repeater")),
        (".+3w", Some("repeater")),
        ("+4m", Some("repeater")),
        ("+5y", Some("repeater")),
        ("-2d", Some("delay")),
        ("--4w", Some("delay")),
        ("+000d", Some("repeater")),
        ("+4294967295d", Some("repeater")),
        ("+4294967296d", Some("repeater")),
        ("+", None),
        ("++w", None),
        (".1w", None),
        ("+-1w", None),
        ("+1ww", None),
        ("+1.2w", None),
        ("---2d", None),
        ("-d", None),
        ("-2", None),
        ("+1q", None),
    ] {
        let source = format!("α <2026-09-23 {cookie}>\r\n");
        let parsed = Org::parse(&source);
        assert_eq!(parsed.to_org(), source);
        let timestamp = parsed
            .records()
            .iter()
            .find(|record| record.kind == "timestamp")
            .expect("native timestamp record");
        for name in ["repeater", "delay"] {
            assert_eq!(
                timestamp.field(name),
                (field == Some(name)).then_some(cookie),
                "{cookie}"
            );
        }
        let components = match cookie {
            "+1h" => Some(("+", "1", "h")),
            "++12d" => Some(("++", "12", "d")),
            ".+3w" => Some((".+", "3", "w")),
            "+4m" => Some(("+", "4", "m")),
            "+5y" => Some(("+", "5", "y")),
            "-2d" => Some(("-", "2", "d")),
            "--4w" => Some(("--", "4", "w")),
            "+000d" => Some(("+", "000", "d")),
            "+4294967295d" => Some(("+", "4294967295", "d")),
            "+4294967296d" => Some(("+", "4294967296", "d")),
            _ => None,
        };
        for prefix in ["repeater", "delay"] {
            let expected = (field == Some(prefix)).then_some(components).flatten();
            for (suffix, value) in [
                ("mark", expected.map(|value| value.0)),
                ("value", expected.map(|value| value.1)),
                ("unit", expected.map(|value| value.2)),
            ] {
                let name = format!("{prefix}-{suffix}");
                assert_eq!(timestamp.field(&name), value, "{cookie} {name}");
                if let Some(value) = value {
                    let range = timestamp
                        .field_range(&name)
                        .expect("native component range");
                    assert_eq!(
                        &source[usize::from(range.start())..usize::from(range.end())],
                        value
                    );
                }
            }
        }
        if let Some(name) = field {
            let range = timestamp.field_range(name).expect("source-backed cookie");
            assert_eq!(
                &source[usize::from(range.start())..usize::from(range.end())],
                cookie
            );
            assert_eq!(usize::from(range.start()), "α <2026-09-23 ".len());
        }
        let document = parsed.document();
        let ElementData::Paragraph(objects) = &document.children[0].data else {
            panic!("timestamp paragraph");
        };
        let typed = objects
            .iter()
            .find_map(|object| match &object.data {
                ObjectData::Timestamp(timestamp) => Some(timestamp),
                _ => None,
            })
            .expect("typed timestamp");
        let number = components.and_then(|(_, value, _)| value.parse::<u32>().ok());
        assert_eq!(
            typed.repeater.as_ref().map(|value| value.value),
            (field == Some("repeater")).then_some(number).flatten(),
            "{cookie}"
        );
        assert_eq!(
            typed.warning.as_ref().map(|value| value.value),
            (field == Some("delay")).then_some(number).flatten(),
            "{cookie}"
        );
        if let Some(repeater) = &typed.repeater {
            let expected = match cookie {
                "++12d" => crate::ast::RepeaterKind::CatchUp,
                ".+3w" => crate::ast::RepeaterKind::Restart,
                _ => crate::ast::RepeaterKind::Cumulate,
            };
            assert_eq!(repeater.kind, expected, "{cookie}");
        }
        if let Some(warning) = &typed.warning {
            let expected = if cookie == "--4w" {
                crate::ast::WarningKind::First
            } else {
                crate::ast::WarningKind::All
            };
            assert_eq!(warning.kind, expected, "{cookie}");
        }
    }
}

fn public_parser_keeps_opaque_blocks_inside_list_items() {
    for (opening, closing, kind) in [
        ("src rust", "src", "src-block"),
        ("example", "example", "example-block"),
        ("comment", "comment", "comment-block"),
        ("export html", "export", "export-block"),
    ] {
        let source =
            format!("- α\r\n  #+begin_{opening}\r\n  * literal\r\n  #+end_{closing}\r\n- β\r\n");
        let parsed = Org::parse(&source);
        assert_eq!(parsed.to_org(), source);
        let records = parsed.records();
        let block = records
            .iter()
            .find(|record| record.kind == kind)
            .expect(kind);
        let item = &records[block.parent_id.expect("block owner")];
        assert_eq!(item.kind, "item");
        assert!(item.child_ids.contains(&block.id));
        assert!(item.range.start() <= block.range.start());
        assert!(block.range.end() <= item.range.end());
        let list = &records[item.parent_id.expect("item owner")];
        assert_eq!(list.kind, "plain-list");
        assert_eq!(
            list.child_ids
                .iter()
                .filter(|&&id| records[id].kind == "item")
                .count(),
            2
        );
        assert!(!records.iter().any(|record| record.kind == "headline"));
    }
}

fn public_parser_projects_quoted_emphasis_from_scheme_events() {
    for (source, kind) in [(r#""*quoted*""#, "bold"), ("'/quoted/'", "italic")] {
        let parsed = Org::parse(source);
        assert_eq!(parsed.to_org(), source);
        assert_eq!(
            parsed
                .records()
                .iter()
                .filter(|record| record.kind == kind)
                .count(),
            1
        );
    }
}

fn aot_table_rows_and_cells_reach_the_owned_ast() {
    let source = "* H\n| Principle ID | Evidence |\n|--------------+----------|\n| P-001        | one      |\n";
    let parsed = Org::parse(source);
    let document = parsed.document();
    let table = document.sections[0]
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::Table(table) => Some(table),
            _ => None,
        })
        .expect("AOT table in section");
    assert_eq!(table.rows.len(), 3);
    assert_eq!(table.rows[0].cells.len(), 2);
    assert_eq!(table.rows[2].cells.len(), 2);
    assert_eq!(
        table.rows[0].cells[0].objects[0].data,
        ObjectData::Plain("Principle ID".to_owned())
    );
}

fn named_source_block_uses_scheme_affiliation_in_owned_ast() {
    let document = Org::parse(
        "* Contract\n#+NAME: task.rule\n#+BEGIN_SRC org-contract\n(assert exists (headline))\n#+END_SRC\n",
    )
    .document();
    let blocks = document.source_block_records();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].name.as_deref(), Some("task.rule"));
}

fn nested_elements_reuse_the_engine_ancestor_projection() {
    let source = "#+TITLE: File\n* Parent\n** Child\n:PROPERTIES:\n:ID: child\n:END:\nBody\n";
    let parsed = Org::parse(source);
    let records = parsed.records();
    let parent = records
        .iter()
        .find(|record| record.kind == "headline" && record.field("title") == Some("Parent"))
        .expect("parent headline");
    let child = records
        .iter()
        .find(|record| record.kind == "headline" && record.field("title") == Some("Child"))
        .expect("child headline");
    let property = records
        .iter()
        .find(|record| record.kind == "node-property")
        .expect("nested property");
    assert_eq!(parsed.nearest_headline_ancestor(parent.id), None);
    assert_eq!(parsed.nearest_headline_ancestor(child.id), Some(parent.id));
    assert_eq!(
        parsed.nearest_headline_ancestor(property.id),
        Some(child.id)
    );
    assert_eq!(parsed.keywords().count(), 1);
    assert!(parsed.document().diagnostics.is_empty());
}
