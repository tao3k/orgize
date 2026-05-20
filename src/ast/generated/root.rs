//! Generated typed AST wrappers for one syntax family.

use crate::syntax::{OrgLanguage, SyntaxKind, SyntaxNode, SyntaxToken};
use rowan::{
    TextRange, TextSize,
    ast::{AstChildren, AstNode, support},
};

use super::{AffiliatedKeyword, affiliated_keyword};

/// Typed syntax wrapper for `SyntaxDocument` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyntaxDocument {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for SyntaxDocument {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::DOCUMENT
    }
    fn cast(node: SyntaxNode) -> Option<SyntaxDocument> {
        Self::can_cast(node.kind()).then(|| SyntaxDocument { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl SyntaxDocument {
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
    pub fn section(&self) -> Option<SyntaxSection> {
        support::child(&self.syntax)
    }
    pub fn first_headline(&self) -> Option<Headline> {
        support::child(&self.syntax)
    }
    pub fn last_headline(&self) -> Option<Headline> {
        super::last_child(&self.syntax)
    }
    pub fn headlines(&self) -> AstChildren<Headline> {
        support::children(&self.syntax)
    }
    pub fn pre_blank(&self) -> usize {
        super::blank_lines(&self.syntax)
    }
}
/// Typed syntax wrapper for `SyntaxSection` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyntaxSection {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for SyntaxSection {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::SECTION
    }
    fn cast(node: SyntaxNode) -> Option<SyntaxSection> {
        Self::can_cast(node.kind()).then(|| SyntaxSection { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl SyntaxSection {
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

/// Typed syntax wrapper for `Paragraph` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Paragraph {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Paragraph {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::PARAGRAPH
    }
    fn cast(node: SyntaxNode) -> Option<Paragraph> {
        Self::can_cast(node.kind()).then(|| Paragraph { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Paragraph {
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

/// Typed syntax wrapper for `Headline` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Headline {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Headline {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::HEADLINE
    }
    fn cast(node: SyntaxNode) -> Option<Headline> {
        Self::can_cast(node.kind()).then(|| Headline { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Headline {
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
    pub fn section(&self) -> Option<SyntaxSection> {
        support::child(&self.syntax)
    }
    pub fn planning(&self) -> Option<SyntaxPlanning> {
        support::child(&self.syntax)
    }
    pub fn properties(&self) -> Option<PropertyDrawer> {
        support::child(&self.syntax)
    }
    pub fn headlines(&self) -> AstChildren<Headline> {
        support::children(&self.syntax)
    }
    pub fn post_blank(&self) -> usize {
        super::blank_lines(&self.syntax)
    }
}

/// Typed syntax wrapper for `PropertyDrawer` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropertyDrawer {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for PropertyDrawer {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::PROPERTY_DRAWER
    }
    fn cast(node: SyntaxNode) -> Option<PropertyDrawer> {
        Self::can_cast(node.kind()).then(|| PropertyDrawer { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl PropertyDrawer {
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
    pub fn node_properties(&self) -> AstChildren<NodeProperty> {
        support::children(&self.syntax)
    }
}

/// Typed syntax wrapper for `NodeProperty` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeProperty {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for NodeProperty {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::NODE_PROPERTY
    }
    fn cast(node: SyntaxNode) -> Option<NodeProperty> {
        Self::can_cast(node.kind()).then(|| NodeProperty { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl NodeProperty {
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

/// Typed syntax wrapper for `SyntaxPlanning` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyntaxPlanning {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for SyntaxPlanning {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::PLANNING
    }
    fn cast(node: SyntaxNode) -> Option<SyntaxPlanning> {
        Self::can_cast(node.kind()).then(|| SyntaxPlanning { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl SyntaxPlanning {
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
