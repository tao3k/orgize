//! Radio link projection helpers for semantic object runs.

use super::{ObjectData, ParsedAnnotation};

pub(super) fn next_radio_link<'a>(
    value: &str,
    cursor: usize,
    targets: &'a [String],
) -> Option<(usize, usize, &'a str)> {
    for (relative_start, _) in value[cursor..].char_indices() {
        let start = cursor + relative_start;
        if !is_radio_link_start_boundary(value, start) {
            continue;
        }

        let mut best: Option<(usize, &'a str)> = None;
        for target in targets {
            let end = start + target.len();
            if !value[start..].starts_with(target) || !is_radio_link_end_boundary(value, end) {
                continue;
            }

            if best.is_none_or(|(best_end, _)| end > best_end) {
                best = Some((end, target.as_str()));
            }
        }

        if let Some((end, target)) = best {
            return Some((start, end, target));
        }
    }

    None
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

fn is_radio_link_start_boundary(value: &str, start: usize) -> bool {
    let before = value[..start].chars().next_back();
    !before.is_some_and(is_radio_link_word_char)
}

fn is_radio_link_end_boundary(value: &str, end: usize) -> bool {
    let after = value[end..].chars().next();
    !after.is_some_and(is_radio_link_word_char)
}

fn is_radio_link_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '-'
}
