//! Priority-cookie lint checks.

use crate::ast::PriorityValue;

use super::lint_model::{location_for_offsets, LintFinding, LintSeverity};

pub(crate) fn priority_cookie_findings(source: &str) -> Vec<LintFinding> {
    source
        .split_inclusive('\n')
        .scan(0, |offset, segment| {
            let current = *offset;
            *offset += segment.len();
            Some((current, segment))
        })
        .filter_map(|(offset, segment)| priority_cookie_finding(source, offset, segment))
        .collect()
}

fn priority_cookie_finding(source: &str, offset: usize, segment: &str) -> Option<LintFinding> {
    let line = segment.trim_end_matches('\n').trim_end_matches('\r');
    let trimmed_start = line.len() - line.trim_start_matches([' ', '\t']).len();
    let trimmed = &line[trimmed_start..];
    let (start, end, message) = malformed_priority_cookie(trimmed)?;
    Some(LintFinding {
        code: "ORG010",
        severity: LintSeverity::Warning,
        message,
        location: location_for_offsets(
            source,
            offset + trimmed_start + start,
            offset + trimmed_start + end,
        ),
    })
}

fn malformed_priority_cookie(line: &str) -> Option<(usize, usize, String)> {
    let bytes = line.as_bytes();
    let stars = bytes.iter().take_while(|byte| **byte == b'*').count();
    if stars == 0
        || !bytes
            .get(stars)
            .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        return None;
    }

    let first = next_token(line, stars)?;
    if let Some(finding) = malformed_priority_token(line, first) {
        return Some(finding);
    }

    let second = next_token(line, first.1)?;
    malformed_priority_token(line, second)
}

fn malformed_priority_token(line: &str, token: (usize, usize)) -> Option<(usize, usize, String)> {
    let raw = &line[token.0..token.1];
    if !raw.starts_with("[#") {
        return None;
    }
    let Some(close) = raw.find(']') else {
        return Some((
            token.0,
            token.1,
            "priority cookie is missing a closing `]`".to_string(),
        ));
    };
    if close + 1 != raw.len() {
        return Some((
            token.0,
            token.1,
            format!("priority cookie `{raw}` has trailing text after `]`"),
        ));
    }
    let value = &raw[2..close];
    if PriorityValue::parse(value).is_some() {
        return None;
    }
    Some((
        token.0,
        token.1,
        format!("priority cookie `{raw}` is not a supported Org priority value"),
    ))
}

fn next_token(line: &str, start: usize) -> Option<(usize, usize)> {
    let token_start = line[start..]
        .char_indices()
        .find_map(|(offset, ch)| (!ch.is_whitespace()).then_some(start + offset))?;
    let token_end = line[token_start..]
        .char_indices()
        .find(|(_, ch)| ch.is_whitespace())
        .map_or(line.len(), |(offset, _)| token_start + offset);
    Some((token_start, token_end))
}
