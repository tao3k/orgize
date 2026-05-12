use rowan::ast::support;

use crate::SyntaxKind;

use super::Token;

use super::{SyntaxClock, SyntaxTimestamp};

impl SyntaxClock {
    pub fn value(&self) -> Option<SyntaxTimestamp> {
        support::child(&self.syntax)
    }

    /// ```rust
    /// use orgize::{Org, syntax_ast::SyntaxClock};
    ///
    /// let clock = Org::parse("CLOCK: [2003-09-16 Tue 09:39]").first_node::<SyntaxClock>().unwrap();
    /// assert!(clock.duration().is_none());
    /// let clock = Org::parse("CLOCK: [2003-09-16 Tue 09:39] =>12:00").first_node::<SyntaxClock>().unwrap();
    /// assert_eq!(clock.duration().unwrap(), "12:00");
    ///
    /// ```
    pub fn duration(&self) -> Option<Token> {
        self.syntax
            .children_with_tokens()
            .skip_while(|t| t.kind() != SyntaxKind::DOUBLE_ARROW)
            .skip(1)
            .find(|t| t.kind() != SyntaxKind::WHITESPACE)
            .and_then(|e| {
                debug_assert_eq!(e.kind(), SyntaxKind::TEXT);
                Some(Token(e.into_token()?))
            })
    }

    /// ```rust
    /// use orgize::{Org, syntax_ast::SyntaxClock};
    ///
    /// let clock = Org::parse("CLOCK: [2003-09-16 Tue 09:39]").first_node::<SyntaxClock>().unwrap();
    /// assert!(!clock.is_closed());
    /// let clock = Org::parse("CLOCK: [2003-09-16 Tue 09:39] =>12:00").first_node::<SyntaxClock>().unwrap();
    /// assert!(clock.is_closed());
    /// ```
    pub fn is_closed(&self) -> bool {
        self.syntax
            .children_with_tokens()
            .any(|t| t.kind() == SyntaxKind::DOUBLE_ARROW)
    }

    /// ```rust
    /// use orgize::{Org, syntax_ast::SyntaxClock};
    ///
    /// let clock = Org::parse("CLOCK: [2003-09-16 Tue 09:39]").first_node::<SyntaxClock>().unwrap();
    /// assert!(clock.is_running());
    /// let clock = Org::parse("CLOCK: [2003-09-16 Tue 09:39] =>12:00").first_node::<SyntaxClock>().unwrap();
    /// assert!(!clock.is_running());
    /// ```
    pub fn is_running(&self) -> bool {
        !self.is_closed()
    }
}
