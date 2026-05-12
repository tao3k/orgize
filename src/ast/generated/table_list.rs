//! Generated typed AST wrappers for one syntax family.

use crate::syntax::{OrgLanguage, SyntaxKind, SyntaxNode, SyntaxToken};
use rowan::{
    ast::{support, AstChildren, AstNode},
    TextRange, TextSize,
};

use super::{affiliated_keyword, AffiliatedKeyword};

/// Typed syntax wrapper for `OrgTable` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrgTable {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for OrgTable {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::ORG_TABLE
    }
    fn cast(node: SyntaxNode) -> Option<OrgTable> {
        Self::can_cast(node.kind()).then(|| OrgTable { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl OrgTable {
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
/// Typed syntax wrapper for `OrgTableRow` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrgTableRow {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for OrgTableRow {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::ORG_TABLE_RULE_ROW || kind == SyntaxKind::ORG_TABLE_STANDARD_ROW
    }
    fn cast(node: SyntaxNode) -> Option<OrgTableRow> {
        Self::can_cast(node.kind()).then(|| OrgTableRow { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl OrgTableRow {
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

/// Typed syntax wrapper for `OrgTableCell` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrgTableCell {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for OrgTableCell {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::ORG_TABLE_CELL
    }
    fn cast(node: SyntaxNode) -> Option<OrgTableCell> {
        Self::can_cast(node.kind()).then(|| OrgTableCell { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl OrgTableCell {
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

/// Typed syntax wrapper for `SyntaxList` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyntaxList {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for SyntaxList {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::LIST
    }
    fn cast(node: SyntaxNode) -> Option<SyntaxList> {
        Self::can_cast(node.kind()).then(|| SyntaxList { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl SyntaxList {
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
    pub fn items(&self) -> AstChildren<SyntaxListItem> {
        support::children(&self.syntax)
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

/// Typed syntax wrapper for `SyntaxListItem` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyntaxListItem {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for SyntaxListItem {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::LIST_ITEM
    }
    fn cast(node: SyntaxNode) -> Option<SyntaxListItem> {
        Self::can_cast(node.kind()).then(|| SyntaxListItem { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl SyntaxListItem {
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

/// Typed syntax wrapper for `SyntaxDrawer` nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyntaxDrawer {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for SyntaxDrawer {
    type Language = OrgLanguage;
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::DRAWER
    }
    fn cast(node: SyntaxNode) -> Option<SyntaxDrawer> {
        Self::can_cast(node.kind()).then(|| SyntaxDrawer { syntax: node })
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl SyntaxDrawer {
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
