//! Clocktable time-window parsing and clock interval clipping.

use super::agenda_model::{civil_from_days, days_from_civil, days_in_month};
use super::{
    Clock, ClockTableParameter, ClockTableTimeBound, ClockTableTimeWindow,
    ClockTableTimeWindowSource, ClockTableWarning, ClockTableWarningKind, TimestampMoment,
};

#[derive(Clone, Debug)]
pub(crate) struct ClockTableWindowFilter {
    pub(crate) window: ClockTableTimeWindow,
    start_minute: Option<i64>,
    end_exclusive_minute: Option<i64>,
}

impl ClockTableWindowFilter {
    fn contains_minute(&self, minute: i64) -> bool {
        self.start_minute.is_none_or(|start| minute >= start)
            && self
                .end_exclusive_minute
                .is_none_or(|end_exclusive| minute < end_exclusive)
    }

    fn overlap_minutes(&self, start: i64, end_exclusive: i64) -> Option<u64> {
        if end_exclusive <= start {
            return Some(0);
        }

        let overlap_start = self.start_minute.map_or(start, |bound| start.max(bound));
        let overlap_end = self
            .end_exclusive_minute
            .map_or(end_exclusive, |bound| end_exclusive.min(bound));
        if overlap_end <= overlap_start {
            Some(0)
        } else {
            Some((overlap_end - overlap_start) as u64)
        }
    }
}

pub(crate) fn clock_table_time_window(
    parameters: &[ClockTableParameter],
) -> (Option<ClockTableWindowFilter>, Vec<ClockTableWarning>) {
    if parameter_value(parameters, "tstart").is_some()
        || parameter_value(parameters, "tend").is_some()
    {
        return clock_table_tstart_tend_window(parameters);
    }

    if let Some(block) = parameter_value(parameters, "block") {
        return clock_table_block_window(&block);
    }

    (None, Vec::new())
}

pub(crate) fn clipped_clock_seconds(
    clock: &Clock,
    duration_seconds: u64,
    time_window: &ClockTableWindowFilter,
) -> Option<u64> {
    let start = clock_start_minute(clock)?;
    let end_exclusive = clock_end_minute(clock)
        .or_else(|| start.checked_add(duration_seconds.div_ceil(60) as i64))?;
    let overlap_minutes = time_window.overlap_minutes(start, end_exclusive)?;
    if overlap_minutes == 0 {
        return Some(0);
    }

    if overlap_minutes == (end_exclusive - start) as u64 {
        Some(duration_seconds)
    } else {
        Some(overlap_minutes.saturating_mul(60))
    }
}

pub(crate) fn clock_start_in_window(
    clock: &Clock,
    time_window: &ClockTableWindowFilter,
) -> Option<bool> {
    Some(time_window.contains_minute(clock_start_minute(clock)?))
}

fn clock_table_tstart_tend_window(
    parameters: &[ClockTableParameter],
) -> (Option<ClockTableWindowFilter>, Vec<ClockTableWarning>) {
    let tstart = parameter_value(parameters, "tstart");
    let tend = parameter_value(parameters, "tend");
    let start = parse_optional_clocktable_time_bound(tstart.as_deref(), BoundRole::Start);
    let end = parse_optional_clocktable_time_bound(tend.as_deref(), BoundRole::End);

    match (start, end) {
        (Some(start), Some(end)) => {
            let window = ClockTableTimeWindow {
                source: ClockTableTimeWindowSource::TstartTend,
                start: start.map(|bound| bound.bound),
                end_exclusive: end.map(|bound| bound.bound),
            };
            (
                Some(ClockTableWindowFilter {
                    start_minute: start.map(|bound| bound.minute),
                    end_exclusive_minute: end.map(|bound| bound.minute),
                    window,
                }),
                Vec::new(),
            )
        }
        _ => (
            None,
            vec![ClockTableWarning {
                kind: ClockTableWarningKind::TimeRangePreserved,
                message: "tstart/tend parameters are preserved; only absolute Org timestamp or YYYY-MM-DD bounds are applied"
                    .to_string(),
            }],
        ),
    }
}

fn clock_table_block_window(
    block: &str,
) -> (Option<ClockTableWindowFilter>, Vec<ClockTableWarning>) {
    match parse_clocktable_block_window(block) {
        Some(window) => (Some(window), Vec::new()),
        None => (
            None,
            vec![ClockTableWarning {
                kind: ClockTableWarningKind::BlockRangePreserved,
                message: "block parameter is preserved; only absolute YYYY, YYYY-QN, YYYY-MM, YYYY-WNN, or YYYY-MM-DD blocks are applied"
                    .to_string(),
            }],
        ),
    }
}

fn parameter_value(parameters: &[ClockTableParameter], key: &str) -> Option<String> {
    parameters
        .iter()
        .find(|parameter| parameter.key.eq_ignore_ascii_case(key))
        .and_then(|parameter| parameter.value.clone())
}

#[derive(Clone, Copy)]
enum BoundRole {
    Start,
    End,
}

