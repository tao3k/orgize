//! Execute a typed Scheme-AOT query pack against a parsed Org Element graph.

use gerbil_parser_runtime::{GraphIndexError, GraphRecord, GraphRelation};

use crate::org_aot::{OrgAotDocument, org_graph_spec};

use super::model::{
    OrgElementFieldMatch, OrgElementQueryError, OrgElementQueryPack, OrgElementQueryRule,
    OrgElementRelation,
};
use super::query_plan;

/// The maintained Scheme-AOT `:org-elements-query` query pack.
#[must_use]
pub fn org_element_query_pack() -> &'static OrgElementQueryPack {
    &query_plan::QUERIES
}

fn admit_rule(rule: &OrgElementQueryRule) -> Result<(), OrgElementQueryError> {
    if rule.id.is_empty()
        || rule.target_scope != (rule.relation != OrgElementRelation::Any)
        || rule.groups.is_empty()
        || rule.groups.len() > 32
        || rule.groups.iter().any(|group| group.len() > 16)
    {
        return Err(OrgElementQueryError::InvalidRule);
    }
    let node = org_graph_spec()
        .rules
        .iter()
        .find(|node| node.kind == rule.node_kind)
        .ok_or(OrgElementQueryError::InvalidRule)?;
    for group in rule.groups {
        for property in *group {
            match property.name {
                "title" | "raw-value" | "priority" | "tags" | "todo-keyword" | "todo-type"
                | "source-title"
                    if matches!(node.kind, "headline" | "inlinetask") => {}
                name if node.fields.iter().any(|field| field.name == name) => {}
                _ => return Err(OrgElementQueryError::InvalidRule),
            }
        }
    }
    Ok(())
}

pub(crate) fn property_matches(
    document: &OrgAotDocument,
    record: &GraphRecord,
    name: &str,
    value: &str,
    matcher: OrgElementFieldMatch,
) -> bool {
    let matches = |actual: &str| match matcher {
        OrgElementFieldMatch::Exact => actual == value,
        OrgElementFieldMatch::Contains => actual.contains(value),
    };
    match name {
        "title" | "priority" | "todo-keyword" => document
            .headline_derived_field(record.id, name)
            .is_some_and(matches),
        "todo-type" => document.headline_todo_type(record.id).is_some_and(matches),
        "raw-value" | "source-title" if matches!(record.kind, "headline" | "inlinetask") => {
            record.field("title").is_some_and(matches)
        }
        "tags" if matches!(record.kind, "headline" | "inlinetask") => {
            record.values("tag").any(matches)
        }
        name => record.values(name).any(matches),
    }
}

impl OrgAotDocument {
    /// Execute a maintained named query within an Element scope.
    ///
    /// # Errors
    ///
    /// Rejects an unknown query, stale graph pack, or invalid scope.
    pub fn query_named(
        &self,
        id: &str,
        scope_id: usize,
    ) -> Result<Vec<usize>, OrgElementQueryError> {
        self.query_with_pack(org_element_query_pack(), id, scope_id)
    }

    /// Execute a consumer-supplied Scheme-AOT pack without a Gerbil runtime.
    ///
    /// # Errors
    ///
    /// Rejects unknown IDs, stale graph revisions and invalid properties.
    pub fn query_with_pack(
        &self,
        pack: &OrgElementQueryPack,
        id: &str,
        scope_id: usize,
    ) -> Result<Vec<usize>, OrgElementQueryError> {
        if pack.graph_digest != org_graph_spec().projection_digest {
            return Err(OrgElementQueryError::StaleGraph);
        }
        let mut named_rules = pack.rules.iter().filter(|rule| rule.id == id);
        let rule = named_rules
            .next()
            .ok_or(OrgElementQueryError::UnknownQuery)?;
        if named_rules.next().is_some() {
            return Err(OrgElementQueryError::InvalidRule);
        }
        if self.graph_index().subtree_end(scope_id).is_none() {
            return Err(OrgElementQueryError::InvalidScope);
        }
        admit_rule(rule)?;
        let relation = match rule.relation {
            OrgElementRelation::Any => GraphRelation::Any,
            OrgElementRelation::At => GraphRelation::At,
            OrgElementRelation::ChildOf => GraphRelation::ChildOf,
            OrgElementRelation::DescendantOf => GraphRelation::DescendantOf,
        };
        self.graph_index()
            .select(
                self.records(),
                scope_id,
                rule.node_kind,
                relation,
                &[scope_id],
                |record| {
                    rule.groups.iter().any(|group| {
                        group.iter().all(|property| {
                            property_matches(
                                self,
                                record,
                                property.name,
                                property.value,
                                property.matcher,
                            )
                        })
                    })
                },
            )
            .map_err(|error| match error {
                GraphIndexError::InvalidScope => OrgElementQueryError::InvalidScope,
                GraphIndexError::InvalidRecord | GraphIndexError::InvalidTarget => {
                    OrgElementQueryError::InvalidRule
                }
            })
    }
}
