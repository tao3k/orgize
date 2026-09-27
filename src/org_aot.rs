//! Cargo-only Org parsing from Scheme-declared AOT language artifacts.
//!
//! The default AOT entrypoint runs the Org-owned Scheme event algorithm.
//! The public `Org` facade uses this parser; the owned semantic AST is a
//! projection of its Scheme-generated Element graph.

use std::{collections::HashMap, sync::OnceLock};

use gerbil_parser_rowan::{
    Diagnostic, GraphProjectionSpec, GraphRecord, LanguageSpec, Parse, ParseError, ParseReceipt,
    SyntaxNode, parse_generated_events, project_syntax_graph,
};
use rowan::TextRange;

use crate::config::ParseConfig;
use crate::contract_feature::{
    ContractExecutionError, ContractPack, ContractResult, ContractRule, ContractScopeNodeId,
    evaluate_contract,
};

#[rustfmt::skip]
#[path = "../languages/org/v1/generated/parser.rs"]
mod grammar;
#[rustfmt::skip]
#[path = "../languages/org/v1/generated/graph.rs"]
mod graph;
#[path = "org_aot_citation_functions.rs"]
mod citation_functions;
#[path = "org_aot_contract_plan.rs"]
mod contract_plan;
#[path = "org_aot_headline_functions.rs"]
mod headline_functions;
#[path = "org_aot_headline_view.rs"]
mod headline_view;
pub use headline_view::OrgHeadline;
#[path = "org_aot_affiliation.rs"]
mod affiliation;
#[path = "org_aot_link_functions.rs"]
mod link_functions;
#[path = "org_aot_todo_directive.rs"]
mod todo_directive;
#[rustfmt::skip]
#[path = "org_aot_events.rs"]
mod generated_context_events;

/// A source-backed, lossless Rowan tree and its Scheme-declared Element graph.
#[derive(Debug)]
pub struct OrgAotDocument {
    parse: Parse,
    records: Vec<GraphRecord>,
    base_config: ParseConfig,
    config: ParseConfig,
    todo_directives: Vec<String>,
    configured_todo: Vec<String>,
    configured_done: Vec<String>,
    headline_properties: Vec<Option<HeadlineProperties>>,
    subtree_end: Vec<usize>,
    affiliations: OnceLock<HashMap<usize, Vec<usize>>>,
}

#[derive(Debug)]
struct HeadlineProperties {
    todo_type: Option<&'static str>,
    details: OnceLock<HeadlineDetails>,
}

#[derive(Debug)]
struct HeadlineDetails {
    todo_keyword: Option<String>,
    content_after_todo: String,
    priority_cookie: Option<String>,
    display_title: String,
    source_title: String,
    is_comment: bool,
}

/// An error from the parser or the generated Element projection contract.
#[derive(Debug)]
pub enum OrgAotError {
    /// The generated parser rejected the source.
    Parse(ParseError),
    /// The generated Element projection did not match the parser artifact.
    Projection(Diagnostic),
}

/// Parse Org source through the Scheme-generated event algorithm and Element projection.
///
/// No Gerbil runtime or package is needed by a Cargo consumer.
///
/// # Errors
///
/// Returns the parser receipt on parse failure, or a projection diagnostic if
/// the generated graph table is stale or invalid.
pub fn parse_org_aot(source: &str) -> Result<OrgAotDocument, OrgAotError> {
    static DEFAULT_CONFIG: OnceLock<ParseConfig> = OnceLock::new();
    parse_org_aot_events(
        source,
        generated_context_events::parse_org_rowan_events(source),
        DEFAULT_CONFIG.get_or_init(ParseConfig::default),
    )
}

/// Parse Org with the same Scheme-generated algorithm and a caller-provided configuration.
///
/// # Errors
///
/// Returns the parser receipt on parse failure, or a projection diagnostic if
/// the generated graph table is stale or invalid.
pub fn parse_org_aot_with_config(
    source: &str,
    config: &ParseConfig,
) -> Result<OrgAotDocument, OrgAotError> {
    let events = generated_context_events::parse_org_rowan_events_with_parameters(
        source,
        config.effective_inlinetask_min_level(),
        config.inline_script_policy(),
    );
    parse_org_aot_events(source, events, config)
}

