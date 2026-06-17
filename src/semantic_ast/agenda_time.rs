//! Plain headline time extraction for semantic agenda rows.

use super::agenda_model::{AgendaQuery, AgendaTime};
use super::model::Section;

#[derive(Clone, Copy)]
pub(crate) struct HeadlineTimeSpec {
    pub(crate) start: AgendaTime,
    pub(crate) end: Option<AgendaTime>,
}

pub(crate) fn headline_time<A>(
    section: &Section<A>,
    query: &AgendaQuery,
) -> Option<HeadlineTimeSpec> {
    if query.search_headline_time {
        parse_headline_time(section.raw_title.as_str())
    } else {
        None
    }
}

fn parse_headline_time(raw_title: &str) -> Option<HeadlineTimeSpec> {
    let bytes = raw_title.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if is_timestamp_like_start(bytes, index) {
            index = skip_timestamp_like(bytes, index);
            continue;
        }
        if let Some((start, end, _)) = parse_time_range_at(bytes, index) {
            return Some(HeadlineTimeSpec { start, end });
        }
        index += 1;
    }

    None
}

fn parse_time_range_at(
    bytes: &[u8],
    index: usize,
) -> Option<(AgendaTime, Option<AgendaTime>, usize)> {
    if !is_ascii_word_boundary_before(bytes, index) {
        return None;
    }
    let (start, mut next_index) = parse_time_at(bytes, index)?;
    let mut end = None;

    if next_index < bytes.len() && bytes[next_index] == b'-' {
        let mut range_index = next_index + 1;
        if range_index < bytes.len() && bytes[range_index] == b'-' {
            range_index += 1;
        }
        if let Some((end_time, after_end)) = parse_time_at(bytes, range_index) {
            end = Some(end_time);
            next_index = after_end;
        }
    }

    Some((start, end, next_index))
}

fn parse_time_at(bytes: &[u8], index: usize) -> Option<(AgendaTime, usize)> {
    let mut cursor = index;
    let first = *bytes.get(cursor)?;
    if !first.is_ascii_digit() {
        return None;
    }

    let mut hour = first - b'0';
    cursor += 1;
    if first <= b'2' && cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
        hour = hour.saturating_mul(10).saturating_add(bytes[cursor] - b'0');
        cursor += 1;
    }

    let mut minute = 0;
    let mut am_pm = None;
    if cursor < bytes.len() && bytes[cursor] == b':' {
        let tens = *bytes.get(cursor + 1)?;
        let ones = *bytes.get(cursor + 2)?;
        if !matches!(tens, b'0'..=b'5') || !ones.is_ascii_digit() {
            return None;
        }
        minute = (tens - b'0') * 10 + (ones - b'0');
        cursor += 3;
        if let Some(marker) = parse_am_pm(bytes, cursor) {
            am_pm = Some(marker.0);
            cursor = marker.1;
        }
    } else if let Some(marker) = parse_am_pm(bytes, cursor) {
        am_pm = Some(marker.0);
        cursor = marker.1;
    } else {
        return None;
    }

    if !is_ascii_word_boundary_after(bytes, cursor) {
        return None;
    }

    Some((
        AgendaTime {
            hour: normalize_hour(hour, am_pm),
            minute,
        },
        cursor,
    ))
}

fn parse_am_pm(bytes: &[u8], index: usize) -> Option<(u8, usize)> {
    let first = bytes.get(index)?.to_ascii_lowercase();
    let second = bytes.get(index + 1)?.to_ascii_lowercase();
    match (first, second) {
        (b'a', b'm') => Some((b'a', index + 2)),
        (b'p', b'm') => Some((b'p', index + 2)),
        _ => None,
    }
}

fn normalize_hour(hour: u8, am_pm: Option<u8>) -> u8 {
    match am_pm {
        Some(b'a') if hour == 12 => 0,
        Some(b'a') => hour,
        Some(b'p') if hour == 12 => 12,
        Some(b'p') => hour.saturating_add(12),
        _ => hour,
    }
}

fn is_timestamp_like_start(bytes: &[u8], index: usize) -> bool {
    match bytes.get(index) {
        Some(b'<') => bytes
            .get(index + 1)
            .is_some_and(|next| next.is_ascii_digit() || *next == b'%'),
        Some(b'[') => bytes
            .get(index + 1)
            .is_some_and(|next| next.is_ascii_digit()),
        _ => false,
    }
}

fn skip_timestamp_like(bytes: &[u8], index: usize) -> usize {
    let closing = if bytes[index] == b'<' { b'>' } else { b']' };
    bytes[index + 1..]
        .iter()
        .position(|byte| *byte == closing)
        .map_or(index + 1, |position| index + position + 2)
}

fn is_ascii_word_boundary_before(bytes: &[u8], index: usize) -> bool {
    index == 0 || !bytes[index - 1].is_ascii_alphanumeric()
}

fn is_ascii_word_boundary_after(bytes: &[u8], index: usize) -> bool {
    index >= bytes.len() || !bytes[index].is_ascii_alphanumeric()
}
