//! Shared helpers for typed syntax AST wrappers.

use crate::{
    syntax::{SyntaxKind, SyntaxNode},
    SyntaxToken,
};
use rowan::{ast::AstNode, NodeOrToken, TextRange, TextSize};
use std::{
    borrow::{Borrow, Cow},
    fmt,
    hash::Hash,
    ops::Deref,
};

pub(super) fn blank_lines(parent: &SyntaxNode) -> usize {
    parent
        .children_with_tokens()
        .filter(|n| n.kind() == SyntaxKind::BLANK_LINE)
        .count()
}

pub(super) fn last_child<N: AstNode>(parent: &rowan::SyntaxNode<N::Language>) -> Option<N> {
    parent.children().filter_map(N::cast).last()
}

pub(super) fn last_token(parent: &SyntaxNode, kind: SyntaxKind) -> Option<Token> {
    parent
        .children_with_tokens()
        .filter_map(filter_token(kind))
        .last()
}

pub(super) fn token(parent: &SyntaxNode, kind: SyntaxKind) -> Option<Token> {
    rowan::ast::support::token(parent, kind).map(Token)
}

pub(super) fn filter_token(
    kind: SyntaxKind,
) -> impl Fn(NodeOrToken<SyntaxNode, SyntaxToken>) -> Option<Token> {
    move |elem| match elem {
        NodeOrToken::Token(tk) if tk.kind() == kind => Some(Token(tk)),
        _ => None,
    }
}

/// A simple wrapper of `SyntaxToken`
///
/// It implements the `AsRef<str>` and `Display` trait,
/// allowing to directly use some `str` methods.
///
/// Also it implements `Hash` and `Eq` traits, so can be
/// used as keys in `HashMap`. However, note that it only
/// compares the underlying text inside `SyntaxToken`,
/// meaning two `Token`s from different positions
/// might be considered equal.
#[derive(Eq, Clone)]
pub struct Token(pub(crate) SyntaxToken);

impl Token {
    pub fn syntax(&self) -> &SyntaxToken {
        &self.0
    }

    /// Range of this token
    pub fn text_range(&self) -> TextRange {
        self.0.text_range()
    }

    /// Beginning position of this token
    pub fn start(&self) -> TextSize {
        self.0.text_range().start()
    }

    /// Ending position of this token
    pub fn end(&self) -> TextSize {
        self.0.text_range().end()
    }
}

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str {
        self.0.text()
    }
}

impl Borrow<str> for Token {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

impl<'a> PartialEq<&'a str> for Token {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}

impl PartialEq<String> for Token {
    fn eq(&self, other: &String) -> bool {
        self.as_ref() == other
    }
}

impl PartialEq<Token> for Token {
    fn eq(&self, other: &Token) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}

impl<'a> PartialEq<Cow<'a, str>> for Token {
    fn eq(&self, other: &Cow<'a, str>) -> bool {
        self.as_ref() == other
    }
}

impl PartialEq<str> for Token {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl Deref for Token {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_ref()
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.0.text(), f)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.0.text(), f)
    }
}
