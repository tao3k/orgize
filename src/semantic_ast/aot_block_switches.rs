//! Typed projection of Scheme-classified source-block switches.

use gerbil_parser_rowan::{GraphFieldValue, GraphRecord};

use super::block_model::{BlockLineNumberMode, BlockLineNumbering, BlockSwitches};

pub(super) fn project_block_switches(
    record: &GraphRecord,
    source: &str,
) -> (Option<String>, BlockSwitches) {
    let names: Vec<_> = record
        .fields
        .iter()
        .filter(|field| field.name == "switch-name")
        .collect();
    let mut options = BlockSwitches::default();
    let Some(first) = names.first() else {
        return (None, options);
    };
    let mut end = usize::from(first.range.end());
    for (index, name) in names.iter().enumerate() {
        let next_start = names
            .get(index + 1)
            .map_or(usize::MAX, |next| usize::from(next.range.start()));
        let value = switch_value(record, name, next_start);
        end = end.max(usize::from(name.range.end()));
        if let Some(value) = value {
            end = end.max(usize::from(value.range.end()));
        }
        match name.value.as_str() {
            "-i" => options.preserve_indentation = true,
            "-r" => options.remove_labels = true,
            "-k" => options.keep_labels = true,
            "-n" | "+n" => {
                options.line_numbering = Some(BlockLineNumbering {
                    mode: if name.value == "-n" {
                        BlockLineNumberMode::New
                    } else {
                        BlockLineNumberMode::Continued
                    },
                    start: value.and_then(|value| value.value.parse().ok()),
                });
            }
            "-l" => {
                options.label_format = value.map(|value| {
                    value
                        .value
                        .strip_prefix('"')
                        .and_then(|text| text.strip_suffix('"'))
                        .unwrap_or(&value.value)
                        .to_owned()
                });
            }
            _ => unreachable!("Scheme emits only declared source switches"),
        }
    }
    let start = usize::from(first.range.start());
    let raw = source[start..end].to_owned();
    options.raw = Some(raw.clone());
    (Some(raw), options)
}

fn switch_value<'a>(
    record: &'a GraphRecord,
    name: &GraphFieldValue,
    next_start: usize,
) -> Option<&'a GraphFieldValue> {
    record.fields.iter().find(|field| {
        field.name == "switch-value"
            && field.range.start() >= name.range.end()
            && usize::from(field.range.start()) < next_start
    })
}

pub(super) fn project_block_parameters(record: &GraphRecord, source: &str) -> Option<String> {
    let key_start = usize::from(record.field_range("header-key")?.start());
    let end = usize::from(record.field_range("header")?.end());
    source
        .get(key_start.checked_sub(1)?..end)
        .map(str::trim)
        .map(str::to_owned)
}
