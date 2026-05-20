use crate::SyntaxKind;

use super::{InlineSrc, Token, filter_token};

impl InlineSrc {
    /// Language of the code
    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::InlineSrc};
    ///
    /// let s = Org::parse("src_C{int a = 0;}").first_node::<InlineSrc>().unwrap();
    /// assert_eq!(s.language(), "C");
    /// let s = Org::parse("src_xml[:exports code]{<tag>text</tag>}").first_node::<InlineSrc>().unwrap();
    /// assert_eq!(s.language(), "xml");
    /// ```
    pub fn language(&self) -> Token {
        self.syntax
            .children_with_tokens()
            .nth(1)
            .and_then(filter_token(SyntaxKind::TEXT))
            .expect("inline src must contains TEXT")
    }

    /// Optional header arguments
    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::InlineSrc};
    ///
    /// let s = Org::parse("src_C{int a = 0;}").first_node::<InlineSrc>().unwrap();
    /// assert!(s.parameters().is_none());
    /// let s = Org::parse("src_xml[:exports code]{<tag>text</tag>}").first_node::<InlineSrc>().unwrap();
    /// assert_eq!(s.parameters().unwrap(), ":exports code");
    /// ```
    pub fn parameters(&self) -> Option<Token> {
        self.syntax
            .children_with_tokens()
            .skip_while(|n| n.kind() != SyntaxKind::L_BRACKET)
            .nth(1)
            .and_then(|n| {
                debug_assert_eq!(n.kind(), SyntaxKind::TEXT);
                Some(Token(n.into_token()?))
            })
    }

    /// Source code
    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::InlineSrc};
    ///
    /// let s = Org::parse("src_C{int a = 0;}").first_node::<InlineSrc>().unwrap();
    /// assert_eq!(s.value(), "int a = 0;");
    /// let s = Org::parse("src_xml[:exports code]{<tag>text</tag>}").first_node::<InlineSrc>().unwrap();
    /// assert_eq!(s.value(), "<tag>text</tag>");
    /// ```
    pub fn value(&self) -> Token {
        self.syntax
            .children_with_tokens()
            .filter_map(filter_token(SyntaxKind::TEXT))
            .last()
            .expect("inline src must contains TEXT")
    }
}
