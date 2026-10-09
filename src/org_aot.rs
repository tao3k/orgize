//! Native Gerbil Org parsing with source-backed navigation and Element projections.
//!
//! The default AOT entrypoint runs the Org-owned Scheme event algorithm.
//! The public `Org` facade uses this parser; the owned semantic AST is a
//! projection of its Scheme-generated Element graph.

use std::{collections::HashMap, sync::OnceLock};

use gerbil_parser_runtime::TextRange;
use gerbil_parser_runtime::{
    Diagnostic, GraphIndex, GraphProjectionSpec, GraphRecord, LanguageSpec, Parse, ParseError,
    ParseReceipt, SyntaxNode, parse_generated_events, project_syntax_graph,
};

use crate::config::ParseConfig;
use crate::contract_feature::{
    ContractExecutionError, ContractPack, ContractResult, ContractRule, ContractScopeNodeId,
    evaluate_contract,
};

#[rustfmt::skip]
#[path = "../languages/org/generated/parser.rs"]
mod grammar;
#[rustfmt::skip]
#[path = "../languages/org/generated/graph.rs"]
mod graph;
#[path = "org_aot_contract_plan.rs"]
mod contract_plan;
#[path = "org_aot_headline_view.rs"]
mod headline_view;
pub use headline_view::OrgHeadline;
#[path = "org_aot_affiliation.rs"]
mod affiliation;
#[path = "org_native_identity.rs"]
mod event_identity;
#[path = "org_aot_keyword_view.rs"]
mod keyword_view;
#[path = "org_aot_metadata_batch.rs"]
mod metadata_batch;
#[path = "org_native_semantic_functions.rs"]
mod native_functions;

#[path = "org_native_events/mod.rs"]
mod native_events;
#[path = "org_native_expression.rs"]
mod native_expression;
pub(crate) use native_expression::NativeExpressionValue;

pub(crate) fn parse_native_expression_values(
    source: &str,
    grammar: &LanguageSpec,
) -> Result<Vec<NativeExpressionValue>, String> {
    native_events::parse_expression_values(source, grammar)
}

pub(crate) fn initialize_native_owner() -> Result<(), String> {
    native_events::initialize_owner()
}

pub(crate) fn expand_native_macro_fields(operation: u8, fields: &[&str]) -> Result<String, String> {
    native_events::expand_macro_fields(operation, fields)
}

