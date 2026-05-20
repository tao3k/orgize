//! Parsed Org document facade.

use rowan::ast::AstNode;
use rowan::{GreenNode, TextSize};

use crate::SyntaxElement;
use crate::ast::ParsedAst;
use crate::config::ParseConfig;
use crate::export::{
    HtmlExport, HtmlExportOptions, LatexExport, LatexExportOptions, MarkdownExport,
    MarkdownExportOptions, TraversalContext, Traverser,
};
use crate::syntax::document::document_node;
use crate::syntax::{OrgLanguage, SyntaxNode};
use crate::syntax_ast;

#[derive(Debug)]
/// Parsed Org document with access to semantic and lossless syntax APIs.
pub struct Org {
    pub(crate) source: String,
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
        ParsedAst::from_syntax_tree_with_config(
            &SyntaxNode::new_root(self.green.clone()),
            &self.source,
            &self.config,
        )
    }

    /// Returns the lossless syntax-tree document.
    pub fn syntax_document(&self) -> syntax_ast::SyntaxDocument {
        syntax_ast::SyntaxDocument {
            syntax: SyntaxNode::new_root(self.green.clone()),
        }
    }

    /// Returns org-mode string
    pub fn to_org(&self) -> String {
        self.source.clone()
    }

    /// Convert org element tree to html-format using default html handler
    pub fn to_html(&self) -> String {
        let mut handler = HtmlExport::default();
        self.traverse(&mut handler);
        handler.finish()
    }

    /// Convert org element tree to html-format using explicit html options.
    pub fn to_html_with_options(&self, options: HtmlExportOptions) -> String {
        let mut handler = HtmlExport::with_options(options);
        self.traverse(&mut handler);
        handler.finish()
    }

    /// Convert org element tree to LaTeX body text using the default LaTeX handler.
    pub fn to_latex(&self) -> String {
        let mut handler = LatexExport::default();
        self.traverse(&mut handler);
        handler.finish()
    }

    /// Convert org element tree to LaTeX body text using explicit LaTeX options.
    pub fn to_latex_with_options(&self, options: LatexExportOptions) -> String {
        let mut handler = LatexExport::with_options(options);
        self.traverse(&mut handler);
        handler.finish()
    }

    /// Convert org element tree to Markdown using the default Markdown handler.
    pub fn to_markdown(&self) -> String {
        let mut handler = MarkdownExport::default();
        self.traverse(&mut handler);
        handler.finish()
    }

    /// Convert org element tree to Markdown using explicit Markdown options.
    pub fn to_markdown_with_options(&self, options: MarkdownExportOptions) -> String {
        let mut handler = MarkdownExport::with_options(options);
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

impl ParseConfig {
    /// Parses input with current config.
    pub fn parse(self, input: impl AsRef<str>) -> Org {
        let source = input.as_ref().to_string();
        let config = self.with_file_todo_keywords(&source);
        let input = (source.as_str(), &config).into();
        let node = document_node(input).unwrap().1;

        Org {
            source,
            config,
            green: node.into_node().unwrap(),
        }
    }
}
