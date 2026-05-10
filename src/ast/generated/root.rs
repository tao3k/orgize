//! Generated typed AST wrappers for one syntax family.

use crate::syntax::{OrgLanguage, SyntaxKind, SyntaxNode, SyntaxToken};
use rowan::{
    ast::{support, AstChildren, AstNode},
    TextRange, TextSize,
};

use super::{affiliated_keyword, AffiliatedKeyword};

/// Typed syntax wrapper for `Document` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Document {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Document {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::DOCUMENT
    }
    fn cast(node: SyntaxNode) -> Option<Document> {
        Self::can_cast(node.kind()).then(|| Document { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Document {
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
    pub fn section(&self) -> Option<Section> {
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
/// Typed syntax wrapper for `Section` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Section {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Section {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::SECTION
    }
    fn cast(node: SyntaxNode) -> Option<Section> {
        Self::can_cast(node.kind()).then(|| Section { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Section {
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
    pub fn section(&self) -> Option<Section> {
        support::child(&self.syntax)
    }
    pub fn planning(&self) -> Option<Planning> {
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

/// Typed syntax wrapper for `Planning` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Planning {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Planning {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::PLANNING
    }
    fn cast(node: SyntaxNode) -> Option<Planning> {
        Self::can_cast(node.kind()).then(|| Planning { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Planning {
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
