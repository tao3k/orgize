use rowan::ast::AstNode;
use rowan::{GreenNode, TextSize};

use crate::ast::ParsedAst;
use crate::config::ParseConfig;
use crate::export::{HtmlExport, TraversalContext, Traverser};
use crate::syntax::{OrgLanguage, SyntaxNode};
use crate::syntax_ast;
use crate::SyntaxElement;

#[derive(Debug)]
pub struct Org {
    pub(crate) green: GreenNode,
    pub(crate) config: ParseConfig,
}

impl Org {
    /// Parse input string to Org element tree using default parse config
    pub fn parse(input: impl AsRef<str>) -> Org {
        ParseConfig::default().parse(input)
    }

    pub fn green(&self) -> &GreenNode {
        &self.green
    }

    pub fn config(&self) -> &ParseConfig {
        &self.config
    }

    /// Returns the owned semantic document.
    pub fn document(&self) -> ParsedAst {
        let source = self.green.to_string();
        ParsedAst::from_syntax_tree(&SyntaxNode::new_root(self.green.clone()), &source)
    }

    /// Returns the lossless syntax-tree document.
    pub fn syntax_document(&self) -> syntax_ast::Document {
        syntax_ast::Document {
            syntax: SyntaxNode::new_root(self.green.clone()),
        }
    }

    /// Returns org-mode string
    pub fn to_org(&self) -> String {
        self.green.to_string()
    }

    /// Convert org element tree to html-format using default html handler
    pub fn to_html(&self) -> String {
        let mut handler = HtmlExport::default();
        self.traverse(&mut handler);
        handler.finish()
    }

    /// Walk through org element tree using given traverser
    pub fn traverse<T: Traverser>(&self, t: &mut T) {
        let mut ctx = TraversalContext::default();
        t.element(
            SyntaxElement::Node(SyntaxNode::new_root(self.green.clone())),
            &mut ctx,
        );
    }

    /// Returns the first node in org element tree in depth first order
    pub fn first_node<N: AstNode<Language = OrgLanguage>>(&self) -> Option<N> {
        fn find<N: AstNode<Language = OrgLanguage>>(node: SyntaxNode) -> Option<N> {
            if N::can_cast(node.kind()) {
                N::cast(node)
            } else {
                node.children().find_map(find)
            }
        }
        find(SyntaxNode::new_root(self.green.clone()))
    }

    /// Returns node in given offset
    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::Headline};
    ///
    /// let org = Org::parse("\n\n* foo\n* bar");
    ///
    /// assert!(org.node_at_offset::<Headline>(0).is_none());
    ///
    /// let hdl = org.node_at_offset::<Headline>(2).unwrap();
    /// assert_eq!(hdl.title_raw(), "foo");
    ///
    /// let hdl = org.node_at_offset::<Headline>(9).unwrap();
    /// assert_eq!(hdl.title_raw(), "bar");
    ///
    /// assert!(org.node_at_offset::<Headline>(999).is_none());
    /// ```
    pub fn node_at_offset<N: AstNode<Language = OrgLanguage>>(
        &self,
        offset: impl Into<TextSize>,
    ) -> Option<N> {
        let offset = offset.into();
        fn find<N: AstNode<Language = OrgLanguage>>(
            node: SyntaxNode,
            offset: TextSize,
        ) -> Option<N> {
            if !node.text_range().contains(offset) {
                None
            } else if N::can_cast(node.kind()) {
                N::cast(node)
            } else {
                node.children().find_map(|node| find(node, offset))
            }
        }
        find(SyntaxNode::new_root(self.green.clone()), offset)
    }
}
