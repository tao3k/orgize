use crate::{syntax::OrgLanguage, SyntaxElement, SyntaxKind, SyntaxNode};
use rowan::{ast::AstNode, TextRange, TextSize};

use super::Token;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cloze {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for Cloze {
    type Language = OrgLanguage;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::CLOZE
    }

    fn cast(node: SyntaxNode) -> Option<Cloze> {
        Self::can_cast(node.kind()).then(|| Cloze { syntax: node })
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl Cloze {
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

    pub fn text(&self) -> impl Iterator<Item = SyntaxElement> {
        self.syntax
            .children_with_tokens()
            .skip(1)
            .take_while(|n| n.kind() != SyntaxKind::R_CURLY)
    }

    /// ```rust
    /// use orgize::{Org, syntax_ast::Cloze};
    ///
    /// let cloze = Org::parse("{{text}}").first_node::<Cloze>().unwrap();
    /// assert_eq!(cloze.text_raw(), "text");
    /// let cloze = Org::parse("{{$\\frac{1}{2}$}{}@id}").first_node::<Cloze>().unwrap();
    /// assert_eq!(cloze.text_raw(), "$\\frac{1}{2}$");
    /// let cloze = Org::parse("{{ [[file:my_image.png]] }{hint}}").first_node::<Cloze>().unwrap();
    /// assert_eq!(cloze.text_raw(), " [[file:my_image.png]] ");
    /// ```
    pub fn text_raw(&self) -> String {
        self.text()
            .fold(String::new(), |acc, e| acc + &e.to_string())
    }

    /// ```rust
    /// use orgize::{Org, syntax_ast::Cloze};
    ///
    /// let cloze = Org::parse("{{text}}").first_node::<Cloze>().unwrap();
    /// assert!(cloze.hint().is_none());
    /// let cloze = Org::parse("{{text}{}@id}").first_node::<Cloze>().unwrap();
    /// assert_eq!(cloze.hint().unwrap(), "");
    /// let cloze = Org::parse("{{text}{hint}}").first_node::<Cloze>().unwrap();
    /// assert_eq!(cloze.hint().unwrap(), "hint");
    /// ```
    pub fn hint(&self) -> Option<Token> {
        self.syntax
            .children_with_tokens()
            .skip_while(|n| n.kind() != SyntaxKind::L_CURLY)
            .nth(1)
            .and_then(|e| {
                debug_assert_eq!(e.kind(), SyntaxKind::TEXT);
                Some(Token(e.into_token()?))
            })
    }

    /// ```rust
    /// use orgize::{Org, syntax_ast::Cloze};
    ///
    /// let cloze = Org::parse("{{text}}").first_node::<Cloze>().unwrap();
    /// assert!(cloze.id().is_none());
    /// let cloze = Org::parse("{{text}@}").first_node::<Cloze>().unwrap();
    /// assert_eq!(cloze.id().unwrap(), "");
    /// let cloze = Org::parse("{{text}@id}").first_node::<Cloze>().unwrap();
    /// assert_eq!(cloze.id().unwrap(), "id");
    /// ```
    pub fn id(&self) -> Option<Token> {
        self.syntax
            .children_with_tokens()
            .skip_while(|n| n.kind() != SyntaxKind::AT)
            .nth(1)
            .and_then(|e| {
                debug_assert_eq!(e.kind(), SyntaxKind::TEXT);
                Some(Token(e.into_token()?))
            })
    }
}
