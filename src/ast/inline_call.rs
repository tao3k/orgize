use crate::syntax::SyntaxKind;

use super::{filter_token, InlineCall, Token};

impl InlineCall {
    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::InlineCall};
    ///
    /// let call = Org::parse("call_square(4)").first_node::<InlineCall>().unwrap();
    /// assert_eq!(call.call(), "square");
    /// ```
    pub fn call(&self) -> Token {
        self.syntax
            .children_with_tokens()
            .filter_map(filter_token(SyntaxKind::TEXT))
            .nth(1)
            .expect("inline call must contains two TEXT")
    }

    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::InlineCall};
    ///
    /// let call = Org::parse("call_square[:results output](4)").first_node::<InlineCall>().unwrap();
    /// assert_eq!(call.inside_header().unwrap(), ":results output");
    ///
    /// let call = Org::parse("call_square(4)[:results html]").first_node::<InlineCall>().unwrap();
    /// assert!(call.inside_header().is_none());
    /// ```
    pub fn inside_header(&self) -> Option<Token> {
        self.syntax
            .children_with_tokens()
            .take_while(|e| e.kind() != SyntaxKind::L_PARENS)
            .skip_while(|e| e.kind() != SyntaxKind::L_BRACKET)
            .nth(1)
            .and_then(|e| {
                debug_assert_eq!(e.kind(), SyntaxKind::TEXT);
                Some(Token(e.into_token()?))
            })
    }

    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::InlineCall};
    ///
    /// let call = Org::parse("call_square(4)").first_node::<InlineCall>().unwrap();
    /// assert_eq!(call.arguments(), "4");
    /// ```
    pub fn arguments(&self) -> Token {
        self.syntax
            .children_with_tokens()
            .skip_while(|e| e.kind() != SyntaxKind::L_PARENS)
            .find_map(filter_token(SyntaxKind::TEXT))
            .expect("inline call must contains TEXT after L_PARENS")
    }

    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::InlineCall};
    ///
    /// let call = Org::parse("call_square[:results output](4)[:results html]").first_node::<InlineCall>().unwrap();
    /// assert_eq!(call.end_header().unwrap(), ":results html");
    ///
    /// let call = Org::parse("call_square[:results output](4)").first_node::<InlineCall>().unwrap();
    /// assert!(call.end_header().is_none());
    /// ```
    pub fn end_header(&self) -> Option<Token> {
        self.syntax
            .children_with_tokens()
            .skip_while(|e| e.kind() != SyntaxKind::L_BRACKET)
            .skip(1)
            .skip_while(|e| e.kind() != SyntaxKind::L_BRACKET)
            .nth(1)
            .and_then(|e| {
                debug_assert_eq!(e.kind(), SyntaxKind::TEXT);
                Some(Token(e.into_token()?))
            })
    }
}
