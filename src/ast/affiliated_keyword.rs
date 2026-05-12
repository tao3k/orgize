use crate::syntax::SyntaxKind;

use super::{filter_token, AffiliatedKeyword, Token};

impl AffiliatedKeyword {
    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::AffiliatedKeyword};
    ///
    /// let keyword = Org::parse("#+CAPTION: VALUE\nabc").first_node::<AffiliatedKeyword>().unwrap();
    /// assert_eq!(keyword.key(), "CAPTION");
    /// ```
    pub fn key(&self) -> Token {
        self.syntax
            .children_with_tokens()
            .find_map(filter_token(SyntaxKind::TEXT))
            .expect("keyword must contains TEXT")
    }

    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::AffiliatedKeyword};
    ///
    /// let keyword = Org::parse("#+CAPTION: VALUE\nabc").first_node::<AffiliatedKeyword>().unwrap();
    /// assert!(keyword.optional().is_none());
    /// let keyword = Org::parse("#+CAPTION[OPTIONAL]: VALUE\nabc").first_node::<AffiliatedKeyword>().unwrap();
    /// assert_eq!(keyword.optional().unwrap(), "OPTIONAL");
    /// ```
    pub fn optional(&self) -> Option<Token> {
        self.syntax
            .children_with_tokens()
            .skip_while(|it| it.kind() != SyntaxKind::L_BRACKET)
            .nth(1)
            .and_then(filter_token(SyntaxKind::TEXT))
    }

    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::AffiliatedKeyword};
    ///
    /// let keyword = Org::parse("#+CAPTION: VALUE\nabc").first_node::<AffiliatedKeyword>().unwrap();
    /// assert_eq!(keyword.value().unwrap(), " VALUE");
    /// let keyword = Org::parse("#+CAPTION[OPTIONAL]:VALUE\nabc").first_node::<AffiliatedKeyword>().unwrap();
    /// assert_eq!(keyword.value().unwrap(), "VALUE");
    /// ```
    pub fn value(&self) -> Option<Token> {
        self.syntax
            .children_with_tokens()
            .filter_map(filter_token(SyntaxKind::TEXT))
            .last()
    }
}
