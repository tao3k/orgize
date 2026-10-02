//! Parser configuration for Org syntax and semantic projection.

#[allow(dead_code)] // The full Scheme catalog is generated together; parsing uses this slice first.
#[path = "../languages/org/v1/generated/elements.rs"]
mod org_elements;

pub(crate) fn org_affiliated_keyword_names() -> &'static [&'static str] {
    org_elements::ORG_AFFILIATED_KEYWORDS
}

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

/// Controls how semantic radio links are projected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadioLinkProjection {
    /// Link plain text segments against collected `<<<radio targets>>>`.
    PlainText,
    /// Link parsed object spans such as `*marked up*` or `\alpha` against
    /// collected radio targets.
    Semantic,
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

    /// Semantic radio-link projection mode.
    ///
    /// `PlainText` preserves the historical lightweight behavior. `Semantic`
    /// performs an opt-in second semantic pass over parsed object spans so
    /// radio targets containing markup or entities can be linked without
    /// changing the lossless syntax tree.
    pub radio_link_projection: RadioLinkProjection,

    /// Minimum headline level parsed as an inlinetask.
    ///
    /// This mirrors `org-inlinetask-min-level`; Org's default is 15.
    pub inlinetask_min_level: usize,

    /// Tab width used when deriving normalized source/example/fixed-width lines.
    ///
    /// This mirrors `org-src-tab-width`; Org's default is 4.
    pub src_tab_width: usize,

    /// Preserve source/example/fixed-width indentation during normalized projection.
    ///
    /// This mirrors `org-src-preserve-indentation`; block-level `-i` also
    /// enables preservation for a single source/example block.
    pub src_preserve_indentation: bool,
}

impl ParseConfig {
    pub(crate) fn effective_inlinetask_min_level(&self) -> usize {
        self.inlinetask_min_level.max(1)
    }

    pub(crate) fn inline_script_policy(&self) -> usize {
        match self.use_sub_superscript {
            UseSubSuperscript::Nil => 0,
            UseSubSuperscript::Brace => 1,
            UseSubSuperscript::True => 2,
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
            affiliated_keywords: org_elements::ORG_AFFILIATED_KEYWORDS
                .iter()
                .map(|keyword| (*keyword).to_string())
                .collect(),
            radio_link_projection: RadioLinkProjection::PlainText,
            inlinetask_min_level: 15,
            src_tab_width: 4,
            src_preserve_indentation: false,
        }
    }
}
