//! Generated typed AST wrappers for one syntax family.

use crate::syntax::{OrgLanguage, SyntaxKind, SyntaxNode, SyntaxToken};
use rowan::{
    ast::{support, AstChildren, AstNode},
    TextRange, TextSize,
};
/// Typed syntax wrapper for `SyntaxTimestamp` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyntaxTimestamp {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for SyntaxTimestamp {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::TIMESTAMP_ACTIVE
            || kind == SyntaxKind::TIMESTAMP_INACTIVE
            || kind == SyntaxKind::TIMESTAMP_DIARY
    }
    fn cast(node: SyntaxNode) -> Option<SyntaxTimestamp> {
        Self::can_cast(node.kind()).then(|| SyntaxTimestamp { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl SyntaxTimestamp {
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
    pub fn year_start(&self) -> Option<super::Token> {
        super::token(&self.syntax, SyntaxKind::TIMESTAMP_YEAR)
    }
    pub fn month_start(&self) -> Option<super::Token> {
        super::token(&self.syntax, SyntaxKind::TIMESTAMP_MONTH)
    }
    pub fn day_start(&self) -> Option<super::Token> {
        super::token(&self.syntax, SyntaxKind::TIMESTAMP_DAY)
    }
    pub fn hour_start(&self) -> Option<super::Token> {
        super::token(&self.syntax, SyntaxKind::TIMESTAMP_HOUR)
    }
    pub fn minute_start(&self) -> Option<super::Token> {
        super::token(&self.syntax, SyntaxKind::TIMESTAMP_MINUTE)
    }
    pub fn year_end(&self) -> Option<super::Token> {
        super::last_token(&self.syntax, SyntaxKind::TIMESTAMP_YEAR)
    }
    pub fn month_end(&self) -> Option<super::Token> {
        super::last_token(&self.syntax, SyntaxKind::TIMESTAMP_MONTH)
    }
    pub fn day_end(&self) -> Option<super::Token> {
        super::last_token(&self.syntax, SyntaxKind::TIMESTAMP_DAY)
    }
    pub fn hour_end(&self) -> Option<super::Token> {
        super::last_token(&self.syntax, SyntaxKind::TIMESTAMP_HOUR)
    }
    pub fn minute_end(&self) -> Option<super::Token> {
        super::last_token(&self.syntax, SyntaxKind::TIMESTAMP_MINUTE)
    }
}

/// Typed syntax wrapper for `LatexEnvironment` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LatexEnvironment {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for LatexEnvironment {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::LATEX_ENVIRONMENT
    }
    fn cast(node: SyntaxNode) -> Option<LatexEnvironment> {
        Self::can_cast(node.kind()).then(|| LatexEnvironment { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl LatexEnvironment {
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

/// Typed syntax wrapper for `LatexFragment` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LatexFragment {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for LatexFragment {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::LATEX_FRAGMENT
    }
    fn cast(node: SyntaxNode) -> Option<LatexFragment> {
        Self::can_cast(node.kind()).then(|| LatexFragment { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl LatexFragment {
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

/// Typed syntax wrapper for `Entity` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Entity {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Entity {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::ENTITY
    }
    fn cast(node: SyntaxNode) -> Option<Entity> {
        Self::can_cast(node.kind()).then(|| Entity { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Entity {
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

/// Typed syntax wrapper for `LineBreak` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LineBreak {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for LineBreak {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::LINE_BREAK
    }
    fn cast(node: SyntaxNode) -> Option<LineBreak> {
        Self::can_cast(node.kind()).then(|| LineBreak { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl LineBreak {
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

/// Typed syntax wrapper for `Superscript` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Superscript {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Superscript {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::SUPERSCRIPT
    }
    fn cast(node: SyntaxNode) -> Option<Superscript> {
        Self::can_cast(node.kind()).then(|| Superscript { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Superscript {
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

/// Typed syntax wrapper for `Subscript` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Subscript {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Subscript {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::SUBSCRIPT
    }
    fn cast(node: SyntaxNode) -> Option<Subscript> {
        Self::can_cast(node.kind()).then(|| Subscript { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Subscript {
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