#[derive(Clone, Copy)]
struct ParsedTimeBound {
    bound: ClockTableTimeBound,
    minute: i64,
}

fn parse_optional_clocktable_time_bound(
    raw: Option<&str>,
    role: BoundRole,
) -> Option<Option<ParsedTimeBound>> {
    match raw {
        Some(value) => parse_clocktable_time_bound(value, role).map(Some),
        None => Some(None),
    }
}

fn parse_clocktable_time_bound(raw: &str, role: BoundRole) -> Option<ParsedTimeBound> {
    let value = normalized_parameter_value(raw);
    let value = strip_timestamp_delimiters(value.as_str());
    let (year, month, day, rest) = parse_date_prefix(value)?;
    let (hour, minute, has_time) = parse_time_from_rest(rest).unwrap_or((0, 0, false));
    let bound = ClockTableTimeBound {
        year,
        month,
        day,
        hour,
        minute,
    };
    let bound = if matches!(role, BoundRole::End) && !has_time {
        add_days_to_bound(bound, 1)?
    } else {
        bound
    };
    Some(ParsedTimeBound {
        minute: bound_to_minute(bound)?,
        bound,
    })
}

fn parse_clocktable_block_window(raw: &str) -> Option<ClockTableWindowFilter> {
    let value = normalized_parameter_value(raw);
    let value = value.as_str();
    let (start, end_exclusive) = if let Some((year, quarter)) = parse_quarter_block(value) {
        let month = (quarter - 1) * 3 + 1;
        (
            date_bound(year, month, 1)?,
            add_months_to_month_start(year, month, 3)?,
        )
    } else if let Some((year, week)) = parse_week_block(value) {
        let start = iso_week_start_bound(year, week)?;
        (start, add_days_to_bound(start, 7)?)
    } else if let Some((year, month, day, rest)) = parse_date_prefix(value) {
        if !rest.trim().is_empty() {
            return None;
        }
        let start = date_bound(year, month, day)?;
        (start, add_days_to_bound(start, 1)?)
    } else if let Some((year, month)) = parse_month_block(value) {
        (
            date_bound(year, month, 1)?,
            add_months_to_month_start(year, month, 1)?,
        )
    } else {
        let year = parse_year_block(value)?;
        (
            date_bound(year, 1, 1)?,
            date_bound(year.checked_add(1)?, 1, 1)?,
        )
    };

    Some(ClockTableWindowFilter {
        start_minute: Some(bound_to_minute(start)?),
        end_exclusive_minute: Some(bound_to_minute(end_exclusive)?),
        window: ClockTableTimeWindow {
            source: ClockTableTimeWindowSource::Block,
            start: Some(start),
            end_exclusive: Some(end_exclusive),
        },
    })
}

pub(crate) fn clock_start_minute(clock: &Clock) -> Option<i64> {
    clock
        .value
        .as_ref()
        .and_then(|timestamp| timestamp.start.as_ref())
        .and_then(moment_to_minute)
}

pub(crate) fn clock_end_minute(clock: &Clock) -> Option<i64> {
    clock
        .value
        .as_ref()
        .and_then(|timestamp| timestamp.end.as_ref())
        .and_then(moment_to_minute)
}

pub(crate) fn clock_start_bound(clock: &Clock) -> Option<ClockTableTimeBound> {
    clock
        .value
        .as_ref()
        .and_then(|timestamp| timestamp.start.as_ref())
        .and_then(moment_to_bound)
}

pub(crate) fn clock_end_bound(clock: &Clock) -> Option<ClockTableTimeBound> {
    clock
        .value
        .as_ref()
        .and_then(|timestamp| timestamp.end.as_ref())
        .and_then(moment_to_bound)
}

fn moment_to_minute(moment: &TimestampMoment) -> Option<i64> {
    moment_to_bound(moment).and_then(bound_to_minute)
}

fn moment_to_bound(moment: &TimestampMoment) -> Option<ClockTableTimeBound> {
    let bound = ClockTableTimeBound {
        year: moment.year,
        month: moment.month,
        day: moment.day,
        hour: moment.hour.unwrap_or(0),
        minute: moment.minute.unwrap_or(0),
    };
    bound_to_minute(bound)?;
    Some(bound)
}

