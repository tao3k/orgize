//! Semantic timestamp metadata projection helpers.

use crate::{syntax::SyntaxKind, syntax::SyntaxNode, syntax_ast};

use super::{
    RepeaterKind, TimeUnit, TimestampMoment, TimestampRepeater, TimestampWarning, WarningKind,
};

use super::conversion_util::parse_token;

#[derive(Default)]
struct TimestampMomentBuilder {
    year: Option<u16>,
    month: Option<u8>,
    day: Option<u8>,
    day_name: Option<String>,
    times: Vec<(u8, u8)>,
    pending_hour: Option<u8>,
}

impl TimestampMomentBuilder {
    fn is_empty(&self) -> bool {
        self.year.is_none()
            && self.month.is_none()
            && self.day.is_none()
            && self.day_name.is_none()
            && self.times.is_empty()
            && self.pending_hour.is_none()
    }

    fn to_moment(&self, time_index: usize) -> Option<TimestampMoment> {
        let (hour, minute) = self
            .times
            .get(time_index)
            .copied()
            .map(|(hour, minute)| (Some(hour), Some(minute)))
            .unwrap_or((None, None));

        Some(TimestampMoment {
            year: self.year?,
            month: self.month?,
            day: self.day?,
            day_name: self.day_name.clone(),
            hour,
            minute,
        })
    }
}

pub(super) fn timestamp_moment_range(
    node: &SyntaxNode,
    is_range: bool,
) -> (Option<TimestampMoment>, Option<TimestampMoment>) {
    let moments = timestamp_moment_builders(node);
    let start = moments.first().and_then(|moment| moment.to_moment(0));
    let end = if is_range {
        if moments.len() > 1 {
            moments.last().and_then(|moment| moment.to_moment(0))
        } else {
            moments.first().and_then(|moment| moment.to_moment(1))
        }
    } else {
        None
    };

    (start, end)
}

fn timestamp_moment_builders(node: &SyntaxNode) -> Vec<TimestampMomentBuilder> {
    let mut moments = Vec::new();
    let mut current = TimestampMomentBuilder::default();

    for token in node
        .children_with_tokens()
        .filter_map(|element| element.into_token())
    {
        match token.kind() {
            SyntaxKind::TIMESTAMP_YEAR => {
                if !current.is_empty() {
                    moments.push(current);
                    current = TimestampMomentBuilder::default();
                }
                current.year = parse_token(token.text());
            }
            SyntaxKind::TIMESTAMP_MONTH => current.month = parse_token(token.text()),
            SyntaxKind::TIMESTAMP_DAY => current.day = parse_token(token.text()),
            SyntaxKind::TIMESTAMP_DAYNAME => current.day_name = Some(token.text().to_string()),
            SyntaxKind::TIMESTAMP_HOUR => current.pending_hour = parse_token(token.text()),
            SyntaxKind::TIMESTAMP_MINUTE => {
                if let (Some(hour), Some(minute)) =
                    (current.pending_hour.take(), parse_token(token.text()))
                {
                    current.times.push((hour, minute));
                }
            }
            _ => {}
        }
    }

    if !current.is_empty() {
        moments.push(current);
    }

    moments
}

pub(super) fn timestamp_repeater(
    timestamp: &syntax_ast::SyntaxTimestamp,
) -> Option<TimestampRepeater> {
    Some(TimestampRepeater {
        kind: match timestamp.repeater_type()? {
            syntax_ast::RepeaterType::Cumulate => RepeaterKind::Cumulate,
            syntax_ast::RepeaterType::CatchUp => RepeaterKind::CatchUp,
            syntax_ast::RepeaterType::Restart => RepeaterKind::Restart,
        },
        value: timestamp.repeater_value()?,
        unit: timestamp_time_unit(timestamp.repeater_unit()?),
    })
}

pub(super) fn timestamp_warning(
    timestamp: &syntax_ast::SyntaxTimestamp,
) -> Option<TimestampWarning> {
    Some(TimestampWarning {
        kind: match timestamp.warning_type()? {
            syntax_ast::DelayType::All => WarningKind::All,
            syntax_ast::DelayType::First => WarningKind::First,
        },
        value: timestamp.warning_value()?,
        unit: timestamp_time_unit(timestamp.warning_unit()?),
    })
}

fn timestamp_time_unit(unit: syntax_ast::SyntaxTimeUnit) -> TimeUnit {
    match unit {
        syntax_ast::SyntaxTimeUnit::Hour => TimeUnit::Hour,
        syntax_ast::SyntaxTimeUnit::Day => TimeUnit::Day,
        syntax_ast::SyntaxTimeUnit::Week => TimeUnit::Week,
        syntax_ast::SyntaxTimeUnit::Month => TimeUnit::Month,
        syntax_ast::SyntaxTimeUnit::Year => TimeUnit::Year,
    }
}
