//! Source selector parsing and bounded line extraction for document commands.

use std::path::PathBuf;

/// Source file plus an optional inclusive 1-based line range.
pub struct SourceSelector {
    pub path: PathBuf,
    pub range: Option<SourceLineRange>,
}

/// Inclusive 1-based line range selected from one source file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceLineRange {
    pub start_line: usize,
    pub end_line: usize,
}

impl SourceLineRange {
    pub fn new(start_line: usize, end_line: usize) -> Self {
        Self {
            start_line,
            end_line: end_line.max(start_line),
        }
    }
}

impl SourceSelector {
    pub fn parse(selector: &str) -> Result<Self, String> {
        let Some((path, range)) = selector.rsplit_once(':') else {
            return Ok(Self {
                path: PathBuf::from(selector),
                range: None,
            });
        };
        if path.is_empty() {
            return Err(format!("invalid selector `{selector}`"));
        }
        Ok(Self {
            path: PathBuf::from(path),
            range: Some(parse_line_range(range)?),
        })
    }
}

/// Select the requested inclusive line range from source text.
pub fn select_source(source: &str, range: Option<SourceLineRange>) -> String {
    let Some(range) = range else {
        return source.to_string();
    };
    let mut output = String::new();
    for (index, line) in source.split_inclusive('\n').enumerate() {
        let line_no = index + 1;
        if line_no >= range.start_line && line_no <= range.end_line {
            output.push_str(line);
        }
    }
    output
}

fn parse_line_range(value: &str) -> Result<SourceLineRange, String> {
    let (start, end) = value.split_once('-').unwrap_or((value, value));
    let start = start
        .parse::<usize>()
        .map_err(|_| format!("invalid selector line `{value}`"))?;
    let end = end
        .parse::<usize>()
        .map_err(|_| format!("invalid selector line `{value}`"))?;
    Ok(SourceLineRange::new(start, end))
}
