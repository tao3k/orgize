//! Typed projection of native Scheme block-line facts, never source scanning.

use super::{BlockCodeRef, BlockLine};

pub(super) fn project_block_lines<A>(
    rows: Vec<Vec<String>>,
    source: &str,
    mut ann_for_range: impl FnMut(usize, usize) -> A,
) -> Vec<BlockLine<A>> {
    rows.into_iter()
        .map(|row| {
            let [
                number,
                raw,
                text,
                normal,
                without,
                normal_without,
                indent,
                ending,
                start,
                end,
                column,
                end_column,
                name,
                reference,
            ]: [String; 14] = row.try_into().expect("native block line row arity");
            let number = native_number(&number);
            let start = native_number(&start);
            let end = native_number(&end);
            assert!(
                start <= end
                    && end <= source.len()
                    && source.is_char_boundary(start)
                    && source.is_char_boundary(end),
                "native block source range"
            );
            BlockLine {
                ann: ann_for_range(start, end),
                number,
                source: raw,
                value: text,
                normalized_value: normal,
                value_without_code_ref: without,
                normalized_value_without_code_ref: normal_without,
                removed_indent: native_number(&indent),
                line_ending: (!ending.is_empty()).then_some(ending),
                code_ref: (!name.is_empty()).then(|| BlockCodeRef {
                    line: number,
                    column: native_number(&column),
                    end_column: native_number(&end_column),
                    name,
                    raw: reference,
                }),
            }
        })
        .collect()
}

pub(super) fn native_number(value: &str) -> usize {
    value.parse().expect("admitted native unsigned value")
}
