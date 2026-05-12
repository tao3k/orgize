use rowan::ast::AstNode;

use super::{SyntaxPlanning, SyntaxTimestamp};
use crate::syntax::SyntaxKind;

impl SyntaxPlanning {
    /// Returns deadline timestamp
    ///
    ///
    /// ```rust
    /// use orgize::{syntax_ast::SyntaxPlanning, Org};
    ///
    /// let s = Org::parse("* a\nDEADLINE: <2019-04-08 Mon>")
    ///     .first_node::<SyntaxPlanning>()
    ///     .unwrap()
    ///     .deadline()
    ///     .unwrap();
    /// assert_eq!(s.day_start().unwrap(), "08");
    /// ```
    pub fn deadline(&self) -> Option<SyntaxTimestamp> {
        self.syntax
            .children()
            .filter(|n| n.kind() == SyntaxKind::PLANNING_DEADLINE)
            .last()
            .and_then(|n| n.children().find_map(SyntaxTimestamp::cast))
    }

    /// Returns scheduled timestamp
    ///
    /// ```rust
    /// use orgize::{syntax_ast::SyntaxPlanning, Org};
    ///
    /// let s = Org::parse("* a\nSCHEDULED: <2019-04-08 Mon>")
    ///     .first_node::<SyntaxPlanning>()
    ///     .unwrap()
    ///     .scheduled()
    ///     .unwrap();
    /// assert_eq!(s.year_start().unwrap(), "2019");
    /// ```
    pub fn scheduled(&self) -> Option<SyntaxTimestamp> {
        self.syntax
            .children()
            .filter(|n| n.kind() == SyntaxKind::PLANNING_SCHEDULED)
            .last()
            .and_then(|n| n.children().find_map(SyntaxTimestamp::cast))
    }

    /// Returns closed timestamp
    ///
    /// ```rust
    /// use orgize::{syntax_ast::SyntaxPlanning, Org};
    ///
    /// let s = Org::parse("* a\nCLOSED: <2019-04-08 Mon>")
    ///     .first_node::<SyntaxPlanning>()
    ///     .unwrap()
    ///     .closed()
    ///     .unwrap();
    /// assert_eq!(s.month_start().unwrap(), "04");
    /// ```
    pub fn closed(&self) -> Option<SyntaxTimestamp> {
        self.syntax
            .children()
            .filter(|n| n.kind() == SyntaxKind::PLANNING_CLOSED)
            .last()
            .and_then(|n| n.children().find_map(SyntaxTimestamp::cast))
    }
}
