use super::{
    CenterBlock, CommentBlock, DynBlock, ExampleBlock, ExportBlock, QuoteBlock, SourceBlock,
    SpecialBlock, SyntaxKind, Token, VerseBlock, filter_token,
};
use rowan::TextSize;

impl SourceBlock {
    /// ```rust
    /// use orgize::{Org, syntax_ast::SourceBlock};
    ///
    /// let block = Org::parse("#+begin_src c\n#+end_src").first_node::<SourceBlock>().unwrap();
    /// assert_eq!(block.language().unwrap(), "c");
    /// let block = Org::parse("#+begin_src javascript \n#+end_src").first_node::<SourceBlock>().unwrap();
    /// assert_eq!(block.language().unwrap(), "javascript");
    ///
    /// let block = Org::parse("#+begin_src\n#+end_src").first_node::<SourceBlock>().unwrap();
    /// assert!(block.language().is_none());
    /// ````
    pub fn language(&self) -> Option<Token> {
        self.syntax
            .children()
            .find(|e| e.kind() == SyntaxKind::BLOCK_BEGIN)
            .into_iter()
            .flat_map(|n| n.children_with_tokens())
            .find_map(filter_token(SyntaxKind::SRC_BLOCK_LANGUAGE))
    }

    /// ```rust
    /// use orgize::{Org, syntax_ast::SourceBlock};
    ///
    /// let block = Org::parse("#+begin_src emacs-lisp -n 20\n#+end_src").first_node::<SourceBlock>().unwrap();
    /// assert_eq!(block.switches().unwrap(), "-n 20");
    /// let block = Org::parse("#+begin_src emacs-lisp -n 20 -r :tangle yes \n#+end_src").first_node::<SourceBlock>().unwrap();
    /// assert_eq!(block.switches().unwrap(), "-n 20 -r");
    ///
    /// let block = Org::parse("#+begin_src emacs-lisp\n#+end_src").first_node::<SourceBlock>().unwrap();
    /// assert!(block.switches().is_none());
    /// let block = Org::parse("#+begin_src\n#+end_src").first_node::<SourceBlock>().unwrap();
    /// assert!(block.switches().is_none());
    /// let block = Org::parse("#+begin_src :tangle yes\n#+end_src").first_node::<SourceBlock>().unwrap();
    /// assert!(block.switches().is_none());
    /// ````
    pub fn switches(&self) -> Option<Token> {
        self.syntax
            .children()
            .find(|e| e.kind() == SyntaxKind::BLOCK_BEGIN)
            .into_iter()
            .flat_map(|n| n.children_with_tokens())
            .find_map(filter_token(SyntaxKind::SRC_BLOCK_SWITCHES))
    }

    /// ```rust
    /// use orgize::{Org, syntax_ast::SourceBlock};
    ///
    /// let block = Org::parse("#+begin_src c :tangle yes\n#+end_src").first_node::<SourceBlock>().unwrap();
    /// assert_eq!(block.parameters().unwrap(), ":tangle yes");
    /// let block = Org::parse("#+begin_src c :tangle   \n#+end_src").first_node::<SourceBlock>().unwrap();
    /// assert_eq!(block.parameters().unwrap(), ":tangle");
    ///
    /// let block = Org::parse("#+begin_src c\n#+end_src").first_node::<SourceBlock>().unwrap();
    /// assert!(block.parameters().is_none());
    /// ````
    pub fn parameters(&self) -> Option<Token> {
        self.syntax
            .children()
            .find(|e| e.kind() == SyntaxKind::BLOCK_BEGIN)
            .into_iter()
            .flat_map(|n| n.children_with_tokens())
            .find_map(filter_token(SyntaxKind::SRC_BLOCK_PARAMETERS))
    }

    /// Return unescaped source code string
    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::SourceBlock};
    ///
    /// let block = Org::parse(r#"
    /// #+begin_src
    /// #+end_src
    /// "#).first_node::<SourceBlock>().unwrap();
    /// assert_eq!(block.value(), "");
    ///
    /// let block = Org::parse(r#"
    /// #+begin_src
    /// ,* foo
    /// ,#+ bar
    /// #+end_src
    /// "#).first_node::<SourceBlock>().unwrap();
    /// assert_eq!(block.value(), "* foo\n#+ bar\n");
    /// ````
    pub fn value(&self) -> String {
        self.syntax
            .children()
            .find(|e| e.kind() == SyntaxKind::BLOCK_CONTENT)
            .into_iter()
            .flat_map(|n| n.children_with_tokens())
            .filter_map(filter_token(SyntaxKind::TEXT))
            .fold(String::new(), |acc, value| acc + &value)
    }
}

