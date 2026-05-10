//! Owned semantic AST for Org documents.
//!
//! The parser still builds the lossless rowan syntax tree. This module is the
//! semantic, org-element-like layer projected from that syntax tree.

mod block_metadata;
mod conversion;
mod macro_expansion;
mod model;
mod targets;
mod traversal;

pub use model::{
    AstMut, AstRef, BareAst, Block, BlockCodeRef, BlockHeaderArg, BlockKind, BlockLineNumberMode,
    BlockLineNumbering, Checkbox, Citation, CiteReference, Clock, Diagnostic, DiagnosticKind,
    Document, Drawer, Element, ElementData, FootnoteDef, IncludeDirective, IncludeOption, Keyword,
    Link, LinkTarget, List, ListItem, ListType, MacroDefinition, MacroExpansion,
    MacroExpansionStatus, MarkupKind, Object, ObjectData, ParsedAnnotation, ParsedAst, Planning,
    Property, RepeaterKind, Section, SourcePosition, Table, TableCell, TableColumnAlignment,
    TableRow, TargetDefinition, TargetKind, TimeUnit, Timestamp, TimestampKind, TimestampMoment,
    TimestampRepeater, TimestampWarning, TodoKeyword, TodoState, WarningKind,
};
