//! Execute a typed Scheme-AOT query pack against a parsed Org Element graph.

use gerbil_parser_rowan::GraphRecord;

use crate::org_aot::{OrgAotDocument, org_graph_spec};

use super::model::{
    OrgElementFieldMatch, OrgElementPropertyRule, OrgElementQueryError, OrgElementQueryPack,
    OrgElementQueryRule, OrgElementRelation,
};
use super::query_plan;

/// The maintained Scheme-AOT `:org-elements` query pack.
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
                "title" | "raw-value" | "todo-keyword" | "priority" | "tags" => {
                    return Err(OrgElementQueryError::UnsupportedField);
                }
                "todo-type" | "source-title" if node.kind == "headline" => {}
                name if node.fields.iter().any(|field| field.name == name) => {}
                _ => return Err(OrgElementQueryError::InvalidRule),
            }
        }
    }
    Ok(())
}

fn property_matches(
    document: &OrgAotDocument,
    record: &GraphRecord,
    property: &OrgElementPropertyRule,
) -> bool {
    let matches = |actual: &str| match property.matcher {
        OrgElementFieldMatch::Exact => actual == property.value,
        OrgElementFieldMatch::Contains => actual.contains(property.value),
    };
    match property.name {
        "todo-type" => document.headline_todo_type(record.id).is_some_and(matches),
        "source-title" if record.kind == "headline" => record.field("title").is_some_and(matches),
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
    /// Rejects unknown IDs, stale graph revisions and unsupported properties.
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
        let end = self
            .element_subtree_end(scope_id)
            .ok_or(OrgElementQueryError::InvalidScope)?;
        admit_rule(rule)?;
        let mut matches = Vec::new();
        for record in &self.records()[scope_id..end] {
            if record.kind != rule.node_kind {
                continue;
            }
            let related = match rule.relation {
                OrgElementRelation::Any => true,
                OrgElementRelation::At => record.id == scope_id,
                OrgElementRelation::ChildOf => record.parent_id == Some(scope_id),
                OrgElementRelation::DescendantOf => record.id > scope_id,
            };
            if related
                && rule.groups.iter().any(|group| {
                    group
                        .iter()
                        .all(|property| property_matches(self, record, property))
                })
            {
                matches.push(record.id);
            }
        }
        Ok(matches)
    }
}