impl ExampleBlock {
    /// ```rust
    /// use orgize::{Org, syntax_ast::ExampleBlock};
    ///
    /// let block = Org::parse("#+begin_example -n 3\n#+end_example").first_node::<ExampleBlock>().unwrap();
    /// assert_eq!(block.switches().unwrap(), "-n 3");
    ///
    /// let block = Org::parse("#+begin_example\n#+end_example").first_node::<ExampleBlock>().unwrap();
    /// assert!(block.switches().is_none());
    /// ````
    pub fn switches(&self) -> Option<Token> {
        self.syntax
            .children()
            .find(|e| e.kind() == SyntaxKind::BLOCK_BEGIN)
            .into_iter()
            .flat_map(|n| n.children_with_tokens())
            .find_map(filter_token(SyntaxKind::SRC_BLOCK_SWITCHES))
    }
}

impl ExportBlock {
    /// ```rust
    /// use orgize::{Org, syntax_ast::ExportBlock};
    ///
    /// let block = Org::parse("#+begin_export html\n#+end_export").first_node::<ExportBlock>().unwrap();
    /// assert_eq!(block.ty().unwrap(), "html");
    ///
    /// let block = Org::parse("#+begin_export\n#+end_export").first_node::<ExportBlock>().unwrap();
    /// assert!(block.ty().is_none());
    /// ````
    pub fn ty(&self) -> Option<Token> {
        self.syntax
            .children()
            .find(|e| e.kind() == SyntaxKind::BLOCK_BEGIN)
            .into_iter()
            .flat_map(|n| n.children_with_tokens())
            .find_map(filter_token(SyntaxKind::EXPORT_BLOCK_TYPE))
    }

    /// Returns export block contents
    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::ExportBlock};
    ///
    /// let block = Org::parse(r#"
    /// #+begin_export html
    /// <style>.red { color: red; }</style>
    /// #+end_export
    /// "#).first_node::<ExportBlock>().unwrap();
    /// assert_eq!(block.value(), "<style>.red { color: red; }</style>\n");
    ///
    /// let block = Org::parse(r#"
    /// #+BEGIN_EXPORT org
    /// ,#+BEGIN_EXPORT html
    /// <style>.red { color: red; }</style>
    /// ,#+END_EXPORT
    /// #+END_EXPORT
    /// "#).first_node::<ExportBlock>().unwrap();
    /// assert_eq!(block.value(), r#"#+BEGIN_EXPORT html
    /// <style>.red { color: red; }</style>
    /// #+END_EXPORT
    /// "#);
    /// ```
    pub fn value(&self) -> String {
        self.syntax
            .children()
            .find(|e| e.kind() == SyntaxKind::BLOCK_CONTENT)
            .into_iter()
            .flat_map(|n| n.children_with_tokens())
            .filter_map(filter_token(SyntaxKind::TEXT))
            .fold(String::new(), |acc, value| acc + &value)
    }
}

macro_rules! impl_content_border {
    ($block:ident) => {
        impl $block {
            /// Beginning position of block content
            pub fn content_start(&self) -> TextSize {
                self.syntax
                    .children()
                    .find(|n| n.kind() == SyntaxKind::BLOCK_BEGIN)
                    .map(|n| n.text_range().end())
                    .unwrap_or_else(|| {
                        debug_assert!(false, "block must contains BLOCK_BEGIN");
                        TextSize::default()
                    })
            }

            /// Ending position of block content
            pub fn content_end(&self) -> TextSize {
                self.syntax
                    .children()
                    .find(|n| n.kind() == SyntaxKind::BLOCK_END)
                    .map(|n| n.text_range().start())
                    .unwrap_or_else(|| {
                        debug_assert!(false, "block must contains BLOCK_END");
                        TextSize::default()
                    })
            }
        }
    };
}

impl_content_border!(SourceBlock);
impl_content_border!(ExportBlock);
impl_content_border!(CenterBlock);
impl_content_border!(CommentBlock);
impl_content_border!(ExampleBlock);
impl_content_border!(QuoteBlock);
impl_content_border!(SpecialBlock);
impl_content_border!(VerseBlock);
impl_content_border!(DynBlock);
