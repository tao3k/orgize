//! Generated typed AST wrappers for one syntax family.

use crate::syntax::{OrgLanguage, SyntaxKind, SyntaxNode, SyntaxToken};
use rowan::{
    ast::{support, AstChildren, AstNode},
    TextRange, TextSize,
};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InlineCall {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for InlineCall {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::INLINE_CALL
    }
    fn cast(node: SyntaxNode) -> Option<InlineCall> {
        Self::can_cast(node.kind()).then(|| InlineCall { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl InlineCall {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InlineSrc {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for InlineSrc {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::INLINE_SRC
    }
    fn cast(node: SyntaxNode) -> Option<InlineSrc> {
        Self::can_cast(node.kind()).then(|| InlineSrc { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl InlineSrc {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Citation {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Citation {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::CITATION
    }
    fn cast(node: SyntaxNode) -> Option<Citation> {
        Self::can_cast(node.kind()).then(|| Citation { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Citation {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Link {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Link {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::LINK
    }
    fn cast(node: SyntaxNode) -> Option<Link> {
        Self::can_cast(node.kind()).then(|| Link { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Link {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cookie {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Cookie {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::COOKIE
    }
    fn cast(node: SyntaxNode) -> Option<Cookie> {
        Self::can_cast(node.kind()).then(|| Cookie { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Cookie {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RadioTarget {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for RadioTarget {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::RADIO_TARGET
    }
    fn cast(node: SyntaxNode) -> Option<RadioTarget> {
        Self::can_cast(node.kind()).then(|| RadioTarget { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl RadioTarget {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnRef {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for FnRef {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::FN_REF
    }
    fn cast(node: SyntaxNode) -> Option<FnRef> {
        Self::can_cast(node.kind()).then(|| FnRef { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FnRef {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Macros {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Macros {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::MACROS
    }
    fn cast(node: SyntaxNode) -> Option<Macros> {
        Self::can_cast(node.kind()).then(|| Macros { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Macros {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Snippet {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Snippet {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::SNIPPET
    }
    fn cast(node: SyntaxNode) -> Option<Snippet> {
        Self::can_cast(node.kind()).then(|| Snippet { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Snippet {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Target {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Target {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::TARGET
    }
    fn cast(node: SyntaxNode) -> Option<Target> {
        Self::can_cast(node.kind()).then(|| Target { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Target {
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