fn parse_org_aot_events(
    source: &str,
    events: Vec<gerbil_parser_rowan::TreeEvent>,
    config: &ParseConfig,
) -> Result<OrgAotDocument, OrgAotError> {
    let parse = parse_generated_events(
        &grammar::LANGUAGE,
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .map_err(OrgAotError::Parse)?;
    document_from_parse(parse, config)
}

pub(crate) fn org_image_link(target: &str) -> bool {
    link_functions::org_image_link_p(target)
}

pub(crate) fn org_link_kind(path: &str) -> &'static str {
    link_functions::org_link_kind(path)
}

pub(crate) fn org_link_target_key(path: &str) -> &str {
    link_functions::org_link_target_key(path)
}

pub(crate) fn org_link_protocol(path: &str) -> &str {
    link_functions::org_link_protocol(path)
}

pub(crate) fn org_link_protocol_path(path: &str) -> &str {
    link_functions::org_link_protocol_path(path)
}

pub(crate) fn org_link_file_path(path: &str) -> &str {
    link_functions::org_link_file_path(path)
}

pub(crate) fn org_link_search(path: &str) -> &str {
    link_functions::org_link_search(path)
}

pub(crate) fn org_link_file_path_kind(path: &str) -> &'static str {
    link_functions::org_link_file_path_kind(path)
}

pub(crate) fn org_link_search_kind(search: &str) -> &'static str {
    link_functions::org_link_search_kind(search)
}

pub(crate) fn org_link_search_value(search: &str) -> &str {
    link_functions::org_link_search_value(search)
}

pub(crate) fn org_expand_link_abbreviation(
    replacement: &str,
    path: &str,
    encoded_path: &str,
) -> String {
    link_functions::org_expand_link_abbreviation(replacement, path, encoded_path)
}

fn document_from_parse(parse: Parse, config: &ParseConfig) -> Result<OrgAotDocument, OrgAotError> {
    let records = project_syntax_graph(&grammar::LANGUAGE, &graph::GRAPH, &parse.syntax())
        .map_err(OrgAotError::Projection)?;
    let todo_directives = records
        .iter()
        .filter(|record| record.kind == "keyword")
        .filter(|record| {
            record
                .field("key")
                .is_some_and(todo_directive::todo_directive_p)
        })
        .filter_map(|record| record.field("value").map(str::to_owned))
        .collect::<Vec<_>>();
    let mut effective_config = config.clone();
    if !todo_directives.is_empty() {
        let mut open = Vec::new();
        let mut done = Vec::new();
        for directive in &todo_directives {
            open.extend(
                headline_functions::todo_open_words(directive)
                    .into_iter()
                    .filter(|word| !word.is_empty()),
            );
            done.extend(
                headline_functions::todo_done_words(directive)
                    .into_iter()
                    .filter(|word| !word.is_empty()),
            );
        }
        effective_config.todo_keywords = (open, done);
    }
    let headline_properties = records
        .iter()
        .map(|record| {
            let title = record
                .field("title")
                .filter(|_| matches!(record.kind, "headline" | "inlinetask"))?;
            let todo_type = match headline_functions::todo_state_from_directives(
                title,
                &todo_directives,
                &config.todo_keywords.0,
                &config.todo_keywords.1,
            ) {
                "" => None,
                state => Some(state),
            };
            Some(HeadlineProperties {
                todo_type,
                details: OnceLock::new(),
            })
        })
        .collect();
    let mut subtree_end: Vec<usize> = (1..=records.len()).collect();
    for record in records.iter().rev() {
        if let Some(parent) = record.parent_id {
            subtree_end[parent] = subtree_end[parent].max(subtree_end[record.id]);
        }
    }
    Ok(OrgAotDocument {
        parse,
        records,
        base_config: config.clone(),
        config: effective_config,
        todo_directives,
        configured_todo: config.todo_keywords.0.clone(),
        configured_done: config.todo_keywords.1.clone(),
        headline_properties,
        subtree_end,
        affiliations: OnceLock::new(),
    })
}

/// The generated Org grammar, including syntax-kind names and its digest.
#[must_use]
pub fn org_language_spec() -> &'static LanguageSpec {
    &grammar::LANGUAGE
}

/// Digest of the Scheme-generated event algorithm used by `parse_org_aot`.
#[must_use]
pub fn org_event_parser_digest() -> &'static str {
    generated_context_events::PARSER_DIGEST
}

/// The generated Org Element graph contract and its projection digest.
#[must_use]
pub fn org_graph_spec() -> &'static GraphProjectionSpec {
    &graph::GRAPH
}

