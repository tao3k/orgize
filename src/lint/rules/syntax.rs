//! Syntax admission over parser-owned source records, independent of Contracts
//! that survived extraction. This stage never evaluates a query or host program.

use super::{LintFinding, LintSeverity, model::location_for_range_bounds};
use crate::ast::{ParsedAst, contract_block_syntax_error, contract_source_blocks};

pub(super) fn source_syntax_findings(document: &ParsedAst, source: &str) -> Vec<LintFinding> {
    contract_source_blocks(document)
        .iter()
        .filter_map(|block| {
            let message = contract_block_syntax_error(block)?;
            Some(LintFinding {
                code: "ORG046",
                severity: LintSeverity::Error,
                message: format!(
                    "{message}: {}",
                    block.language.as_deref().unwrap_or_default()
                ),
                location: location_for_range_bounds(
                    source,
                    block.source.range_start as usize,
                    block.source.range_end as usize,
                ),
            })
        })
        .collect()
}
