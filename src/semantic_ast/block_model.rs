//! Semantic block line, switch, and fixed-width model.

/// One source, example, or fixed-width content line after Org comma-unquoting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockLine<A = ()> {
    pub ann: A,
    /// One-based line number inside the semantic block value.
    pub number: usize,
    /// Source-backed line text before comma-unquoting and without the line ending.
    pub source: String,
    /// Semantic line text without the line ending.
    pub value: String,
    /// Semantic line text after tab expansion and common-indent normalization.
    pub normalized_value: String,
    /// Semantic line text with a trailing code-reference cookie removed.
    pub value_without_code_ref: String,
    /// Normalized line text with a trailing code-reference cookie removed.
    pub normalized_value_without_code_ref: String,
    /// Number of leading spaces removed from the expanded line.
    pub removed_indent: usize,
    /// Original line ending, when this line ended before the block end marker.
    pub line_ending: Option<String>,
    /// Code reference cookie found on this line.
    pub code_ref: Option<BlockCodeRef>,
}

/// Parsed source/example block switch surface.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlockSwitches {
    pub raw: Option<String>,
    pub line_numbering: Option<BlockLineNumbering>,
    pub preserve_indentation: bool,
    pub keep_labels: bool,
    pub remove_labels: bool,
    pub label_format: Option<String>,
}

/// Line-numbering switch metadata for source and example blocks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockLineNumbering {
    pub mode: BlockLineNumberMode,
    pub start: Option<usize>,
}

/// Org source/example block line-numbering mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockLineNumberMode {
    /// Start a fresh numbered listing with `-n`.
    New,
    /// Continue from the previous numbered listing with `+n`.
    Continued,
}

/// Code reference cookie found inside a source or example block line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockCodeRef {
    /// One-based line number inside the block value.
    pub line: usize,
    /// One-based column where the reference cookie starts inside the line.
    pub column: usize,
    /// One-based column immediately after the reference cookie.
    pub end_column: usize,
    /// Reference name extracted from the active label format.
    pub name: String,
    /// Raw reference cookie as it appears in the block line.
    pub raw: String,
}

/// Fixed-width area projected with source/value line metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticFixedWidth<A = ()> {
    pub value: String,
    pub lines: Vec<BlockLine<A>>,
}

impl<A> SemanticFixedWidth<A> {
    pub fn normalized_value(&self) -> String {
        joined_block_lines(&self.lines, |line| line.normalized_value.as_str())
    }
}

pub(super) fn joined_block_lines<A, F>(lines: &[BlockLine<A>], value: F) -> String
where
    F: for<'a> Fn(&'a BlockLine<A>) -> &'a str,
{
    let mut joined = String::new();
    for line in lines {
        joined.push_str(value(line));
        if let Some(ending) = &line.line_ending {
            joined.push_str(ending);
        }
    }
    joined
}

/// Header argument parsed from a source block parameter string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockHeaderArg {
    /// Header argument key without the leading colon.
    pub key: String,
    /// Header argument value, if present, preserving inner spacing.
    pub value: Option<String>,
    /// Raw header argument fragment as it appears in the begin line.
    pub raw: String,
}