pub(crate) fn native_semantic_rows(
    operation: u8,
    fields: &[&str],
) -> Result<Vec<Vec<String>>, String> {
    use self::native_expression::NativeExpressionValue;
    native_events::semantic_fields(operation, fields)?
        .into_iter()
        .map(|value| {
            let NativeExpressionValue::List(row) = value else {
                return Err("native semantic row expected".into());
            };
            row.into_iter()
                .map(|value| match value {
                    NativeExpressionValue::String(value) => Ok(value),
                    _ => Err("native semantic string expected".into()),
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
pub(crate) fn parse_native_expression(
    source: &str,
    grammar: &'static LanguageSpec,
) -> Result<Parse, OrgAotError> {
    let events =
        native_events::parse_expression_events(source, grammar).map_err(OrgAotError::Native)?;
    parse_generated_events(
        grammar,
        event_identity::NATIVE_PARSER_DIGEST,
        source,
        &events,
    )
    .map_err(OrgAotError::Parse)
}

pub(crate) fn evaluate_native_contract(
    input: crate::c_ffi::ContractInput,
) -> Result<crate::c_ffi::ContractOutput, String> {
    native_events::evaluate_contract(input)
}

/// A source-backed navigation index and its Scheme-declared Element graph.
/// Native parsing requires explicit [`crate::initialize_native_runtime`] at
/// exclusive host startup; parse calls never initialize the runtime implicitly.
#[derive(Debug)]
pub struct OrgAotDocument {
    parse: Parse,
    records: Vec<GraphRecord>,
    base_config: ParseConfig,
    config: ParseConfig,
    headline_properties: Vec<Option<HeadlineProperties>>,
    keyword_plans: keyword_view::KeywordPlans,
    graph_index: GraphIndex,
    headline_ancestors: Vec<Option<usize>>,
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
    /// The linked Gerbil program could not initialize, call, or project events.
    Native(String),
    /// The generated parser rejected the source.
    Parse(ParseError),
    /// The generated Element projection did not match the parser artifact.
    Projection(Diagnostic),
}

/// Parse Org source through the statically linked Gerbil event strategy and Element projection.
///
/// # Errors
///
/// Returns the parser receipt on parse failure, or a projection diagnostic if
/// the generated graph table is stale or invalid.
pub fn parse_org_aot(source: &str) -> Result<OrgAotDocument, OrgAotError> {
    static DEFAULT_CONFIG: OnceLock<ParseConfig> = OnceLock::new();
    parse_org_aot_with_config(source, DEFAULT_CONFIG.get_or_init(ParseConfig::default))
}

/// Parse Org with the native Gerbil strategy and a caller-provided configuration.
///
/// # Errors
///
/// Returns the parser receipt on parse failure, or a projection diagnostic if
/// the generated graph table is stale or invalid.
pub fn parse_org_aot_with_config(
    source: &str,
    config: &ParseConfig,
) -> Result<OrgAotDocument, OrgAotError> {
    let events = native_events::parse_events(
        source,
        config.effective_inlinetask_min_level(),
        config.inline_script_policy(),
        &grammar::LANGUAGE,
    )
    .map_err(OrgAotError::Native)?;
    parse_org_aot_events(source, events, config)
}

pub(crate) const BATCH_MAX_DOCUMENTS: usize = native_events::BATCH_MAX_DOCUMENTS;
pub(crate) const BATCH_MAX_SOURCE_BYTES: usize = native_events::BATCH_MAX_SOURCE_BYTES;

/// Private bounded handoff; every document keeps its own receipt and offsets.
pub(crate) fn parse_org_aot_batch(
    sources: &[&str],
    config: &ParseConfig,
) -> Result<Vec<Result<OrgAotDocument, OrgAotError>>, OrgAotError> {
    let outcomes = native_events::parse_events_batch(
        sources,
        config.effective_inlinetask_min_level(),
        config.inline_script_policy(),
        &grammar::LANGUAGE,
    )
    .map_err(OrgAotError::Native)?;
    let prepared = sources
        .iter()
        .zip(outcomes)
        .map(|(source, events)| {
            events
                .map_err(OrgAotError::Native)
                .and_then(|events| parse_event_artifact(source, events))
                .and_then(|parse| prepare_document(parse, config))
        })
        .collect::<Vec<_>>();
    metadata_batch::finish(prepared, config)
}

#[cfg(test)]
#[path = "../tests/unit/org_native_batch.rs"]
pub(crate) mod batch_tests;

fn parse_org_aot_events(
    source: &str,
    events: Vec<gerbil_parser_runtime::TreeEvent>,
    config: &ParseConfig,
) -> Result<OrgAotDocument, OrgAotError> {
    document_from_parse(parse_event_artifact(source, events)?, config)
}

fn parse_event_artifact(
    source: &str,
    events: Vec<gerbil_parser_runtime::TreeEvent>,
) -> Result<Parse, OrgAotError> {
    crate::runtime_profile::stage("artifact.syntax_index", || {
        parse_generated_events(
            &grammar::LANGUAGE,
            event_identity::NATIVE_PARSER_DIGEST,
            source,
            &events,
        )
    })
    .map_err(OrgAotError::Parse)
}

pub(crate) fn org_image_link(target: &str) -> bool {
    native_functions::org_image_link_p(target)
}

pub(crate) fn org_link_kind(path: &str) -> &'static str {
    native_functions::org_link_kind(path)
}

pub(crate) fn org_link_protocol(path: &str) -> String {
    native_functions::org_link_protocol(path)
}

pub(crate) fn org_link_protocol_path(path: &str) -> String {
    native_functions::org_link_protocol_path(path)
}

pub(crate) fn org_expand_link_abbreviation(
    replacement: &str,
    path: &str,
    encoded_path: &str,
) -> String {
    native_functions::org_expand_link_abbreviation(replacement, path, encoded_path)
}

struct PreparedDocument {
    parse: Parse,
    records: Vec<GraphRecord>,
    graph_index: GraphIndex,
    headline_ancestors: Vec<Option<usize>>,
    keyword_ids: Vec<usize>,
    title_ids: Vec<usize>,
    fields: Vec<String>,
}

fn prepare_document(parse: Parse, config: &ParseConfig) -> Result<PreparedDocument, OrgAotError> {
    let records = crate::runtime_profile::stage("graph.project", || {
        project_syntax_graph(&grammar::LANGUAGE, &graph::GRAPH, &parse.syntax())
    })
    .map_err(OrgAotError::Projection)?;
    let graph_index = crate::runtime_profile::stage("graph.index", || GraphIndex::new(&records))
        .map_err(|error| {
            OrgAotError::Projection(Diagnostic {
                reason_kind: "invalid-graph-index",
                byte_offset: 0,
                message: format!("generated Element graph has invalid preorder: {error:?}"),
            })
        })?;
    let headline_ancestors = crate::runtime_profile::stage("graph.ancestors", || {
        graph_index
            .nearest_ancestors_matching(&records, |record| {
                matches!(record.kind, "headline" | "inlinetask")
            })
            .expect("validated graph index and records have the same preorder")
    });
    let keyword_records = records
        .iter()
        .filter(|r| r.kind == "keyword")
        .collect::<Vec<_>>();
    let keyword_fields = keyword_records
        .iter()
        .flat_map(|r| {
            [
                r.field("key").unwrap_or_default(),
                r.field("raw-value").expect("source-backed keyword value"),
            ]
        })
        .collect::<Vec<_>>();
    let titles = records
        .iter()
        .filter(|r| matches!(r.kind, "headline" | "inlinetask") && r.field("title").is_some())
        .collect::<Vec<_>>();
    let mut fields = Vec::<String>::new();
    for list in [&config.todo_keywords.0, &config.todo_keywords.1] {
        fields.push(list.len().to_string());
        fields.extend_from_slice(list);
    }
    fields.push(keyword_fields.len().to_string());
    fields.extend(keyword_fields.into_iter().map(str::to_owned));
    fields.push((titles.len() * 3).to_string());
    for r in &titles {
        fields.extend([
            r.field("title").unwrap().to_owned(),
            r.field("title-body")
                .expect("source headline title")
                .to_owned(),
            r.field("tag").is_some().to_string(),
        ]);
    }
    let keyword_ids = keyword_records.iter().map(|record| record.id).collect();
    let title_ids = titles.iter().map(|record| record.id).collect();
    Ok(PreparedDocument {
        parse,
        records,
        graph_index,
        headline_ancestors,
        keyword_ids,
        title_ids,
        fields,
    })
}

fn document_from_parse(parse: Parse, config: &ParseConfig) -> Result<OrgAotDocument, OrgAotError> {
    let prepared = prepare_document(parse, config)?;
    let refs = prepared
        .fields
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let rows = native_semantic_rows(17, &refs).map_err(OrgAotError::Native)?;
    finish_document(prepared, rows, config)
}

fn finish_document(
    prepared: PreparedDocument,
    mut rows: Vec<Vec<String>>,
    config: &ParseConfig,
) -> Result<OrgAotDocument, OrgAotError> {
    let PreparedDocument {
        parse,
        records,
        graph_index,
        headline_ancestors,
        keyword_ids,
        title_ids,
        fields: _,
    } = prepared;
    if rows.len() < title_ids.len() + 1 {
        return Err(OrgAotError::Native("truncated native document rows".into()));
    }
    let keyword_rows = rows.split_off(title_ids.len() + 1);
    let keyword_plans = keyword_view::KeywordPlans::admit(&keyword_ids, keyword_rows)
        .map_err(OrgAotError::Native)?;
    let config_row = &rows[0];
    assert!(config_row.len() >= 2, "native TODO config row");
    let open: usize = config_row[0].parse().expect("native open count");
    let done: usize = config_row[1].parse().expect("native done count");
    assert_eq!(
        config_row.len(),
        2 + open + done,
        "native TODO config count"
    );
    let mut effective_config = config.clone();
    effective_config.todo_keywords = (
        config_row[2..2 + open].to_vec(),
        config_row[2 + open..].to_vec(),
    );
    let mut headline_properties = std::iter::repeat_with(|| None)
        .take(records.len())
        .collect::<Vec<_>>();
    for (id, row) in title_ids.into_iter().zip(rows.into_iter().skip(1)) {
        let [
            state,
            keyword,
            content_after_todo,
            priority,
            display_title,
            source_title,
            comment,
        ]: [String; 7] = row.try_into().expect("native headline row arity");
        let todo_type = match state.as_str() {
            "" => None,
            "todo" => Some("todo"),
            "done" => Some("done"),
            _ => panic!("native TODO state"),
        };
        let is_comment = match comment.as_str() {
            "true" => true,
            "false" => false,
            _ => panic!("native comment flag"),
        };
        headline_properties[id] = Some(HeadlineProperties {
            todo_type,
            details: OnceLock::from(HeadlineDetails {
                todo_keyword: (!keyword.is_empty()).then_some(keyword),
                content_after_todo,
                priority_cookie: (!priority.is_empty()).then_some(priority),
                display_title,
                source_title,
                is_comment,
            }),
        });
    }
    Ok(OrgAotDocument {
        parse,
        records,
        base_config: config.clone(),
        config: effective_config,
        headline_properties,
        keyword_plans,
        graph_index,
        headline_ancestors,
        affiliations: OnceLock::new(),
    })
}

/// The generated Org grammar, including syntax-kind names and its digest.
#[must_use]
pub fn org_language_spec() -> &'static LanguageSpec {
    &grammar::LANGUAGE
}

/// Digest of the admitted native program's generated Scheme module contents.
#[must_use]
pub fn org_event_parser_digest() -> &'static str {
    event_identity::NATIVE_PARSER_DIGEST
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
    /// Parse Org through native Gerbil AOT/FFI with default configuration.
    #[must_use]
    pub fn parse(source: impl AsRef<str>) -> Self {
        Self::try_parse(source).expect("native Gerbil Org parser failed")
    }

    /// Parse Org with structured AOT diagnostics.
    ///
    /// # Errors
    /// Returns a native-runtime, parser, or graph-projection error.
    pub fn try_parse(source: impl AsRef<str>) -> Result<Self, OrgAotError> {
        parse_org_aot(source.as_ref())
    }

    /// Return the original source retained by the navigation index.
    #[must_use]
    pub fn to_org(&self) -> String {
        self.syntax().to_string()
    }

    /// Return the effective configuration after file-local Org directives.
    #[must_use]
    pub fn config(&self) -> &ParseConfig {
        &self.config
    }

    /// Return the caller configuration before file-local directives.
    #[must_use]
    pub fn base_config(&self) -> &ParseConfig {
        &self.base_config
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
        self.nearest_headline_ancestor(id).is_some()
    }

    pub(crate) fn nearest_headline_ancestor(&self, id: usize) -> Option<usize> {
        self.headline_ancestors.get(id).copied().flatten()
    }

    /// Replace a UTF-8-aligned byte range and reparse through the Scheme AOT engine.
    pub fn replace_range(&mut self, range: TextRange, replacement: impl AsRef<str>) {
        let mut source = self.to_org();
        source.replace_range(
            usize::from(range.start())..usize::from(range.end()),
            replacement.as_ref(),
        );
        *self = parse_org_aot_with_config(&source, &self.base_config)
            .expect("native Gerbil Org parser failed on edited source");
    }

    fn headline_details(&self, record_id: usize) -> Option<&HeadlineDetails> {
        let properties = self.headline_properties.get(record_id)?.as_ref()?;
        properties.details.get()
    }

    pub(crate) fn headline_derived_field(&self, record_id: usize, name: &str) -> Option<&str> {
        let details = self.headline_details(record_id)?;
        match name {
            "title" => Some(&details.display_title),
            "priority" => details.priority_cookie.as_deref(),
            "todo-keyword" => details.todo_keyword.as_deref(),
            _ => None,
        }
    }

    pub(crate) fn graph_index(&self) -> &GraphIndex {
        &self.graph_index
    }

    /// Return the source-backed navigation root; no token text is duplicated.
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

    pub(crate) fn keyword_fact_rows(&self, record_id: usize) -> &[Vec<String>] {
        self.keyword_plans
            .get(record_id)
            .expect("source-bound native keyword facts")
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
        native_functions::memory_headline_state(
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
        native_functions::planning_key_kind(key)
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
        evaluate_contract(contract, org_graph_spec(), self, scope)
    }
}
