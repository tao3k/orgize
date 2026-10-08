//! Native lifecycle/metadata grammar admission through public and typed owners.

pub(super) fn native_clock_windows_cover_leaps_weeks_and_bounds() {
    assert_eq!(
        plan("clock-bound", &["1970-01-01", "start"]),
        [vec!["1970", "1", "1", "0", "0", "0"]]
    );
    assert_eq!(
        plan("clock-window", &["1970-Q1"]),
        [vec![
            "1970", "1", "1", "0", "0", "0", "1970", "4", "1", "0", "0", "129600"
        ]]
    );
    let week = plan("clock-window", &["2020-W53"]);
    assert_eq!(&week[0][..5], ["2020", "12", "28", "0", "0"]);
    assert_eq!(&week[0][6..11], ["2021", "1", "4", "0", "0"]);
    assert!(plan("clock-window", &["2021-W53"]).is_empty());
    assert!(plan("clock-window", &["65535"]).is_empty());
    assert!(plan("clock-bound", &["2026-02-29", "start"]).is_empty());
    let leap = plan("clock-bound", &["\"[2000-02-28 Mon]\"", "end"]);
    assert_eq!(&leap[0][..5], ["2000", "2", "29", "0", "0"]);
}
pub(super) fn native_property_tokens_and_progress_cookies_cover_quoting() {
    assert_eq!(
        plan(
            "property-tokens",
            &["\"\" 'two words' escaped\\ value tail\\"]
        ),
        [vec!["", "two words", "escaped value", "tail\\"]]
    );
    assert_eq!(
        plan("clock-property-names", &["'(EFFORT \"CUSTOM NAME\")"]),
        [vec!["EFFORT", "CUSTOM NAME"]]
    );
    assert!(plan("clock-property-names", &["'(BAD \"quote)"]).is_empty());
    assert!(plan("clock-property-names", &["'nil"]).is_empty());
    assert_eq!(
        plan("clock-property-names", &["NIL"]),
        [Vec::<String>::new()]
    );
    assert_eq!(
        plan("statistic-cookie", &["[[ +01 / +02 ]]"]),
        [vec!["fraction", "1", "2", ""]]
    );
    assert_eq!(
        plan("statistic-cookie", &["[100%]"]),
        [vec!["percent", "", "", "100"]]
    );
    assert_eq!(
        plan("statistic-cookie", &["[101%]"]),
        [vec!["unknown", "", "", ""]]
    );
    assert_eq!(
        plan("descriptor-name", &["effort_ALL+++"]),
        [vec!["effort"]]
    );
    assert_eq!(plan("tag-values", &[":a::b:"]), [vec!["a", "b"]]);
}
pub(super) fn native_protocol_plans_keep_parameter_presence_and_inert_intent() {
    assert_eq!(
        plan("uri-split", &["HTTP:path:tail"]),
        [vec!["HTTP", "path:tail"]]
    );
    assert_eq!(
        plan("link-protocol-kind", &["shell", "false"]),
        [vec!["executable"]]
    );
    assert_eq!(
        plan(
            "org-protocol",
            &["ORG-PROTOCOL://capture?key=&flag&x=a%20b", "raw"]
        ),
        [
            vec!["capture", "capture"],
            vec!["key=", "key", "", "true"],
            vec!["flag", "flag", "", "false"],
            vec!["x=a%20b", "x", "a%20b", "true"]
        ]
    );
    assert!(plan("org-protocol", &["https://capture?x=y", "raw"]).is_empty());
}
use crate::ast::{OrgDuration, PriorityValue};
fn plan(name: &str, fields: &[&str]) -> Vec<Vec<String>> {
    let mut values = vec![name];
    values.extend_from_slice(fields);
    crate::org_aot::native_semantic_rows(23, &values).unwrap()
}
pub(super) fn native_lifecycle_values_preserve_public_numeric_domains() {
    assert_eq!(
        PriorityValue::parse("\u{2003}A\u{2003}"),
        Some(PriorityValue::Letter('A'))
    );
    assert_eq!(PriorityValue::parse("64"), Some(PriorityValue::Numeric(64)));
    for raw in ["00", "064", "65", "a", "AB", ""] {
        assert!(PriorityValue::parse(raw).is_none());
    }
    for (raw, seconds) in [
        ("", 0),
        ("1:30", 5400),
        ("1:02:03", 3723),
        ("1d3h5min", 97500),
        ("1h 0:30", 5400),
        ("1y", 31557600),
        (".5", 30),
        ("1e2", 6000),
        ("1:99:99", 9639),
        ("1e308", u64::MAX),
    ] {
        let duration = OrgDuration::parse(raw).unwrap();
        assert_eq!(duration.raw, raw);
        assert_eq!(duration.total_seconds, seconds, "{raw}");
    }
    for raw in [
        "-0", "-1", "NaN", "inf", "1e999", "1/2", "#x10", "1:2", "1h junk",
    ] {
        assert!(OrgDuration::parse(raw).is_none(), "{raw}");
    }
    assert_eq!(
        plan("archive", &[" file.org :: Heading::Nested "]),
        [vec![
            "file.org :: Heading::Nested",
            "file.org",
            "Heading::Nested"
        ]]
    );
    assert_eq!(
        plan("timer-stamps", &["λ +1:02:03 -0:01:00 x2:00:00"]),
        [vec!["+1:02:03", "3723"], vec!["-0:01:00", "-60"]]
    );
    assert!(plan("timer-stamps", &["9223372036854775807:00:00"]).is_empty());
}
pub(super) fn native_unicode_lowercase_matches_host_reference() {
    // Reference execution is test-only: production lowercasing belongs to Scheme.
    // Exhaustive scalar coverage, plus context controls, catches provider-version drift.
    for block in (0..0x110000).step_by(4096) {
        let source: String = (block..(block + 4096).min(0x110000))
            .filter_map(char::from_u32)
            .collect();
        let actual = plan("link-search-normalize", &["text", "", &source]);
        assert_eq!(
            actual,
            [vec![source.to_lowercase()]],
            "scalar block {block:x}"
        );
    }
    for source in [
        "ΣΟΣ",
        "ΣΟΣİ",
        "AΣ\u{301}",
        "AΣ\u{301}B",
        "AΣ'B",
        "ΣΣ",
        "ΑΣ.",
    ] {
        assert_eq!(
            plan("link-search-normalize", &["text", "", source]),
            [vec![source.to_lowercase()]],
            "{source}"
        );
    }
}
pub(super) fn native_metadata_time_plans_reject_bad_framing_and_recover() {
    for fields in [
        vec!["missing", "x"],
        vec!["priority"],
        vec!["duration", "1h", "extra"],
    ] {
        assert!(crate::org_aot::native_semantic_rows(23, &fields).is_err());
    }
    assert_eq!(
        plan("headline-time", &["<2026-10-05 Mon 08:00> 9am--10:30pm"]),
        [vec!["9", "0", "22", "30"]]
    );
    assert_eq!(plan("words", &[" A\u{2003}B\nλ "]), [vec!["A", "B", "λ"]]);
    assert_eq!(
        plan("link-abbreviation-index", &["HtTp", "ftp", "HTTP", "http"]),
        [vec!["1"]]
    );
    assert_eq!(
        plan("feed-status", &[" ((\"A\" x))\r\n\n ((\"B\" y)) "]),
        [vec!["((\"A\" x))\n((\"B\" y))", "true", "2"]]
    );
}
