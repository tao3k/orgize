//! Generated typed AST wrappers for one syntax family.

use crate::syntax::{OrgLanguage, SyntaxKind, SyntaxNode, SyntaxToken};
use rowan::{
    ast::{support, AstChildren, AstNode},
    TextRange, TextSize,
};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bold {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Bold {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::BOLD
    }
    fn cast(node: SyntaxNode) -> Option<Bold> {
        Self::can_cast(node.kind()).then(|| Bold { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Bold {
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
pub struct Strike {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Strike {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::STRIKE
    }
    fn cast(node: SyntaxNode) -> Option<Strike> {
        Self::can_cast(node.kind()).then(|| Strike { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Strike {
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
pub struct Italic {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Italic {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::ITALIC
    }
    fn cast(node: SyntaxNode) -> Option<Italic> {
        Self::can_cast(node.kind()).then(|| Italic { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Italic {
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
pub struct Underline {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Underline {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::UNDERLINE
    }
    fn cast(node: SyntaxNode) -> Option<Underline> {
        Self::can_cast(node.kind()).then(|| Underline { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Underline {
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
pub struct Verbatim {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Verbatim {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::VERBATIM
    }
    fn cast(node: SyntaxNode) -> Option<Verbatim> {
        Self::can_cast(node.kind()).then(|| Verbatim { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Verbatim {
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
pub struct Code {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Code {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::CODE
    }
    fn cast(node: SyntaxNode) -> Option<Code> {
        Self::can_cast(node.kind()).then(|| Code { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Code {
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
}
