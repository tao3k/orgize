//! Source-block side-table records for Babel/tangle-aware indexers.

use super::{
    SourcePosition,
    block_model::{BlockCodeRef, BlockHeaderArg},
    model::ParsedAnnotation,
};

/// One source block projected for indexers and agent tooling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockRecord {
    pub source: SourceBlockSource,
    pub kind: SourceBlockRecordKind,
    pub name: Option<String>,
    pub language: Option<String>,
    pub parameters: Option<String>,
    pub header_args: Vec<BlockHeaderArg>,
    pub normalized_header_args: Vec<SourceBlockHeaderArg>,
    pub code_refs: Vec<BlockCodeRef>,
    pub tangle: Option<SourceBlockTangle>,
    pub result: Option<SourceBlockResult>,
    pub value: String,
}

/// One source-block name reference projected for literate-programming tooling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockReference {
    pub source: SourceBlockSource,
    pub kind: SourceBlockReferenceKind,
    pub variable: Option<String>,
    pub target: String,
    pub resolved: bool,
}

/// Source-block reference syntax aligned with Org Babel entry points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBlockReferenceKind {
    /// `#+CALL: name(...)`.
    BabelCall,
    /// `:var value=name(...)` or `:var value=name` source-block dependency.
    HeaderVar,
    /// `call_name(...)` inline Babel call.
    InlineCall,
    /// `<<name>>` or `<<name(args)>>` noweb reference inside source text.
    Noweb,
}

/// Org source execution construct kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBlockRecordKind {
    /// `#+begin_src` / `#+end_src` block.
    Block,
    /// `src_lang[:headers]{body}` inline source object.
    InlineSource,
}

/// Source location for source-block records and nested source-block evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockSource {
    pub start: SourcePosition,
    pub end: SourcePosition,
    pub range_start: u32,
    pub range_end: u32,
}

impl SourceBlockSource {
    pub(crate) fn from_annotation(annotation: &ParsedAnnotation) -> Self {
        Self {
            start: annotation.start,
            end: annotation.end,
            range_start: annotation.range.start().into(),
            range_end: annotation.range.end().into(),
        }
    }
}

/// Header argument normalized for agent-facing Babel projections.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockHeaderArg {
    pub key: String,
    pub value: Option<String>,
    pub raw: String,
    pub kind: SourceBlockHeaderArgKind,
    pub source: SourceBlockHeaderArgSource,
    pub tokens: Vec<String>,
    pub variable: Option<SourceBlockHeaderVar>,
}

/// Header argument category aligned with Org Babel's common headers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBlockHeaderArgKind {
    Cache,
    Dir,
    Eval,
    Exports,
    Hlines,
    Noweb,
    Results,
    Session,
    Tangle,
    Var,
    Other,
}

/// Whether a header argument came from source text or an Org Babel default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBlockHeaderArgSource {
    Explicit,
    Default,
}

/// Parsed `:var name=value` style binding metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockHeaderVar {
    pub name: String,
    pub assignment: Option<String>,
}

/// Parsed `:tangle` header argument metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockTangle {
    pub raw: String,
    pub mode: SourceBlockTangleMode,
    pub target: Option<String>,
}

/// Normalized `:tangle` mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBlockTangleMode {
    Yes,
    No,
    File,
}

/// Result evidence following a source block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockResult {
    pub source: SourceBlockSource,
    pub kind: SourceBlockResultKind,
    pub hash: Option<String>,
    pub name: Option<String>,
    pub keyword_value: String,
    pub value: String,
}

/// Result syntax attached to source execution output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBlockResultKind {
    Keyword,
    InlineMacro,
}
