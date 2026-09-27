//! Cargo-only Org parsing from Scheme-declared AOT language artifacts.
//!
//! The default AOT entrypoint runs the Org-owned Scheme event algorithm.
//! The public `Org` facade still requires its separate typed-AST cutover.

use std::sync::OnceLock;

use gerbil_parser_rowan::{
    Diagnostic, GraphProjectionSpec, GraphRecord, LanguageSpec, Parse, ParseError, ParseReceipt,
    SyntaxNode, parse_generated_events, project_syntax_graph,
};

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
#[path = "org_aot_contract_plan.rs"]
mod contract_plan;
#[path = "org_aot_headline_functions.rs"]
mod headline_functions;
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
    todo_directives: Vec<String>,
    headline_properties: Vec<Option<HeadlineProperties>>,
    subtree_end: Vec<usize>,
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
    display_title: String,
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
    parse_org_aot_events(
        source,
        generated_context_events::parse_org_rowan_events(source),
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
    parse_org_aot_events(source, events)
}

fn parse_org_aot_events(
    source: &str,
    events: Vec<gerbil_parser_rowan::TreeEvent>,
) -> Result<OrgAotDocument, OrgAotError> {
    let parse = parse_generated_events(
        &grammar::LANGUAGE,
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .map_err(OrgAotError::Parse)?;
    document_from_parse(parse)
}

pub(crate) fn org_image_link(target: &str) -> bool {
    link_functions::org_image_link_p(target)
}

fn document_from_parse(parse: Parse) -> Result<OrgAotDocument, OrgAotError> {
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
    let headline_properties = records
        .iter()
        .map(|record| {
            let title = record
                .field("title")
                .filter(|_| matches!(record.kind, "headline" | "inlinetask"))?;
            let todo_type =
                match headline_functions::todo_state_from_directives(title, &todo_directives) {
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
        todo_directives,
        headline_properties,
        subtree_end,
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
    fn headline_details(&self, record_id: usize) -> Option<&HeadlineDetails> {
        let properties = self.headline_properties.get(record_id)?.as_ref()?;
        Some(properties.details.get_or_init(|| {
            let record = &self.records[record_id];
            let title = record.field("title").expect("projected headline title");
            let todo_keyword =
                headline_functions::todo_keyword_from_directives(title, &self.todo_directives);
            let content_after_todo =
                headline_functions::headline_content_after_todo(title, &self.todo_directives);
            let display_title = headline_functions::headline_display_title(
                &content_after_todo,
                record.field("tag").is_some(),
            );
            HeadlineDetails {
                todo_keyword: (!todo_keyword.is_empty()).then_some(todo_keyword),
                content_after_todo,
                display_title,
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

    /// Query a headline's TODO type from this document's keyword Elements.
    /// File-local TODO, SEQ_TODO and TYP_TODO declarations override defaults.
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

    /// Return the file-local TODO keyword recognized by the Scheme AOT algorithm.
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
