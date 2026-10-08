//! Clocktable time-window parsing and clock interval clipping.

use super::agenda_model::{days_from_civil, days_in_month};
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

fn native_time_bound(row: &[String]) -> ParsedTimeBound {
    assert_eq!(row.len(), 6, "native clock bound arity");
    ParsedTimeBound {
        bound: ClockTableTimeBound {
            year: row[0].parse().expect("native clock year"),
            month: row[1].parse().expect("native clock month"),
            day: row[2].parse().expect("native clock day"),
            hour: row[3].parse().expect("native clock hour"),
            minute: row[4].parse().expect("native clock minute"),
        },
        minute: row[5].parse().expect("native clock epoch minute"),
    }
}
fn parse_clocktable_time_bound(raw: &str, role: BoundRole) -> Option<ParsedTimeBound> {
    let role = match role {
        BoundRole::Start => "start",
        BoundRole::End => "end",
    };
    let mut rows = super::org_native_values::rows("clock-bound", &[raw, role]);
    assert!(rows.len() <= 1, "native clock bound count");
    rows.pop().map(|row| native_time_bound(&row))
}
fn parse_clocktable_block_window(raw: &str) -> Option<ClockTableWindowFilter> {
    let row = super::org_native_values::optional("clock-window", raw)?;
    assert_eq!(row.len(), 12, "native clock window arity");
    let start = native_time_bound(&row[..6]);
    let end = native_time_bound(&row[6..]);
    Some(ClockTableWindowFilter {
        start_minute: Some(start.minute),
        end_exclusive_minute: Some(end.minute),
        window: ClockTableTimeWindow {
            source: ClockTableTimeWindowSource::Block,
            start: Some(start.bound),
            end_exclusive: Some(end.bound),
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
