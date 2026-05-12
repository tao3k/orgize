//! Generated typed syntax AST wrappers grouped by syntax family.

#![allow(clippy::all)]
#![allow(unused)]

mod blocks;
mod elements;
mod object_core;
mod object_markup;
mod object_misc;
mod root;
mod support;
mod table_list;

pub use blocks::{
    CenterBlock, CommentBlock, ExampleBlock, ExportBlock, QuoteBlock, SourceBlock, SpecialBlock,
    VerseBlock,
};
pub use elements::{
    AffiliatedKeyword, BabelCall, Comment, DynBlock, FixedWidth, FnDef, Rule, SyntaxClock,
    SyntaxInlinetask, SyntaxInlinetaskEnd, SyntaxKeyword, TableEl,
};
pub use object_core::{
    Cookie, FnRef, InlineCall, InlineSrc, Macros, RadioTarget, Snippet, SyntaxCitation, SyntaxLink,
    Target,
};
pub use object_markup::{Bold, Code, Italic, Strike, Underline, Verbatim};
pub use object_misc::{
    Entity, LatexEnvironment, LatexFragment, LineBreak, Subscript, Superscript, SyntaxTimestamp,
};
pub use root::{
    Headline, NodeProperty, Paragraph, PropertyDrawer, SyntaxDocument, SyntaxPlanning,
    SyntaxSection,
};
pub(super) use support::affiliated_keyword;
pub use table_list::{
    OrgTable, OrgTableCell, OrgTableRow, SyntaxDrawer, SyntaxList, SyntaxListItem,
};

pub(super) use super::{blank_lines, last_child, last_token, token, Token};
