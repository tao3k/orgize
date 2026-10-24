//! End-to-end source-backed timestamp Objects from Scheme through Rust/native.

fn scheme_timestamps_project_structured_fields_through_native_index() {
    let source = "<2026-09-23 Wed 10:00-11:00 ++1w -2d> [2026-09-23]--[2026-09-24] <%%(diary-float t 1 2)> <%%(diary-float t 4 2) 12:00-14:00>\n";
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("Scheme timestamp Objects build a lossless native navigation index");
    let timestamps = document
        .records()
        .iter()
        .filter(|record| record.kind == "timestamp")
        .collect::<Vec<_>>();
    assert_eq!(timestamps.len(), 4);
    assert_eq!(
        timestamps[0].values("date").collect::<Vec<_>>(),
        ["2026-09-23"]
    );
    assert_eq!(timestamps[0].field("day-name"), Some("Wed"));
    assert_eq!(timestamps[0].field("time"), Some("10:00-11:00"));
    assert_eq!(timestamps[0].field("repeater"), Some("++1w"));
    assert_eq!(timestamps[0].field("delay"), Some("-2d"));
    assert_eq!(
        timestamps[1].values("date").collect::<Vec<_>>(),
        ["2026-09-23", "2026-09-24"]
    );
    assert_eq!(timestamps[1].field("range-separator"), Some("--"));
    assert_eq!(
        timestamps[2].field("diary-expression"),
        Some("%%(diary-float t 1 2)")
    );
    assert_eq!(timestamps[3].field("time"), Some("12:00-14:00"));
    assert_eq!(document.syntax().to_string(), source);
}

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[(
    "org_timestamp_aot::scheme_timestamps_project_structured_fields_through_native_index",
    scheme_timestamps_project_structured_fields_through_native_index,
)];
