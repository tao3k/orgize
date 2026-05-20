//! Generated typed AST wrappers for one syntax family.

use crate::syntax::{OrgLanguage, SyntaxKind, SyntaxNode, SyntaxToken};
use rowan::{
    TextRange, TextSize,
    ast::{AstChildren, AstNode, support},
};

use super::affiliated_keyword;

/// Typed syntax wrapper for `DynBlock` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DynBlock {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for DynBlock {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::DYN_BLOCK
    }
    fn cast(node: SyntaxNode) -> Option<DynBlock> {
        Self::can_cast(node.kind()).then(|| DynBlock { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl DynBlock {
    /// Beginning position of this element
    pub fn start(&self) -> TextSize {
        self.syntax.text_range().start()
    }
    /// Ending position of this element
    pub fn end(&self) -> TextSize {
        self.syntax.text_range().end()
    }
    /// Range of this element
    pub fn text_range(&self) -> TextRange {
        self.syntax.text_range()
    }
    /// Raw text of this element
    pub fn raw(&self) -> String {
        self.syntax.to_string()
    }
    pub fn caption(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "CAPTION")
    }
    pub fn header(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "HEADER")
    }
    pub fn name(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "NAME")
    }
    pub fn plot(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "PLOT")
    }
    pub fn results(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "RESULTS")
    }
    pub fn attr(&self, backend: &str) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| {
            k.starts_with("ATTR_") && &k[5..] == backend
        })
    }
}
/// Typed syntax wrapper for `SyntaxInlinetask` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyntaxInlinetask {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for SyntaxInlinetask {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::INLINETASK
    }
    fn cast(node: SyntaxNode) -> Option<SyntaxInlinetask> {
        Self::can_cast(node.kind()).then(|| SyntaxInlinetask { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl SyntaxInlinetask {
    /// Beginning position of this element
    pub fn start(&self) -> TextSize {
        self.syntax.text_range().start()
    }
    /// Ending position of this element
    pub fn end(&self) -> TextSize {
        self.syntax.text_range().end()
    }
    /// Range of this element
    pub fn text_range(&self) -> TextRange {
        self.syntax.text_range()
    }
    /// Raw text of this element
    pub fn raw(&self) -> String {
        self.syntax.to_string()
    }
    pub fn post_blank(&self) -> usize {
        super::blank_lines(&self.syntax)
    }
}
/// Typed syntax wrapper for `SyntaxInlinetaskEnd` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyntaxInlinetaskEnd {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for SyntaxInlinetaskEnd {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::INLINETASK_END
    }
    fn cast(node: SyntaxNode) -> Option<SyntaxInlinetaskEnd> {
        Self::can_cast(node.kind()).then(|| SyntaxInlinetaskEnd { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl SyntaxInlinetaskEnd {
    /// Beginning position of this element
    pub fn start(&self) -> TextSize {
        self.syntax.text_range().start()
    }
    /// Ending position of this element
    pub fn end(&self) -> TextSize {
        self.syntax.text_range().end()
    }
    /// Range of this element
    pub fn text_range(&self) -> TextRange {
        self.syntax.text_range()
    }
    /// Raw text of this element
    pub fn raw(&self) -> String {
        self.syntax.to_string()
    }
}
/// Typed syntax wrapper for `SyntaxKeyword` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyntaxKeyword {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for SyntaxKeyword {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::KEYWORD
    }
    fn cast(node: SyntaxNode) -> Option<SyntaxKeyword> {
        Self::can_cast(node.kind()).then(|| SyntaxKeyword { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl SyntaxKeyword {
    /// Beginning position of this element
    pub fn start(&self) -> TextSize {
        self.syntax.text_range().start()
    }
    /// Ending position of this element
    pub fn end(&self) -> TextSize {
        self.syntax.text_range().end()
    }
    /// Range of this element
    pub fn text_range(&self) -> TextRange {
        self.syntax.text_range()
    }
    /// Raw text of this element
    pub fn raw(&self) -> String {
        self.syntax.to_string()
    }
}

/// Typed syntax wrapper for `BabelCall` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BabelCall {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for BabelCall {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::BABEL_CALL
    }
    fn cast(node: SyntaxNode) -> Option<BabelCall> {
        Self::can_cast(node.kind()).then(|| BabelCall { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl BabelCall {
    /// Beginning position of this element
    pub fn start(&self) -> TextSize {
        self.syntax.text_range().start()
    }
    /// Ending position of this element
    pub fn end(&self) -> TextSize {
        self.syntax.text_range().end()
    }
    /// Range of this element
    pub fn text_range(&self) -> TextRange {
        self.syntax.text_range()
    }
    /// Raw text of this element
    pub fn raw(&self) -> String {
        self.syntax.to_string()
    }
}

/// Typed syntax wrapper for `AffiliatedKeyword` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AffiliatedKeyword {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for AffiliatedKeyword {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::AFFILIATED_KEYWORD
    }
    fn cast(node: SyntaxNode) -> Option<AffiliatedKeyword> {
        Self::can_cast(node.kind()).then(|| AffiliatedKeyword { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AffiliatedKeyword {
    /// Beginning position of this element
    pub fn start(&self) -> TextSize {
        self.syntax.text_range().start()
    }
    /// Ending position of this element
    pub fn end(&self) -> TextSize {
        self.syntax.text_range().end()
    }
    /// Range of this element
    pub fn text_range(&self) -> TextRange {
        self.syntax.text_range()
    }
    /// Raw text of this element
    pub fn raw(&self) -> String {
        self.syntax.to_string()
    }
}

/// Typed syntax wrapper for `TableEl` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TableEl {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for TableEl {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::TABLE_EL
    }
    fn cast(node: SyntaxNode) -> Option<TableEl> {
        Self::can_cast(node.kind()).then(|| TableEl { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl TableEl {
    /// Beginning position of this element
    pub fn start(&self) -> TextSize {
        self.syntax.text_range().start()
    }
    /// Ending position of this element
    pub fn end(&self) -> TextSize {
        self.syntax.text_range().end()
    }
    /// Range of this element
    pub fn text_range(&self) -> TextRange {
        self.syntax.text_range()
    }
    /// Raw text of this element
    pub fn raw(&self) -> String {
        self.syntax.to_string()
    }
    pub fn post_blank(&self) -> usize {
        super::blank_lines(&self.syntax)
    }
}

/// Typed syntax wrapper for `SyntaxClock` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyntaxClock {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for SyntaxClock {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::CLOCK
    }
    fn cast(node: SyntaxNode) -> Option<SyntaxClock> {
        Self::can_cast(node.kind()).then(|| SyntaxClock { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl SyntaxClock {
    /// Beginning position of this element
    pub fn start(&self) -> TextSize {
        self.syntax.text_range().start()
    }
    /// Ending position of this element
    pub fn end(&self) -> TextSize {
        self.syntax.text_range().end()
    }
    /// Range of this element
    pub fn text_range(&self) -> TextRange {
        self.syntax.text_range()
    }
    /// Raw text of this element
    pub fn raw(&self) -> String {
        self.syntax.to_string()
    }
    pub fn post_blank(&self) -> usize {
        super::blank_lines(&self.syntax)
    }
}

/// Typed syntax wrapper for `FnDef` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnDef {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for FnDef {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::FN_DEF
    }
    fn cast(node: SyntaxNode) -> Option<FnDef> {
        Self::can_cast(node.kind()).then(|| FnDef { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FnDef {
    /// Beginning position of this element
    pub fn start(&self) -> TextSize {
        self.syntax.text_range().start()
    }
    /// Ending position of this element
    pub fn end(&self) -> TextSize {
        self.syntax.text_range().end()
    }
    /// Range of this element
    pub fn text_range(&self) -> TextRange {
        self.syntax.text_range()
    }
    /// Raw text of this element
    pub fn raw(&self) -> String {
        self.syntax.to_string()
    }
    pub fn post_blank(&self) -> usize {
        super::blank_lines(&self.syntax)
    }
    pub fn caption(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "CAPTION")
    }
    pub fn header(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "HEADER")
    }
    pub fn name(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "NAME")
    }
    pub fn plot(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "PLOT")
    }
    pub fn results(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "RESULTS")
    }
    pub fn attr(&self, backend: &str) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| {
            k.starts_with("ATTR_") && &k[5..] == backend
        })
    }
}

/// Typed syntax wrapper for `Comment` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Comment {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Comment {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::COMMENT
    }
    fn cast(node: SyntaxNode) -> Option<Comment> {
        Self::can_cast(node.kind()).then(|| Comment { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Comment {
    /// Beginning position of this element
    pub fn start(&self) -> TextSize {
        self.syntax.text_range().start()
    }
    /// Ending position of this element
    pub fn end(&self) -> TextSize {
        self.syntax.text_range().end()
    }
    /// Range of this element
    pub fn text_range(&self) -> TextRange {
        self.syntax.text_range()
    }
    /// Raw text of this element
    pub fn raw(&self) -> String {
        self.syntax.to_string()
    }
    pub fn text(&self) -> Option<super::Token> {
        super::token(&self.syntax, SyntaxKind::TEXT)
    }
    pub fn post_blank(&self) -> usize {
        super::blank_lines(&self.syntax)
    }
    pub fn caption(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "CAPTION")
    }
    pub fn header(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "HEADER")
    }
    pub fn name(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "NAME")
    }
    pub fn plot(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "PLOT")
    }
    pub fn results(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "RESULTS")
    }
    pub fn attr(&self, backend: &str) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| {
            k.starts_with("ATTR_") && &k[5..] == backend
        })
    }
}

/// Typed syntax wrapper for `Rule` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rule {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Rule {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::RULE
    }
    fn cast(node: SyntaxNode) -> Option<Rule> {
        Self::can_cast(node.kind()).then(|| Rule { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Rule {
    /// Beginning position of this element
    pub fn start(&self) -> TextSize {
        self.syntax.text_range().start()
    }
    /// Ending position of this element
    pub fn end(&self) -> TextSize {
        self.syntax.text_range().end()
    }
    /// Range of this element
    pub fn text_range(&self) -> TextRange {
        self.syntax.text_range()
    }
    /// Raw text of this element
    pub fn raw(&self) -> String {
        self.syntax.to_string()
    }
    pub fn post_blank(&self) -> usize {
        super::blank_lines(&self.syntax)
    }
}

/// Typed syntax wrapper for `FixedWidth` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FixedWidth {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for FixedWidth {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::FIXED_WIDTH
    }
    fn cast(node: SyntaxNode) -> Option<FixedWidth> {
        Self::can_cast(node.kind()).then(|| FixedWidth { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FixedWidth {
    /// Beginning position of this element
    pub fn start(&self) -> TextSize {
        self.syntax.text_range().start()
    }
    /// Ending position of this element
    pub fn end(&self) -> TextSize {
        self.syntax.text_range().end()
    }
    /// Range of this element
    pub fn text_range(&self) -> TextRange {
        self.syntax.text_range()
    }
    /// Raw text of this element
    pub fn raw(&self) -> String {
        self.syntax.to_string()
    }
    pub fn text(&self) -> Option<super::Token> {
        super::token(&self.syntax, SyntaxKind::TEXT)
    }
    pub fn post_blank(&self) -> usize {
        super::blank_lines(&self.syntax)
    }
    pub fn caption(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "CAPTION")
    }
    pub fn header(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "HEADER")
    }
    pub fn name(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "NAME")
    }
    pub fn plot(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "PLOT")
    }
    pub fn results(&self) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| k == "RESULTS")
    }
    pub fn attr(&self, backend: &str) -> Option<AffiliatedKeyword> {
        affiliated_keyword(&self.syntax, |k| {
            k.starts_with("ATTR_") && &k[5..] == backend
        })
    }
}
