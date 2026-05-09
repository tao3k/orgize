//! Generated typed AST wrappers for one syntax family.

use crate::syntax::{OrgLanguage, SyntaxKind, SyntaxNode, SyntaxToken};
use rowan::{
    ast::{support, AstChildren, AstNode},
    TextRange, TextSize,
};

use super::{affiliated_keyword, AffiliatedKeyword};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpecialBlock {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for SpecialBlock {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::SPECIAL_BLOCK
    }
    fn cast(node: SyntaxNode) -> Option<SpecialBlock> {
        Self::can_cast(node.kind()).then(|| SpecialBlock { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl SpecialBlock {
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuoteBlock {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for QuoteBlock {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::QUOTE_BLOCK
    }
    fn cast(node: SyntaxNode) -> Option<QuoteBlock> {
        Self::can_cast(node.kind()).then(|| QuoteBlock { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl QuoteBlock {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CenterBlock {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for CenterBlock {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::CENTER_BLOCK
    }
    fn cast(node: SyntaxNode) -> Option<CenterBlock> {
        Self::can_cast(node.kind()).then(|| CenterBlock { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl CenterBlock {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VerseBlock {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for VerseBlock {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::VERSE_BLOCK
    }
    fn cast(node: SyntaxNode) -> Option<VerseBlock> {
        Self::can_cast(node.kind()).then(|| VerseBlock { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl VerseBlock {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommentBlock {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for CommentBlock {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::COMMENT_BLOCK
    }
    fn cast(node: SyntaxNode) -> Option<CommentBlock> {
        Self::can_cast(node.kind()).then(|| CommentBlock { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl CommentBlock {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExampleBlock {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ExampleBlock {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::EXAMPLE_BLOCK
    }
    fn cast(node: SyntaxNode) -> Option<ExampleBlock> {
        Self::can_cast(node.kind()).then(|| ExampleBlock { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl ExampleBlock {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExportBlock {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ExportBlock {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::EXPORT_BLOCK
    }
    fn cast(node: SyntaxNode) -> Option<ExportBlock> {
        Self::can_cast(node.kind()).then(|| ExportBlock { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl ExportBlock {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceBlock {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for SourceBlock {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::SOURCE_BLOCK
    }
    fn cast(node: SyntaxNode) -> Option<SourceBlock> {
        Self::can_cast(node.kind()).then(|| SourceBlock { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl SourceBlock {
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
