//! Public syntax and Element metadata and source-bound values.

use sha2::{Digest, Sha256};

use super::TextRange;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Classification of a syntax kind in the grammar catalog.
pub enum KindCategory {
    Node,
    Token,
}

#[derive(Clone, Copy, Debug)]
/// Name and category of one Scheme syntax kind.
pub struct KindSpec {
    pub name: &'static str,
    pub category: KindCategory,
}

#[derive(Debug)]
/// Public identity and syntax-kind catalog of a Scheme language.
pub struct LanguageSpec {
    pub language: &'static str,
    pub version: &'static str,
    pub contract: &'static str,
    pub grammar_digest: &'static str,
    pub case_insensitive: bool,
    pub root_kind: u16,
    pub kinds: &'static [KindSpec],
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Grammar, native program, and source identities for a document.
pub struct ParseReceipt {
    pub language: &'static str,
    pub version: &'static str,
    pub contract: &'static str,
    pub grammar_digest: &'static str,
    pub parser_digest: Option<&'static str>,
    pub scanner_digest: Option<&'static str>,
    pub source_digest: String,
}

impl ParseReceipt {
    pub(crate) fn new(source: &str, language: &LanguageSpec, parser_digest: &'static str) -> Self {
        Self {
            language: language.language,
            version: language.version,
            contract: language.contract,
            grammar_digest: language.grammar_digest,
            parser_digest: Some(parser_digest),
            scanner_digest: None,
            source_digest: format!("sha256:{:x}", Sha256::digest(source)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// How Scheme folds matching source spans into an Element field.
pub enum GraphFieldMode {
    Append,
    Each,
    AppendOrEmpty,
    NodeText,
    EachNodeText,
}

#[derive(Clone, Copy, Debug)]
/// Source-kind and cardinality declaration for an Element field.
pub struct GraphFieldRule {
    pub token_kind: u16,
    pub name: &'static str,
    pub mode: GraphFieldMode,
}

#[derive(Clone, Copy, Debug)]
/// Declaration of the Element produced for a syntax node.
pub struct GraphNodeRule {
    pub syntax_kind: u16,
    pub category: &'static str,
    pub kind: &'static str,
    pub fields: &'static [GraphFieldRule],
}

#[derive(Clone, Copy, Debug)]
/// Identity and schema of the Scheme Element projection.
pub struct GraphProjectionSpec {
    pub grammar_digest: &'static str,
    pub projection_digest: &'static str,
    pub rules: &'static [GraphNodeRule],
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// An Element produced by Scheme, with source range and ancestry.
pub struct GraphRecord {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub child_ids: Vec<usize>,
    pub syntax_kind: u16,
    pub category: &'static str,
    pub kind: &'static str,
    pub range: TextRange,
    pub fields: Vec<GraphFieldValue>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// One projected Element field and the range covering its source spans.
pub struct GraphFieldValue {
    pub name: &'static str,
    pub value: String,
    pub range: TextRange,
}

impl GraphRecord {
    pub fn values<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        self.fields
            .iter()
            .filter(move |field| field.name == name)
            .map(|field| field.value.as_str())
    }

    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|field| field.name == name)
            .map(|field| field.value.as_str())
    }

    pub fn field_range(&self, name: &str) -> Option<TextRange> {
        self.fields
            .iter()
            .find(|field| field.name == name)
            .map(|field| field.range)
    }
}
