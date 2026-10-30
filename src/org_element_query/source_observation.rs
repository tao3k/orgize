//! Source-bound observations from Scheme-AOT named Element queries.
//!
//! These values bind a query result to one parser-owned document. A caller
//! still owns file identity, WorkTree cut, access grants, and publication.

use crate::config::ParseConfig;
use crate::org_aot::{
    OrgAotDocument, OrgAotError, org_event_parser_digest, org_graph_spec, parse_org_aot_with_config,
};
use crate::org_aot_edit::org_source_digest;

use super::execute::org_element_query_pack;
use super::model::{OrgElementQueryError, OrgElementQueryPack, OrgElementQueryRule};

/// One graph-local result and its byte span in the parsed source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrgElementQueryMatch {
    pub id: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

/// Result of a maintained Scheme-AOT query on one source-backed parse.
///
/// This is an in-process observation, not a file identity, authorization, or
/// durable Session admission receipt. The numeric IDs are valid only for this
/// parse; persistent Org `ID` properties require separate validation.
#[derive(Debug, Eq, PartialEq)]
pub struct OrgElementQuerySourceObservation {
    source_digest: String,
    source_bytes: usize,
    parser_digest: &'static str,
    graph_digest: &'static str,
    base_config: ParseConfig,
    effective_config: ParseConfig,
    rule: OrgElementQueryRule,
    scope_id: usize,
    matches: Vec<OrgElementQueryMatch>,
}

/// Failure to reconstruct a current source observation for comparison.
#[derive(Debug)]
pub enum OrgElementQueryRecheckError {
    /// The current bytes or generated graph could not be parsed.
    Parse(OrgAotError),
    /// The current query pack, scope, or rule could not be admitted.
    Query(OrgElementQueryError),
}

impl OrgElementQuerySourceObservation {
    #[must_use]
    pub fn source_digest(&self) -> &str {
        &self.source_digest
    }

    #[must_use]
    pub fn source_bytes(&self) -> usize {
        self.source_bytes
    }

    #[must_use]
    pub fn parser_digest(&self) -> &str {
        self.parser_digest
    }

    #[must_use]
    pub fn graph_digest(&self) -> &str {
        self.graph_digest
    }

    #[must_use]
    pub fn base_config(&self) -> &ParseConfig {
        &self.base_config
    }

    #[must_use]
    pub fn effective_config(&self) -> &ParseConfig {
        &self.effective_config
    }

    #[must_use]
    pub fn rule(&self) -> OrgElementQueryRule {
        self.rule
    }

    #[must_use]
    pub fn scope_id(&self) -> usize {
        self.scope_id
    }

    #[must_use]
    pub fn matches(&self) -> &[OrgElementQueryMatch] {
        &self.matches
    }

    /// Recheck the exact UTF-8 source bytes presented by a later consumer.
    #[must_use]
    pub fn matches_source(&self, source: &str) -> bool {
        source.len() == self.source_bytes && org_source_digest(source) == self.source_digest
    }

    /// Reparse the current bytes and rerun the maintained Scheme-AOT named query.
    ///
    /// A caller must authenticate the bytes, parser configuration, file identity,
    /// WorkTree cut, and grant independently. This operation rejects a stale
    /// selection; it does not grant access or publish a Session result.
    ///
    /// # Errors
    ///
    /// Returns a parse or query admission error for the current input.
    pub fn recheck_current_source(
        &self,
        source: &str,
        base_config: &ParseConfig,
    ) -> Result<bool, OrgElementQueryRecheckError> {
        self.recheck_source_with_pack(source, base_config, org_element_query_pack())
    }

    /// Recheck against an explicitly selected Scheme-AOT query pack.
    ///
    /// The caller is responsible for trusting this pack. Keeping the entire
    /// rule in the observation makes a changed rule body with the same ID a
    /// mismatch, even if it happens to select the same records on this file.
    ///
    /// # Errors
    ///
    /// Returns a parse or query admission error for the current input.
    pub fn recheck_source_with_pack(
        &self,
        source: &str,
        base_config: &ParseConfig,
        pack: &OrgElementQueryPack,
    ) -> Result<bool, OrgElementQueryRecheckError> {
        if !self.matches_source(source) || &self.base_config != base_config {
            return Ok(false);
        }
        let current = parse_org_aot_with_config(source, base_config)
            .map_err(OrgElementQueryRecheckError::Parse)?;
        let observed = current
            .query_with_pack_source_observation(pack, self.rule.id, self.scope_id)
            .map_err(OrgElementQueryRecheckError::Query)?;
        Ok(self == &observed)
    }
}

impl OrgAotDocument {
    /// Execute a maintained Scheme-AOT query and bind its IDs to this parse.
    ///
    /// # Errors
    ///
    /// Returns the ordinary query admission error for unknown rules, stale
    /// graph packs, invalid scopes, or invalid generated record IDs.
    pub fn query_named_source_observation(
        &self,
        id: &str,
        scope_id: usize,
    ) -> Result<OrgElementQuerySourceObservation, OrgElementQueryError> {
        self.query_with_pack_source_observation(org_element_query_pack(), id, scope_id)
    }

    /// Execute an explicitly selected Scheme-AOT pack and retain its source binding.
    ///
    /// # Errors
    ///
    /// Returns the ordinary query admission error for a stale or invalid pack,
    /// unknown rule, invalid scope, or invalid generated record IDs.
    pub fn query_with_pack_source_observation(
        &self,
        pack: &OrgElementQueryPack,
        id: &str,
        scope_id: usize,
    ) -> Result<OrgElementQuerySourceObservation, OrgElementQueryError> {
        let ids = self.query_with_pack(pack, id, scope_id)?;
        let rule = pack
            .rules
            .iter()
            .find(|rule| rule.id == id)
            .copied()
            .ok_or(OrgElementQueryError::UnknownQuery)?;
        let matches = ids
            .into_iter()
            .map(|id| {
                let record = self
                    .records()
                    .get(id)
                    .filter(|record| record.id == id)
                    .ok_or(OrgElementQueryError::InvalidRule)?;
                Ok(OrgElementQueryMatch {
                    id,
                    start_byte: usize::from(record.range.start()),
                    end_byte: usize::from(record.range.end()),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let source = self.to_org();
        Ok(OrgElementQuerySourceObservation {
            source_digest: org_source_digest(&source),
            source_bytes: source.len(),
            parser_digest: org_event_parser_digest(),
            graph_digest: org_graph_spec().projection_digest,
            base_config: self.base_config().clone(),
            effective_config: self.config().clone(),
            rule,
            scope_id,
            matches,
        })
    }
}
