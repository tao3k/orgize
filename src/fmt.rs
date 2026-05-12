//! Conservative Org source formatter.

/// Formatter options for [`format_org`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatOptions {
    /// Remove spaces and tabs from line ends.
    pub trim_trailing_whitespace: bool,
    /// Ensure formatted non-empty documents end with one newline.
    pub final_newline: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            trim_trailing_whitespace: true,
            final_newline: true,
        }
    }
}

/// Result of formatting one Org source string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatResult {
    pub output: String,
    pub changed: bool,
}

/// Formats Org source with conservative, source-preserving rules.
///
/// The first formatter lane intentionally avoids semantic rewrites. It only
/// normalizes trailing horizontal whitespace, final blank lines, and final EOF
/// newline shape.
pub fn format_org(source: &str, options: &FormatOptions) -> FormatResult {
    let mut lines = source
        .split('\n')
        .map(|line| {
            let line = line.strip_suffix('\r').unwrap_or(line);
            if options.trim_trailing_whitespace {
                line.trim_end_matches([' ', '\t']).to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>();

    if source.ends_with('\n') {
        lines.pop();
    }

    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }

    let mut output = lines.join("\n");
    if options.final_newline && !output.is_empty() {
        output.push('\n');
    }

    FormatResult {
        changed: output != source,
        output,
    }
}
