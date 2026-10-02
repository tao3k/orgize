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
    let mut delimiters = record
        .fields
        .iter()
        .filter(|field| field.name == "delimiter");
    let first = delimiters
        .next()
        .zip(delimiters.next())
        .map(|(opening, closing)| TextRange::new(opening.range.start(), closing.range.end()));
    let second = delimiters
        .next()
        .zip(delimiters.next())
        .map(|(opening, closing)| TextRange::new(opening.range.start(), closing.range.end()));
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
    let mut points: Vec<GraphTimestampPoint<'_>> = Vec::new();
    for field in &record.fields {
        match field.name {
            "date" => points.push(GraphTimestampPoint {
                date: &field.value,
                day_name: None,
                time: None,
            }),
            "day-name" => {
                if let Some(point) = points.last_mut() {
                    point.day_name = Some(&field.value);
                }
            }
            "time" => {
                if let Some(point) = points.last_mut() {
                    point.time = Some(&field.value);
                }
            }
            _ => {}
        }
    }
    let first = points.first();
    let inline_time_range = first
        .and_then(|point| point.time)
        .and_then(|time| time.split_once('-'));
    let start = first.and_then(|point| {
        graph_timestamp_moment(
            point,
            inline_time_range.map_or(point.time, |range| Some(range.0)),
        )
    });
    let end = points
        .get(1)
        .and_then(|point| graph_timestamp_moment(point, point.time))
        .or_else(|| {
            first.and_then(|point| {
                inline_time_range.and_then(|(_, end)| graph_timestamp_moment(point, Some(end)))
            })
        });
    Timestamp {
        kind,
        raw: raw.to_owned(),
        is_range: record.values("range-separator").next().is_some() || inline_time_range.is_some(),
        start,
        end,
        repeater: record.field("repeater").and_then(graph_repeater_cookie),
        warning: record.field("delay").and_then(graph_warning_cookie),
    }
}

struct GraphTimestampPoint<'a> {
    date: &'a str,
    day_name: Option<&'a str>,
    time: Option<&'a str>,
}

fn graph_timestamp_moment(
    point: &GraphTimestampPoint<'_>,
    time: Option<&str>,
) -> Option<TimestampMoment> {
    let (year, month_day) = point.date.split_once('-')?;
    let (month, day) = month_day.split_once('-')?;
    let (hour, minute) = match time {
        Some(time) => {
            let (hour, minute) = time.split_once(':')?;
            (Some(hour.parse().ok()?), Some(minute.parse().ok()?))
        }
        None => (None, None),
    };
    Some(TimestampMoment {
        year: year.parse().ok()?,
        month: month.parse().ok()?,
        day: day.parse().ok()?,
        day_name: point.day_name.map(str::to_owned),
        hour,
        minute,
    })
}

fn graph_repeater_cookie(value: &str) -> Option<TimestampRepeater> {
    let (kind, body) = if let Some(body) = value.strip_prefix("++") {
        (RepeaterKind::CatchUp, body)
    } else if let Some(body) = value.strip_prefix(".+") {
        (RepeaterKind::Restart, body)
    } else {
        (RepeaterKind::Cumulate, value.strip_prefix('+')?)
    };
    let (value, unit) = graph_cookie_value_and_unit(body)?;
    Some(TimestampRepeater { kind, value, unit })
}

fn graph_warning_cookie(value: &str) -> Option<TimestampWarning> {
    let (kind, body) = if let Some(body) = value.strip_prefix("--") {
        (WarningKind::First, body)
    } else {
        (WarningKind::All, value.strip_prefix('-')?)
    };
    let (value, unit) = graph_cookie_value_and_unit(body)?;
    Some(TimestampWarning { kind, value, unit })
}

fn graph_cookie_value_and_unit(body: &str) -> Option<(u32, TimeUnit)> {
    let number = body.get(..body.len().checked_sub(1)?)?.parse().ok()?;
    let unit = match body.as_bytes().last()? {
        b'h' => TimeUnit::Hour,
        b'd' => TimeUnit::Day,
        b'w' => TimeUnit::Week,
        b'm' => TimeUnit::Month,
        b'y' => TimeUnit::Year,
        _ => return None,
    };
    Some((number, unit))
}
