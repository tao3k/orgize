//! Cargo-only Org parsing from Scheme-declared AOT language artifacts.
//!
//! The default AOT entrypoint runs the Org-owned Scheme event algorithm.
//! The public `Org` facade still requires its separate typed-AST cutover.

use gerbil_parser_rowan::{
    Diagnostic, GraphProjectionSpec, GraphRecord, LanguageSpec, Parse, ParseError, ParseReceipt,
    SyntaxNode, parse_generated_events, project_syntax_graph,
};

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
    todo_states: Vec<Option<&'static str>>,
    subtree_end: Vec<usize>,
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
    let events = generated_context_events::parse_org_rowan_events(source);
    let parse = parse_generated_events(
        &grammar::LANGUAGE,
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .map_err(OrgAotError::Parse)?;
    document_from_parse(parse)
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
    let todo_states = records
        .iter()
        .map(|record| {
            let title = record
                .field("title")
                .filter(|_| record.kind == "headline")?;
            match headline_functions::todo_state_from_directives(title, &todo_directives) {
                "" => None,
                state => Some(state),
            }
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
        todo_states,
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
    pub(crate) fn headline_todo_keyword_matches(&self, record_id: usize, expected: &str) -> bool {
        self.records
            .get(record_id)
            .filter(|record| record.kind == "headline")
            .and_then(|record| record.field("title"))
            .is_some_and(|title| {
                headline_functions::todo_keyword_matches_p(title, &self.todo_directives, expected)
            })
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
        self.todo_states.get(record_id).copied().flatten()
    }

    /// Return the file-local TODO keyword recognized by the Scheme AOT algorithm.
    #[must_use]
    pub fn headline_todo_keyword(&self, record_id: usize) -> Option<String> {
        let title = self
            .records
            .get(record_id)
            .filter(|record| record.kind == "headline")?
            .field("title")?;
        let keyword =
            headline_functions::todo_keyword_from_directives(title, &self.todo_directives);
        (!keyword.is_empty()).then_some(keyword)
    }

    /// Return headline content after a Scheme-recognized TODO keyword.
    /// Priority and tags are retained until their Element projections run.
    #[must_use]
    pub fn headline_content_after_todo(&self, record_id: usize) -> Option<String> {
        let title = self
            .records
            .get(record_id)
            .filter(|record| record.kind == "headline")?
            .field("title")?;
        Some(headline_functions::headline_content_after_todo(
            title,
            &self.todo_directives,
        ))
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
