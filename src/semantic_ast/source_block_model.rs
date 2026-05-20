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
    pub result_options: SourceBlockResultOptions,
    pub execution: SourceBlockExecutionPlan,
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
    File,
    FileDesc,
    FileExt,
    FileMode,
    Hlines,
    Noweb,
    OutputDir,
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
    pub mkdirp: SourceBlockTangleMkdirp,
    pub comments: SourceBlockTangleComments,
    pub shebang: Option<String>,
    pub noweb: SourceBlockTangleNoweb,
}

/// Normalized `:tangle` mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBlockTangleMode {
    Yes,
    No,
    File,
}

/// Parsed `:mkdirp` tangle metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockTangleMkdirp {
    pub raw: String,
    pub enabled: bool,
}

/// Parsed `:comments` tangle metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockTangleComments {
    pub raw: String,
    pub mode: SourceBlockTangleCommentsMode,
}

/// Normalized `:comments` mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBlockTangleCommentsMode {
    No,
    Link,
    Yes,
    Org,
    Both,
    Noweb,
    Other,
}

/// Parsed `:noweb` metadata for the tangle context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockTangleNoweb {
    pub raw: String,
    pub mode: SourceBlockTangleNowebMode,
}

/// Normalized `:noweb` behavior in the tangle context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBlockTangleNowebMode {
    Disabled,
    Expand,
    Strip,
}

/// Parsed result-handling metadata from Org Babel `:results` and file headers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockResultOptions {
    pub raw: String,
    pub source: SourceBlockHeaderArgSource,
    pub tokens: Vec<String>,
    pub collection: Option<SourceBlockResultCollection>,
    pub format: Option<SourceBlockResultFormat>,
    pub handling: SourceBlockResultHandling,
    pub value_type: SourceBlockResultValueType,
    pub unknown: Vec<String>,
    pub file: Option<SourceBlockResultFile>,
}

/// Result collection shape selected by `:results`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBlockResultCollection {
    File,
    List,
    Vector,
    Table,
    Scalar,
    Verbatim,
}

/// Result rendering format selected by `:results`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBlockResultFormat {
    Raw,
    Html,
    Latex,
    Org,
    Code,
    Pp,
    Drawer,
    Link,
    Graphics,
}

/// Result insertion strategy selected by `:results`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBlockResultHandling {
    Replace,
    Silent,
    None,
    Discard,
    Append,
    Prepend,
}

/// Whether Babel should treat the result as a value or output stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBlockResultValueType {
    Value,
    Output,
}

/// File-output metadata from `:file` and related header arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockResultFile {
    pub target: String,
    pub description: Option<String>,
    pub extension: Option<String>,
    pub file_mode: Option<SourceBlockResultFileMode>,
    pub output_dir: Option<String>,
}

/// Raw `:file-mode` permission hint preserved without applying it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockResultFileMode {
    pub raw: String,
}

/// Non-executing execution/export planning metadata from common Babel headers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockExecutionPlan {
    pub eval: SourceBlockEval,
    pub exports: SourceBlockExports,
    pub cache: SourceBlockCache,
    pub session: SourceBlockSession,
    pub directory: Option<SourceBlockDirectory>,
    pub hlines: SourceBlockBooleanHeader,
    pub noweb: SourceBlockNowebPlan,
}

/// Parsed `:eval` policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockEval {
    pub raw: String,
    pub source: SourceBlockHeaderArgSource,
    pub policy: SourceBlockEvalPolicy,
}

/// Org Babel evaluation policies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBlockEvalPolicy {
    Yes,
    No,
    NoExport,
    StripExport,
    NeverExport,
    Eval,
    Never,
    Query,
    Other,
}

/// Parsed `:exports` policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockExports {
    pub raw: String,
    pub source: SourceBlockHeaderArgSource,
    pub policy: SourceBlockExportsPolicy,
}

/// Org Babel export inclusion policies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBlockExportsPolicy {
    Code,
    Results,
    Both,
    None,
    Other,
}

/// Parsed `:cache` metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockCache {
    pub raw: String,
    pub source: SourceBlockHeaderArgSource,
    pub enabled: bool,
}

/// Parsed `:session` metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockSession {
    pub raw: String,
    pub source: SourceBlockHeaderArgSource,
    pub name: Option<String>,
    pub active: bool,
}

/// Parsed `:dir` metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockDirectory {
    pub raw: String,
    pub source: SourceBlockHeaderArgSource,
    pub target: String,
    pub kind: SourceBlockDirectoryKind,
}

/// Directory target class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBlockDirectoryKind {
    Path,
    Attachment,
}

/// Parsed boolean-ish Babel header metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockBooleanHeader {
    pub raw: String,
    pub source: SourceBlockHeaderArgSource,
    pub enabled: bool,
}

/// Parsed `:noweb` behavior by execution context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlockNowebPlan {
    pub raw: String,
    pub source: SourceBlockHeaderArgSource,
    pub tokens: Vec<String>,
    pub eval: SourceBlockNowebAction,
    pub export: SourceBlockNowebAction,
    pub tangle: SourceBlockNowebAction,
}

/// Noweb behavior in one context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBlockNowebAction {
    Disabled,
    Expand,
    Strip,
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
