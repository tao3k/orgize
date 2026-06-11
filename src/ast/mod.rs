//! Lossless typed syntax wrappers projected from the rowan syntax tree.
//!
//! This module owns the legacy wrapper surface re-exported as `orgize::syntax_ast`.

mod generated;

mod affiliated_keyword;
mod block;
mod clock;
#[cfg(feature = "syntax-org-fc")]
mod cloze;
mod comment;
mod document;
mod drawer;
mod entity;
mod fixed_width;
mod headline;
mod inline_call;
mod inline_src;
mod keyword;
mod link;
mod list;
mod macros;
mod planning;
mod snippet;
mod support;
mod table;
mod timestamp;

use crate::syntax::SyntaxKind;

#[cfg(feature = "syntax-org-fc")]
pub use cloze::Cloze;
pub use generated::{
    AffiliatedKeyword, BabelCall, Bold, CenterBlock, Code, Comment, CommentBlock, Cookie,
    DiarySexp, DynBlock, Entity, ExampleBlock, ExportBlock, FixedWidth, FnDef, FnRef, Headline,
    InlineCall, InlineSrc, Italic, LatexEnvironment, LatexFragment, LineBreak, Macros,
    NodeProperty, OrgTable, OrgTableCell, OrgTableRow, Paragraph, PropertyDrawer, QuoteBlock,
    RadioTarget, Rule, Snippet, SourceBlock, SpecialBlock, Strike, Subscript, Superscript,
    SyntaxCitation, SyntaxClock, SyntaxDocument, SyntaxDrawer, SyntaxInlinetask,
    SyntaxInlinetaskEnd, SyntaxKeyword, SyntaxLink, SyntaxList, SyntaxListItem, SyntaxPlanning,
    SyntaxSection, SyntaxTimestamp, TableEl, Target, Underline, Verbatim, VerseBlock,
};
pub use headline::TodoType;
pub use support::Token;
use support::{blank_lines, filter_token, last_child, last_token, token};
pub use timestamp::{DelayType, RepeaterType, SyntaxTimeUnit};