/// The Scheme-AOT Org Contract pack tied to the generated Element graph.
#[must_use]
pub fn org_contract_pack() -> &'static ContractPack {
    &contract_plan::CONTRACTS
}

impl OrgAotDocument {
    /// Parse Org through the Scheme-generated algorithm with default configuration.
    #[must_use]
    pub fn parse(source: impl AsRef<str>) -> Self {
        Self::try_parse(source).expect("Scheme-generated Org parser rejected input")
    }

    /// Parse Org with structured AOT diagnostics.
    ///
    /// # Errors
    /// Returns a parser or graph-projection error if the generated artifacts reject the source.
    pub fn try_parse(source: impl AsRef<str>) -> Result<Self, OrgAotError> {
        parse_org_aot(source.as_ref())
    }

    /// Return exact source text reconstructed from the lossless Rowan graph.
    #[must_use]
    pub fn to_org(&self) -> String {
        self.syntax().to_string()
    }

    /// Return the configuration used for this document's AOT parse.
    #[must_use]
    pub fn config(&self) -> &ParseConfig {
        &self.config
    }

    /// Iterate file-level keyword Elements from the Scheme AOT graph.
    pub fn keywords(&self) -> impl Iterator<Item = &GraphRecord> {
        self.records
            .iter()
            .filter(|record| record.kind == "keyword" && !self.record_within_headline(record.id))
    }

    /// Join file-level `#+TITLE` values in source order.
    #[must_use]
    pub fn title(&self) -> Option<String> {
        self.keywords()
            .filter(|record| {
                record
                    .field("key")
                    .is_some_and(|key| key.eq_ignore_ascii_case("TITLE"))
            })
            .filter_map(|record| record.field("value"))
            .fold(None, |acc: Option<String>, value| {
                let mut title = acc.unwrap_or_default();
                if !title.is_empty() {
                    title.push(' ');
                }
                title.push_str(value.trim());
                Some(title)
            })
    }

    fn record_within_headline(&self, id: usize) -> bool {
        let mut parent = self.records[id].parent_id;
        while let Some(ancestor) = parent {
            if matches!(self.records[ancestor].kind, "headline" | "inlinetask") {
                return true;
            }
            parent = self.records[ancestor].parent_id;
        }
        false
    }

    /// Replace a UTF-8-aligned byte range and reparse through the Scheme AOT engine.
    pub fn replace_range(&mut self, range: TextRange, replacement: impl AsRef<str>) {
        let mut source = self.to_org();
        source.replace_range(
            usize::from(range.start())..usize::from(range.end()),
            replacement.as_ref(),
        );
        *self = parse_org_aot_with_config(&source, &self.base_config)
            .expect("Scheme-generated Org parser rejected edited source");
    }

    fn headline_details(&self, record_id: usize) -> Option<&HeadlineDetails> {
        let properties = self.headline_properties.get(record_id)?.as_ref()?;
        Some(properties.details.get_or_init(|| {
            let record = &self.records[record_id];
            let title = record.field("title").expect("projected headline title");
            let todo_keyword = headline_functions::todo_keyword_from_directives(
                title,
                &self.todo_directives,
                &self.configured_todo,
                &self.configured_done,
            );
            let content_after_todo = headline_functions::headline_content_after_todo(
                title,
                &self.todo_directives,
                &self.configured_todo,
                &self.configured_done,
            );
            let display_title = headline_functions::headline_display_title(
                &content_after_todo,
                record.field("tag").is_some(),
            );
            let source_title = headline_functions::headline_source_title(
                record.field("title-body").expect("projected source title"),
                &todo_keyword,
            );
            let priority_cookie = headline_functions::headline_priority_cookie(&content_after_todo);
            let is_comment = headline_functions::headline_comment_p(&display_title);
            HeadlineDetails {
                todo_keyword: (!todo_keyword.is_empty()).then_some(todo_keyword),
                content_after_todo,
                priority_cookie: (!priority_cookie.is_empty()).then_some(priority_cookie),
                display_title,
                source_title,
                is_comment,
            }
        }))
    }

    pub(crate) fn headline_todo_keyword_matches(&self, record_id: usize, expected: &str) -> bool {
        self.headline_details(record_id)
            .and_then(|details| details.todo_keyword.as_deref())
            .is_some_and(|keyword| keyword == expected)
    }

