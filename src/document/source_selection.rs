//! Source selector parsing and bounded line extraction for document commands.

use std::path::PathBuf;

/// Source file plus its parser-owned structural selector.
#[derive(Debug)]
pub struct SourceSelector {
    pub path: PathBuf,
    pub range: Option<SourceLineRange>,
    pub structural_selector: Option<String>,
    pub structural_fragment: Option<String>,
}

impl SourceSelector {
    /// Parse the legacy direct-read `path[:start-end]` selector.
    pub fn parse_direct_read(value: &str) -> Result<Self, String> {
        let (path, range) = match value.rsplit_once(':') {
            Some((path, candidate)) => match candidate.split_once('-') {
                Some((start, end)) => {
                    let start_line = start
                        .parse::<usize>()
                        .map_err(|_| format!("invalid source line range `{candidate}`"))?;
                    let end_line = end
                        .parse::<usize>()
                        .map_err(|_| format!("invalid source line range `{candidate}`"))?;
                    (path, Some(SourceLineRange::new(start_line, end_line)))
                }
                None => (value, None),
            },
            None => (value, None),
        };
        if path.is_empty() {
            return Err("source selector path must not be empty".to_owned());
        }
        Ok(Self {
            path: PathBuf::from(path),
            range,
            structural_selector: None,
            structural_fragment: None,
        })
    }
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
    pub fn parse_query(selector: &str) -> Result<Self, String> {
        if !selector.contains("://") {
            return Err(format!(
                "query selector `{selector}` is not a parser-owned structural selector"
            ));
        }
        SourceSelector::parse_structural(selector)
    }

    pub fn parse_structural(selector: &str) -> Result<Self, String> {
        if let Some((scheme, rest)) = selector.split_once("://") {
            if !matches!(scheme, "org" | "md") {
                return Err(format!("invalid document selector scheme `{scheme}`"));
            }
            let Some((path, fragment)) = rest.split_once('#') else {
                return Err(format!("invalid structural selector `{selector}`"));
            };
            if path.is_empty() || fragment.is_empty() {
                return Err(format!("invalid structural selector `{selector}`"));
            }
            return Ok(Self {
                path: PathBuf::from(path),
                range: None,
                structural_selector: Some(selector.to_string()),
                structural_fragment: Some(fragment.to_string()),
            });
        }
        Err(format!(
            "document query selector `{selector}` is not structural; use a selector emitted by search/query metadata, for example org://path#structure"
        ))
    }
}

pub fn structural_selector_fragment(selector: &str) -> &str {
    selector
        .split_once('#')
        .map(|(_, fragment)| fragment)
        .unwrap_or(selector)
}

/// Select an optional inclusive line range from source text.
pub fn select_source(source: &str, range: impl Into<Option<SourceLineRange>>) -> String {
    let Some(range) = range.into() else {
        return source.to_owned();
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
