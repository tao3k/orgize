use rowan::ast::AstNode;

use super::{AffiliatedKeyword, SyntaxLink, Token, token};
use crate::{SyntaxElement, SyntaxNode, syntax::SyntaxKind};

impl SyntaxLink {
    /// Returns link destination
    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::SyntaxLink};
    ///
    /// let link = Org::parse("[[#id]]").first_node::<SyntaxLink>().unwrap();
    /// assert_eq!(link.path(), "#id");
    /// let link = Org::parse("[[https://google.com]]").first_node::<SyntaxLink>().unwrap();
    /// assert_eq!(link.path(), "https://google.com");
    /// let link = Org::parse("[[https://google.com][Google]]").first_node::<SyntaxLink>().unwrap();
    /// assert_eq!(link.path(), "https://google.com");
    /// ```
    pub fn path(&self) -> Token {
        token(&self.syntax, SyntaxKind::LINK_PATH).expect("link must contains LINK_PATH")
    }

    /// Returns `true` if link contains description
    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::SyntaxLink};
    ///
    /// let link = Org::parse("[[https://google.com]]").first_node::<SyntaxLink>().unwrap();
    /// assert!(!link.has_description());
    /// let link = Org::parse("[[https://google.com][Google]]").first_node::<SyntaxLink>().unwrap();
    /// assert!(link.has_description());
    /// let link = Org::parse("[[https://example.com][*abc* /abc/]]").first_node::<SyntaxLink>().unwrap();
    /// assert!(link.has_description());
    /// ```
    pub fn has_description(&self) -> bool {
        self.syntax()
            .children_with_tokens()
            .any(|e| e.kind() == SyntaxKind::L_BRACKET)
    }

    /// Returns parsed description
    ///
    /// Returns empty iterator if this link doesn't contain description
    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::SyntaxLink, SyntaxKind};
    ///
    /// let link = Org::parse("[[https://google.com]]").first_node::<SyntaxLink>().unwrap();
    /// assert_eq!(link.description().count(), 0);
    ///
    /// let link = Org::parse("[[https://google.com][Google]]").first_node::<SyntaxLink>().unwrap();
    /// let description = link.description().collect::<Vec<_>>();
    /// assert_eq!((description[0].kind(), description[0].to_string()), (SyntaxKind::TEXT, "Google".into()));
    ///
    /// let link = Org::parse("[[https://example.com][*abc* /abc/]]").first_node::<SyntaxLink>().unwrap();
    /// let description = link.description().collect::<Vec<_>>();
    /// assert_eq!((description[0].kind(), description[0].to_string()), (SyntaxKind::BOLD, "*abc*".into()));
    /// assert_eq!((description[2].kind(), description[2].to_string()), (SyntaxKind::ITALIC, "/abc/".into()));
    /// ```
    pub fn description(&self) -> impl Iterator<Item = SyntaxElement> + use<> {
        self.syntax()
            .children_with_tokens()
            .skip_while(|e| e.kind() != SyntaxKind::L_BRACKET)
            .skip(1)
            .take_while(|e| e.kind() != SyntaxKind::R_BRACKET2)
    }

    /// Returns description raw string
    ///
    /// Returns empty string if this link doesn't contain description
    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::SyntaxLink};
    ///
    /// let link = Org::parse("[[https://google.com]]").first_node::<SyntaxLink>().unwrap();
    /// assert_eq!(link.description_raw(), "");
    /// let link = Org::parse("[[https://google.com][Google]]").first_node::<SyntaxLink>().unwrap();
    /// assert_eq!(link.description_raw(), "Google");
    /// let link = Org::parse("[[https://example.com][*abc* /abc/]]").first_node::<SyntaxLink>().unwrap();
    /// assert_eq!(link.description_raw(), "*abc* /abc/");
    /// ```
    pub fn description_raw(&self) -> String {
        self.description()
            .fold(String::new(), |acc, e| acc + &e.to_string())
    }

    /// Returns `true` if link is an image link
    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::SyntaxLink};
    ///
    /// let link = Org::parse("[[https://google.com]]").first_node::<SyntaxLink>().unwrap();
    /// assert!(!link.is_image());
    /// let link = Org::parse("[[file:/home/dominik/images/jupiter.jpg]]").first_node::<SyntaxLink>().unwrap();
    /// assert!(link.is_image());
    /// ```
    pub fn is_image(&self) -> bool {
        const IMAGE_SUFFIX: &[&str] = &[
            // https://github.com/bzg/org-mode/blob/7de1e818d5fbe6a05c6b1a007eed07dc27e7246b/lisp/ox.el#L253
            ".png", ".jpeg", ".jpg", ".gif", ".tiff", ".tif", ".xbm", ".xpm", ".pbm", ".pgm",
            ".ppm", ".webp", ".avif", ".svg",
        ];

        let path = self.path();

        IMAGE_SUFFIX.iter().any(|e| path.ends_with(e)) && !self.has_description()
    }

    /// Returns caption keyword in this link
    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::SyntaxLink};
    ///
    /// let link = Org::parse("#+CAPTION: image link\n[[file:/home/dominik/images/jupiter.jpg]]").first_node::<SyntaxLink>().unwrap();
    /// assert_eq!(link.caption().unwrap().value().unwrap(), " image link");
    /// let link = Org::parse("#+CAPTION: quoted image\n#+begin_quote\n[[file:plot.png]]\n#+end_quote").first_node::<SyntaxLink>().unwrap();
    /// assert_eq!(link.caption().unwrap().value().unwrap(), " quoted image");
    /// ```
    pub fn caption(&self) -> Option<AffiliatedKeyword> {
        self.syntax
            .ancestors()
            .skip(1)
            .find_map(|node| caption_keyword(&node))
    }
}

fn caption_keyword(node: &SyntaxNode) -> Option<AffiliatedKeyword> {
    node.children()
        .take_while(|node| node.kind() == SyntaxKind::AFFILIATED_KEYWORD)
        .filter_map(AffiliatedKeyword::cast)
        .find(|keyword| keyword.key() == "CAPTION")
}
