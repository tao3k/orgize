//! Radio link projection helpers for semantic object runs.

use super::{ObjectData, ParsedAnnotation};

pub(super) fn next_radio_link<'a>(
    value: &str,
    cursor: usize,
    targets: &'a [String],
) -> Option<(usize, usize, &'a str)> {
    let mut best: Option<(usize, usize, &'a str)> = None;

    for target in targets {
        for (relative_start, _) in value[cursor..].match_indices(target) {
            let start = cursor + relative_start;
            let end = start + target.len();
            if !is_radio_link_boundary(value, start, end) {
                continue;
            }

            let candidate = (start, end, target.as_str());
            if best.as_ref().is_none_or(|(best_start, best_end, _)| {
                start < *best_start || (start == *best_start && end > *best_end)
            }) {
                best = Some(candidate);
            }
            break;
        }
    }

    best
}

pub(super) fn is_semantic_radio_link_candidate(data: &ObjectData<ParsedAnnotation>) -> bool {
    matches!(
        data,
        ObjectData::Plain(_)
            | ObjectData::Markup { .. }
            | ObjectData::Code(_)
            | ObjectData::Verbatim(_)
            | ObjectData::Entity(_)
            | ObjectData::LatexFragment(_)
    )
}

pub(super) fn next_char_boundary(value: &str, index: usize) -> usize {
    value[index..]
        .chars()
        .next()
        .map(|ch| index + ch.len_utf8())
        .unwrap_or(value.len())
}

fn is_radio_link_boundary(value: &str, start: usize, end: usize) -> bool {
    let before = value[..start].chars().next_back();
    let after = value[end..].chars().next();
    !before.is_some_and(is_radio_link_word_char) && !after.is_some_and(is_radio_link_word_char)
}

fn is_radio_link_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '-'
}
