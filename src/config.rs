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
}

impl ParseConfig {
    /// Parses input with current config
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

    fn with_file_todo_keywords(mut self, source: &str) -> Self {
        if let Some(todo_keywords) = parse_file_todo_keywords(source) {
            self.todo_keywords = todo_keywords;
        }
        self
    }

    pub(crate) fn effective_inlinetask_min_level(&self) -> usize {
        self.inlinetask_min_level.max(1)
    }
}

fn parse_file_todo_keywords(source: &str) -> Option<(Vec<String>, Vec<String>)> {
    let mut todo = Vec::new();
    let mut done = Vec::new();
    let mut in_block = false;

    for line in source.lines() {
        let trimmed = line.trim_start_matches([' ', '\t']);
        if in_block {
            if is_keyword_line_with_prefix(trimmed, "end_") {
                in_block = false;
            }
            continue;
        }

        if is_keyword_line_with_prefix(trimmed, "begin_") {
            in_block = true;
            continue;
        }

        if let Some(value) = todo_declaration_value(line) {
            collect_todo_keywords(value, &mut todo, &mut done);
        }
    }

    (!todo.is_empty() || !done.is_empty()).then_some((todo, done))
}

fn todo_declaration_value(line: &str) -> Option<&str> {
    let line = line.trim_start_matches([' ', '\t']);
    let rest = line.strip_prefix("#+")?;
    let (key, value) = rest.split_once(':')?;
    matches!(
        key,
        key if key.eq_ignore_ascii_case("TODO")
            || key.eq_ignore_ascii_case("SEQ_TODO")
            || key.eq_ignore_ascii_case("TYP_TODO")
    )
    .then_some(value)
}

fn is_keyword_line_with_prefix(line: &str, prefix: &str) -> bool {
    let Some(rest) = line.strip_prefix("#+") else {
        return false;
    };
    rest.get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
}

fn collect_todo_keywords(value: &str, todo: &mut Vec<String>, done: &mut Vec<String>) {
    let mut done_side = false;

    for token in value.split_whitespace() {
        if token == "|" {
            done_side = true;
            continue;
        }

        let Some(keyword) = todo_keyword_name(token) else {
            continue;
        };
        let keywords = if done_side { &mut *done } else { &mut *todo };
        if !keywords.iter().any(|existing| existing == &keyword) {
            keywords.push(keyword);
        }
    }
}

fn todo_keyword_name(token: &str) -> Option<String> {
    let token = token.trim();
    if token.is_empty() || token.starts_with('(') {
        return None;
    }

    let name = token
        .split_once('(')
        .map(|(name, _)| name)
        .unwrap_or(token)
        .trim();

    (!name.is_empty() && name != "|").then(|| name.to_string())
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
            radio_link_projection: RadioLinkProjection::PlainText,
            inlinetask_min_level: 15,
        }
    }
}
