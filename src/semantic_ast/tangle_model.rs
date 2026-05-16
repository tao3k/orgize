//! Safe source tangle planning records.

use super::{
    SectionIndexSource, SourceBlockHeaderArg, SourceBlockSource, SourceBlockTangleMode,
    TableFormula,
};

/// Options for safe source tangle planning.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceTangleOptions {
    pub default_stem: Option<String>,
}

impl SourceTangleOptions {
    /// Creates tangle planning options with an explicit source-file stem.
    pub fn with_default_stem(default_stem: impl Into<String>) -> Self {
        Self {
            default_stem: Some(default_stem.into()),
        }
    }
}

/// A non-executing plan for writing tangled source files.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceTanglePlan {
    pub files: Vec<SourceTangleFile>,
    pub skipped: Vec<SourceTangleSkip>,
}

/// Blocks grouped by a target tangle path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceTangleFile {
    pub target: String,
    pub blocks: Vec<SourceTangleBlock>,
}

/// One source block selected for tangle extraction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceTangleBlock {
    pub source: SourceBlockSource,
    pub name: Option<String>,
    pub language: Option<String>,
    pub header_args: Vec<SourceBlockHeaderArg>,
    pub value: String,
}

/// One source block intentionally not included in the tangle plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceTangleSkip {
    pub source: SourceBlockSource,
    pub name: Option<String>,
    pub language: Option<String>,
    pub mode: Option<SourceBlockTangleMode>,
    pub reason: SourceTangleSkipReason,
}

/// Why a source block was excluded from a safe tangle plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceTangleSkipReason {
    InlineSource,
    Disabled,
    MissingTarget,
}

/// Document-local table formula side-table record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableFormulaRecord<A = ()> {
    pub source: SectionIndexSource,
    pub row_count: usize,
    pub column_count: usize,
    pub formulas: Vec<TableFormula<A>>,
}
