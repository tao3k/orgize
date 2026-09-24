//! Cargo-only Org parsing from the Scheme-owned AOT language artifacts.

use gerbil_parser_rowan::{
    Diagnostic, GraphProjectionSpec, GraphRecord, LanguageSpec, LineStructureSpec, Parse,
    ParseError, ParseReceipt, SyntaxNode, parse_structural_lines, project_syntax_graph,
};

use crate::contract_feature::{
    ContractExecutionError, ContractPack, ContractResult, ContractRule, ContractScopeNodeId,
    evaluate_contract,
};

#[rustfmt::skip]
#[path = "../languages/org/v1/generated/parser.rs"]
mod grammar;
#[rustfmt::skip]
#[path = "../languages/org/v1/generated/structure.rs"]
mod structure;
#[rustfmt::skip]
#[path = "../languages/org/v1/generated/graph.rs"]
mod graph;
#[path = "org_aot_contract_plan.rs"]
mod contract_plan;
#[rustfmt::skip]
#[path = "../languages/org/v1/modules/org-elements/generated/todo_directive.rs"]
mod todo_directive;
#[rustfmt::skip]
#[path = "../languages/org/v1/modules/org-elements/generated/todo_state_from_directives.rs"]
mod todo_state_from_directives;

/// A source-backed, lossless Rowan tree and its Scheme-declared Element graph.
#[derive(Debug)]
pub struct OrgAotDocument {
    parse: Parse,
    records: Vec<GraphRecord>,
    todo_directives: Vec<String>,
}

/// An error from the parser or the generated Element projection contract.
#[derive(Debug)]
pub enum OrgAotError {
    /// The generated parser rejected the source.
    Parse(ParseError),
    /// The generated Element projection did not match the parser artifact.
    Projection(Diagnostic),
}

/// Parse Org source through the Scheme-owned AOT parser and Element projection.
///
/// No Gerbil runtime or package is needed by a Cargo consumer.
///
/// # Errors
///
/// Returns the parser receipt on parse failure, or a projection diagnostic if
/// the generated graph table is stale or invalid.
pub fn parse_org_aot(source: &str) -> Result<OrgAotDocument, OrgAotError> {
    let parse = parse_structural_lines(&grammar::LANGUAGE, &structure::STRUCTURE, source)
        .map_err(OrgAotError::Parse)?;
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
        .collect();
    Ok(OrgAotDocument {
        parse,
        records,
        todo_directives,
    })
}

/// The generated Org grammar, including syntax-kind names and its digest.
#[must_use]
pub fn org_language_spec() -> &'static LanguageSpec {
    &grammar::LANGUAGE
}

/// The generated Org contextual structure contract and its parser digest.
#[must_use]
pub fn org_structure_spec() -> &'static LineStructureSpec {
    &structure::STRUCTURE
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
        let headline = self
            .records
            .get(record_id)
            .filter(|record| record.id == record_id && record.kind == "headline")?;
        let title = headline.field("title")?;
        match todo_state_from_directives::todo_state_from_directives(title, &self.todo_directives) {
            "" => None,
            state => Some(state),
        }
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
