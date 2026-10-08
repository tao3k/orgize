//! Typed projection of Scheme-classified source-block switches.

use gerbil_parser_runtime::GraphRecord;

use super::block_model::{BlockHeaderArg, BlockLineNumberMode, BlockLineNumbering, BlockSwitches};

pub(super) fn project_block_switches(
    record: &GraphRecord,
    source: &str,
) -> (Option<String>, BlockSwitches) {
    let mut options = BlockSwitches::default();
    for field in &record.fields {
        match field.name {
            "switch-new-line-start" | "switch-continued-line-start" => {
                options.line_numbering = Some(BlockLineNumbering {
                    mode: if field.name == "switch-new-line-start" {
                        BlockLineNumberMode::New
                    } else {
                        BlockLineNumberMode::Continued
                    },
                    start: field.value.parse().ok(),
                });
            }
            "switch-label-format" => options.label_format = Some(field.value.clone()),
            "switch-name" => match field.value.as_str() {
                "-i" => options.preserve_indentation = true,
                "-r" => options.remove_labels = true,
                "-k" => options.keep_labels = true,
                "-n" | "+n" => {
                    options.line_numbering = Some(BlockLineNumbering {
                        mode: if field.value == "-n" {
                            BlockLineNumberMode::New
                        } else {
                            BlockLineNumberMode::Continued
                        },
                        start: None,
                    });
                }
                "-l" => options.label_format = None,
                _ => unreachable!("Scheme emits only declared source switches"),
            },
            _ => {}
        }
    }
    // The complete extent is declared by the Scheme graph projection, not
    // reconstructed from neighbouring names, values or header syntax here.
    let raw = record
        .field_range("switches")
        .map(|range| source[usize::from(range.start())..usize::from(range.end())].to_owned());
    options.raw = raw.clone();
    (raw, options)
}

pub(super) fn project_block_parameters(record: &GraphRecord) -> Option<String> {
    record.field("header-parameters").map(str::to_owned)
}

/// Decode the native argument stream in its declared source order.
pub(super) fn project_block_header_args(record: &GraphRecord) -> Vec<BlockHeaderArg> {
    project_header_args(record, "argument-value")
}

/// Keyword attributes and INCLUDE options decode native lexical content.
pub(super) fn project_keyword_header_args(record: &GraphRecord) -> Vec<BlockHeaderArg> {
    project_header_args(record, "argument-content")
}

fn project_header_args(record: &GraphRecord, value_field: &str) -> Vec<BlockHeaderArg> {
    let mut args = Vec::new();
    let mut key = None;
    let mut value = None;
    for field in &record.fields {
        match field.name {
            "argument-key" => key = Some(field.value.clone()),
            name if name == value_field => value = Some(field.value.clone()),
            "argument-raw" => {
                args.push(BlockHeaderArg {
                    key: key.take().expect("native header argument key"),
                    value: value.take(),
                    raw: field.value.clone(),
                });
            }
            _ => {}
        }
    }
    args
}