    pub(crate) fn element_subtree_end(&self, record_id: usize) -> Option<usize> {
        self.subtree_end.get(record_id).copied()
    }

    /// Return the lossless Rowan root; its text equals the parsed source.
    #[must_use]
    pub fn syntax(&self) -> SyntaxNode {
        self.parse.syntax()
    }

    /// Return the parser-owned identity receipt for this source.
    #[must_use]
    pub fn receipt(&self) -> &ParseReceipt {
        self.parse.receipt()
    }

    /// Return the ordered, parent-linked Org Element records.
    #[must_use]
    pub fn records(&self) -> &[GraphRecord] {
        &self.records
    }

    /// Return keyword record IDs attached to this Element by the Scheme-owned policy.
    ///
    /// Association is computed lazily in one graph pass, so ordinary parsing
    /// does not pay for affiliated-keyword projections it never queries.
    #[must_use]
    pub fn affiliated_keyword_ids(&self, record_id: usize) -> &[usize] {
        self.affiliations
            .get_or_init(|| affiliation::project(&self.records, &self.syntax().to_string()))
            .get(&record_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Query a headline's Scheme-owned TODO type from keyword Elements and
    /// configured states. File-local declarations override the configuration.
    #[must_use]
    pub fn headline_todo_type(&self, record_id: usize) -> Option<&'static str> {
        self.headline_properties
            .get(record_id)
            .and_then(Option::as_ref)
            .and_then(|properties| properties.todo_type)
    }

    /// Classify admitted headline, planning and inherited archive facts with
    /// the Scheme-owned lifecycle function compiled into this Cargo package.
    pub(crate) fn headline_memory_state(
        &self,
        record_id: usize,
        closed: bool,
        planned: bool,
        archived: bool,
    ) -> &'static str {
        headline_functions::memory_headline_state(
            self.headline_todo_type(record_id).unwrap_or(""),
            closed,
            planned,
            archived,
        )
    }

    /// Return the TODO keyword recognized by the Scheme AOT algorithm.
    #[must_use]
    pub fn headline_todo_keyword(&self, record_id: usize) -> Option<String> {
        self.headline_details(record_id)
            .and_then(|details| details.todo_keyword.clone())
    }

    /// Return headline content after a Scheme-recognized TODO keyword.
    /// Priority and tags are retained until their Element projections run.
    #[must_use]
    pub fn headline_content_after_todo(&self, record_id: usize) -> Option<String> {
        self.headline_details(record_id)
            .map(|details| details.content_after_todo.clone())
    }

    /// Return the Scheme-AOT headline display title after TODO and decorations.
    #[must_use]
    pub fn headline_display_title(&self, record_id: usize) -> Option<String> {
        self.headline_details(record_id)
            .map(|details| details.display_title.clone())
    }

    /// Return the Scheme-AOT title with source whitespace before tags intact.
    #[must_use]
    pub fn headline_source_title(&self, record_id: usize) -> Option<String> {
        self.headline_details(record_id)
            .map(|details| details.source_title.clone())
    }

    pub(crate) fn planning_key_kind(key: &str) -> &'static str {
        headline_functions::planning_key_kind(key)
    }

    pub(crate) fn citation_style(head: &str) -> String {
        citation_functions::citation_style(head)
    }

    pub(crate) fn citation_variant(head: &str) -> String {
        citation_functions::citation_variant(head).to_owned()
    }

    /// Return a priority cookie admitted by the Scheme-AOT headline algorithm.
    #[must_use]
    pub fn headline_priority_cookie(&self, record_id: usize) -> Option<String> {
        self.headline_details(record_id)
            .and_then(|details| details.priority_cookie.clone())
    }

    /// Whether a headline carries Org's case-sensitive COMMENT marker.
    #[must_use]
    pub fn headline_is_comment(&self, record_id: usize) -> Option<bool> {
        self.headline_details(record_id)
            .map(|details| details.is_comment)
    }

    /// Evaluate a Scheme-AOT Org Contract against this document's Element graph.
    ///
    /// # Errors
    ///
    /// Rejects a stale contract digest, invalid graph ancestry, or invalid scope.
    pub fn evaluate_contract(
        &self,
        contract: &ContractRule,
        scope: ContractScopeNodeId,
    ) -> Result<Vec<ContractResult>, ContractExecutionError> {
        evaluate_contract(contract, org_graph_spec(), &self.records, scope)
    }
}
