//! Parser configuration for Org syntax and semantic projection.

use crate::syntax::document::document_node;
use crate::Org;

#[derive(Clone, Debug)]
/// Controls Org subscript and superscript parsing.
pub enum UseSubSuperscript {
    /// Disable subscript and superscript parsing.
    Nil,
    /// Parse only braced subscript and superscript forms.
    Brace,
    /// Parse subscript and superscript forms.
    True,
}

impl UseSubSuperscript {
    pub fn is_nil(&self) -> bool {
        matches!(self, UseSubSuperscript::Nil)
    }

    pub fn is_true(&self) -> bool {
        matches!(self, UseSubSuperscript::True)
    }

    pub fn is_brace(&self) -> bool {
        matches!(self, UseSubSuperscript::Brace)
    }
}

/// Parse configuration
#[derive(Clone, Debug)]
pub struct ParseConfig {
    /// Headline's todo keywords
    pub todo_keywords: (Vec<String>, Vec<String>),

    pub dual_keywords: Vec<String>,

    pub parsed_keywords: Vec<String>,

    /// Control sub/superscript parsing
    ///
    /// Equivalent to `org-use-sub-superscripts`
    ///
    /// - `UseSubSuperscript::Nil`: disable parsing
    /// - `UseSubSuperscript::True`: enable parsing
    /// - `UseSubSuperscript::Brace`: enable parsing, but braces are required
    pub use_sub_superscript: UseSubSuperscript,

    /// Affiliated keywords
    ///
    /// Equivalent to [`org-element-affiliated-keywords`](https://git.sr.ht/~bzg/org-mode/tree/6f960f3c6a4dfe137fbd33fef9f7dadfd229600c/item/lisp/org-element.el#L331)
    pub affiliated_keywords: Vec<String>,
}

impl ParseConfig {
    /// Parses input with current config
    pub fn parse(self, input: impl AsRef<str>) -> Org {
        let source = input.as_ref().to_string();
        let input = (source.as_str(), &self).into();
        let node = document_node(input).unwrap().1;

        Org {
            source,
            config: self,
            green: node.into_node().unwrap(),
        }
    }
}

impl Default for ParseConfig {
    fn default() -> Self {
        ParseConfig {
            todo_keywords: (vec!["TODO".into()], vec!["DONE".into()]),
            dual_keywords: vec!["CAPTION".into(), "RESULTS".into()],
            parsed_keywords: vec!["CAPTION".into()],
            use_sub_superscript: UseSubSuperscript::True,
            affiliated_keywords: vec![
                "CAPTION".into(),
                "DATA".into(),
                "HEADER".into(),
                "HEADERS".into(),
                "LABEL".into(),
                "NAME".into(),
                "PLOT".into(),
                "RESNAME".into(),
                "RESULT".into(),
                "RESULTS".into(),
                "SOURCE".into(),
                "SRCNAME".into(),
                "TBLNAME".into(),
            ],
        }
    }
}
