//! Owned timestamp values projected only from Scheme-classified graph fields.

use gerbil_parser_rowan::GraphRecord;
use rowan::TextRange;

use super::timestamp_model::{
    RepeaterKind, TimeUnit, Timestamp, TimestampKind, TimestampMoment, TimestampRepeater,
    TimestampWarning, WarningKind,
};

pub(super) fn timestamp_point_ranges(
    record: &GraphRecord,
) -> (Option<TextRange>, Option<TextRange>) {
    let mut points = record
        .fields
        .iter()
        .filter(|field| field.name == "point")
        .map(|field| field.range);
    let first = points
        .next()
        .or_else(|| record.field("diary-expression").map(|_| record.range));
    let second = points.next();
    (first, second)
}

pub(super) fn project_timestamp(record: &GraphRecord, raw: &str) -> Timestamp {
    let kind = if record.field("diary-expression").is_some() {
        TimestampKind::Diary
    } else if record.values("delimiter").next() == Some("[") {
        TimestampKind::Inactive
    } else {
        TimestampKind::Active
    };
    let start = graph_timestamp_moment(
        record,
        ["year", "month", "day"],
        ["first-time", "hour", "minute"],
        "day-name",
    );
    let end = if record.field("second-year").is_some() {
        graph_timestamp_moment(
            record,
            ["second-year", "second-month", "second-day"],
            ["second-time", "second-hour", "second-minute"],
            "second-day-name",
        )
    } else if record.field("inline-end-time").is_some() {
        graph_timestamp_moment(
            record,
            ["year", "month", "day"],
            ["inline-end-time", "inline-end-hour", "inline-end-minute"],
            "day-name",
        )
    } else {
        None
    };
    Timestamp {
        kind,
        raw: raw.to_owned(),
        is_range: record.field("range-separator").is_some()
            || record.field("time-range-separator").is_some(),
        start,
        end,
        repeater: graph_repeater_cookie(record),
        warning: graph_warning_cookie(record),
    }
}

// Fixed Scheme field bindings and decimal/enum conversion, not Org parsing.
fn graph_timestamp_moment(
    record: &GraphRecord,
    date: [&str; 3],
    clock: [&str; 3],
    day_name: &str,
) -> Option<TimestampMoment> {
    let (hour, minute) = match record.field(clock[0]) {
        Some(_) => (
            Some(record.field(clock[1])?.parse().ok()?),
            Some(record.field(clock[2])?.parse().ok()?),
        ),
        None => (None, None),
    };
    Some(TimestampMoment {
        year: record.field(date[0])?.parse().ok()?,
        month: record.field(date[1])?.parse().ok()?,
        day: record.field(date[2])?.parse().ok()?,
        day_name: record.field(day_name).map(str::to_owned),
        hour,
        minute,
    })
}

fn graph_repeater_cookie(record: &GraphRecord) -> Option<TimestampRepeater> {
    let kind = match record.field("repeater-mark")? {
        "++" => RepeaterKind::CatchUp,
        ".+" => RepeaterKind::Restart,
        "+" => RepeaterKind::Cumulate,
        _ => return None,
    };
    let value = record.field("repeater-value")?.parse().ok()?;
    let unit = graph_cookie_unit(record.field("repeater-unit")?)?;
    Some(TimestampRepeater { kind, value, unit })
}

fn graph_warning_cookie(record: &GraphRecord) -> Option<TimestampWarning> {
    let kind = match record.field("delay-mark")? {
        "--" => WarningKind::First,
        "-" => WarningKind::All,
        _ => return None,
    };
    let value = record.field("delay-value")?.parse().ok()?;
    let unit = graph_cookie_unit(record.field("delay-unit")?)?;
    Some(TimestampWarning { kind, value, unit })
}

// Enum decoding of Scheme-classified fields, not a second cookie recognizer.
fn graph_cookie_unit(unit: &str) -> Option<TimeUnit> {
    Some(match unit {
        "h" => TimeUnit::Hour,
        "d" => TimeUnit::Day,
        "w" => TimeUnit::Week,
        "m" => TimeUnit::Month,
        "y" => TimeUnit::Year,
        _ => return None,
    })
}