fn normalized_parameter_value(raw: &str) -> String {
    raw.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn strip_timestamp_delimiters(value: &str) -> &str {
    let value = value.trim();
    if value.len() >= 2
        && ((value.starts_with('<') && value.ends_with('>'))
            || (value.starts_with('[') && value.ends_with(']')))
    {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn parse_date_prefix(value: &str) -> Option<(u16, u8, u8, &str)> {
    let value = strip_timestamp_delimiters(value.trim());
    if value.len() < 10 {
        return None;
    }
    let date = value.get(..10)?;
    let mut parts = date.split('-');
    let year = parse_u16(parts.next()?)?;
    let month = parse_u8(parts.next()?)?;
    let day = parse_u8(parts.next()?)?;
    if parts.next().is_some() || value.get(4..5)? != "-" || value.get(7..8)? != "-" {
        return None;
    }
    date_bound(year, month, day)?;
    Some((year, month, day, value.get(10..)?))
}

fn parse_time_from_rest(rest: &str) -> Option<(u8, u8, bool)> {
    rest.split_whitespace()
        .find_map(|token| parse_time_token(token).map(|(hour, minute)| (hour, minute, true)))
}

fn parse_time_token(token: &str) -> Option<(u8, u8)> {
    let token =
        token.trim_matches(|ch: char| matches!(ch, '<' | '>' | '[' | ']' | '"' | '\'' | ',' | ';'));
    let mut parts = token.split(':');
    let hour = parse_u8(parts.next()?)?;
    let minute_part = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    let minute_digits = minute_part
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if minute_digits.len() != 2 {
        return None;
    }
    let minute = parse_u8(&minute_digits)?;
    (hour < 24 && minute < 60).then_some((hour, minute))
}

fn parse_quarter_block(value: &str) -> Option<(u16, u8)> {
    let (year, quarter) = value.split_once("-Q")?;
    let year = parse_u16(year)?;
    let quarter = parse_u8(quarter)?;
    (1..=4).contains(&quarter).then_some((year, quarter))
}

fn parse_week_block(value: &str) -> Option<(u16, u8)> {
    let (year, week) = value.split_once("-W")?;
    let year = parse_u16(year)?;
    let week = parse_u8(week)?;
    (1..=53).contains(&week).then_some((year, week))
}

fn parse_month_block(value: &str) -> Option<(u16, u8)> {
    if value.len() != 7 || value.get(4..5)? != "-" {
        return None;
    }
    let year = parse_u16(value.get(..4)?)?;
    let month = parse_u8(value.get(5..)?)?;
    (1..=12).contains(&month).then_some((year, month))
}

fn parse_year_block(value: &str) -> Option<u16> {
    (value.len() == 4).then(|| parse_u16(value)).flatten()
}

fn parse_u16(value: &str) -> Option<u16> {
    (!value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit()))
        .then(|| value.parse().ok())
        .flatten()
}

fn parse_u8(value: &str) -> Option<u8> {
    (!value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit()))
        .then(|| value.parse().ok())
        .flatten()
}

fn date_bound(year: u16, month: u8, day: u8) -> Option<ClockTableTimeBound> {
    if month == 0 || month > 12 {
        return None;
    }
    let max_day = days_in_month(i32::from(year), i32::from(month));
    if day == 0 || day > max_day {
        return None;
    }
    Some(ClockTableTimeBound {
        year,
        month,
        day,
        hour: 0,
        minute: 0,
    })
}

fn bound_to_minute(bound: ClockTableTimeBound) -> Option<i64> {
    if bound.hour >= 24 || bound.minute >= 60 {
        return None;
    }
    let date = date_bound(bound.year, bound.month, bound.day)?;
    let day_number = i64::from(days_from_civil(
        i32::from(date.year),
        u32::from(date.month),
        u32::from(date.day),
    ));
    day_number
        .checked_mul(1_440)?
        .checked_add(i64::from(bound.hour) * 60 + i64::from(bound.minute))
}

fn add_days_to_bound(bound: ClockTableTimeBound, days: i32) -> Option<ClockTableTimeBound> {
    let day_number = days_from_civil(
        i32::from(bound.year),
        u32::from(bound.month),
        u32::from(bound.day),
    )
    .checked_add(days)?;
    bound_from_day_number(day_number, bound.hour, bound.minute)
}

fn add_months_to_month_start(year: u16, month: u8, months: i32) -> Option<ClockTableTimeBound> {
    if month == 0 || month > 12 {
        return None;
    }
    let total = i32::from(year) * 12 + i32::from(month) - 1 + months;
    let year = total.div_euclid(12);
    let month = total.rem_euclid(12) + 1;
    date_bound(year.try_into().ok()?, month.try_into().ok()?, 1)
}

fn bound_from_day_number(day_number: i32, hour: u8, minute: u8) -> Option<ClockTableTimeBound> {
    let (year, month, day) = civil_from_days(day_number);
    Some(ClockTableTimeBound {
        year: year.try_into().ok()?,
        month: month.try_into().ok()?,
        day: day.try_into().ok()?,
        hour,
        minute,
    })
}

fn iso_week_start_bound(year: u16, week: u8) -> Option<ClockTableTimeBound> {
    let jan4 = days_from_civil(i32::from(year), 1, 4);
    let week1_monday = jan4 - (i32::from(iso_weekday(jan4)) - 1);
    let next_jan4 = days_from_civil(i32::from(year.checked_add(1)?), 1, 4);
    let next_week1_monday = next_jan4 - (i32::from(iso_weekday(next_jan4)) - 1);
    let start_day = week1_monday.checked_add(i32::from(week - 1).checked_mul(7)?)?;
    if start_day >= next_week1_monday {
        return None;
    }
    bound_from_day_number(start_day, 0, 0)
}

fn iso_weekday(day_number: i32) -> u8 {
    (day_number + 3).rem_euclid(7) as u8 + 1
}
