//! Query AST and graph predicates for the Org elements index.

use std::collections::BTreeSet;

use super::elements_bridge_model::{
    OrgElementGraph, OrgElementId, OrgElementsIndexCategory, OrgElementsIndexKind,
    OrgElementsIndexRecord, OrgElementsIndexSummaryValue,
};

/// Predicate for selecting records from `Document::org_elements_index()`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OrgElementsIndexQuery {
    pub category: Option<OrgElementsIndexCategory>,
    pub kind: Option<OrgElementsIndexKind>,
    pub affiliated_name: Option<String>,
    pub context: Option<String>,
    pub outline_path_prefix: Vec<String>,
    pub outline_path_exact_len: Option<usize>,
    pub property_equals: Vec<OrgElementsIndexSummaryPredicate>,
    pub property_contains: Vec<OrgElementsIndexSummaryTextPredicate>,
    pub summary_equals: Vec<OrgElementsIndexSummaryPredicate>,
    pub summary_contains: Vec<OrgElementsIndexSummaryTextPredicate>,
    pub relations: Vec<OrgElementsIndexRelation>,
    pub limit: Option<usize>,
}

/// Graph relation predicates shared by host index queries and `CONTRACT_ORG`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrgElementsIndexRelation {
    /// The candidate record's direct parent is one of these ids.
    ChildOf(BTreeSet<OrgElementId>),
    /// The candidate record has one of these ids in its ancestor chain.
    DescendantOf(BTreeSet<OrgElementId>),
    /// The candidate record is an ancestor of one of these ids.
    AncestorOf(BTreeSet<OrgElementId>),
    /// The candidate record id is exactly one of these ids.
    At(BTreeSet<OrgElementId>),
}

/// Exact-match predicate over one compact Org elements index summary field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementsIndexSummaryPredicate {
    pub key: String,
    pub value: OrgElementsIndexSummaryValue,
}

/// Text substring predicate over one compact Org elements index summary field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementsIndexSummaryTextPredicate {
    pub key: String,
    pub needle: String,
}

impl OrgElementsIndexQuery {
    /// Creates an empty index query.
    pub fn new() -> Self {
        Self::default()
    }

    /// Restricts matches to a record category.
    pub fn category(mut self, category: OrgElementsIndexCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Restricts matches to an Org element kind.
    pub fn kind(mut self, kind: impl Into<OrgElementsIndexKind>) -> Self {
        self.kind = Some(kind.into());
        self
    }

    /// Restricts matches to records with an affiliated `NAME`.
    pub fn affiliated_name(mut self, name: impl Into<String>) -> Self {
        self.affiliated_name = Some(name.into());
        self
    }

    /// Restricts matches to records with a projected context label.
    pub fn context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Restricts matches to records below an outline path prefix.
    pub fn outline_path_prefix(
        mut self,
        outline_path_prefix: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.outline_path_prefix = outline_path_prefix.into_iter().map(Into::into).collect();
        self
    }

    /// Restricts matches to records with exactly this outline depth.
    pub fn outline_path_exact_len(mut self, outline_path_exact_len: usize) -> Self {
        self.outline_path_exact_len = Some(outline_path_exact_len);
        self
    }

    /// Adds an exact predicate over element properties.
    pub fn property_eq(
        mut self,
        key: impl Into<String>,
        value: impl Into<OrgElementsIndexSummaryValue>,
    ) -> Self {
        self.property_equals.push(OrgElementsIndexSummaryPredicate {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    /// Adds a substring predicate over element properties.
    pub fn property_contains(mut self, key: impl Into<String>, needle: impl Into<String>) -> Self {
        self.property_contains
            .push(OrgElementsIndexSummaryTextPredicate {
                key: key.into(),
                needle: needle.into(),
            });
        self
    }

    /// Adds an exact predicate over summary fields.
    pub fn summary_eq(
        mut self,
        key: impl Into<String>,
        value: impl Into<OrgElementsIndexSummaryValue>,
    ) -> Self {
        self.summary_equals.push(OrgElementsIndexSummaryPredicate {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    /// Adds a substring predicate over summary fields.
    pub fn summary_contains(mut self, key: impl Into<String>, needle: impl Into<String>) -> Self {
        self.summary_contains
            .push(OrgElementsIndexSummaryTextPredicate {
                key: key.into(),
                needle: needle.into(),
            });
        self
    }

    /// Limits the number of returned records.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Restricts matches to records whose parent is the supplied id.
    pub fn child_of(mut self, parent_id: OrgElementId) -> Self {
        self.relations.push(OrgElementsIndexRelation::ChildOf(
            [parent_id].into_iter().collect(),
        ));
        self
    }

    /// Restricts matches to records whose parent is in the supplied set.
    pub fn child_of_any(mut self, parent_ids: impl IntoIterator<Item = OrgElementId>) -> Self {
        let parent_ids = parent_ids.into_iter().collect::<BTreeSet<_>>();
        if !parent_ids.is_empty() {
            self.relations
                .push(OrgElementsIndexRelation::ChildOf(parent_ids));
        }
        self
    }

    /// Restricts matches to records below the supplied ancestor id.
    pub fn descendant_of(mut self, ancestor_id: OrgElementId) -> Self {
        self.relations.push(OrgElementsIndexRelation::DescendantOf(
            [ancestor_id].into_iter().collect(),
        ));
        self
    }

    /// Restricts matches to records below any supplied ancestor id.
    pub fn descendant_of_any(
        mut self,
        ancestor_ids: impl IntoIterator<Item = OrgElementId>,
    ) -> Self {
        let ancestor_ids = ancestor_ids.into_iter().collect::<BTreeSet<_>>();
        if !ancestor_ids.is_empty() {
            self.relations
                .push(OrgElementsIndexRelation::DescendantOf(ancestor_ids));
        }
        self
    }

    /// Restricts matches to records above the supplied descendant id.
    pub fn ancestor_of(mut self, descendant_id: OrgElementId) -> Self {
        self.relations.push(OrgElementsIndexRelation::AncestorOf(
            [descendant_id].into_iter().collect(),
        ));
        self
    }

    /// Restricts matches to records above any supplied descendant id.
    pub fn ancestor_of_any(
        mut self,
        descendant_ids: impl IntoIterator<Item = OrgElementId>,
    ) -> Self {
        let descendant_ids = descendant_ids.into_iter().collect::<BTreeSet<_>>();
        if !descendant_ids.is_empty() {
            self.relations
                .push(OrgElementsIndexRelation::AncestorOf(descendant_ids));
        }
        self
    }

    /// Restricts matches to exactly this record id.
    pub fn at(mut self, id: OrgElementId) -> Self {
        self.relations
            .push(OrgElementsIndexRelation::At([id].into_iter().collect()));
        self
    }

    /// Restricts matches to exactly one of these record ids.
    pub fn at_any(mut self, ids: impl IntoIterator<Item = OrgElementId>) -> Self {
        let ids = ids.into_iter().collect::<BTreeSet<_>>();
        if !ids.is_empty() {
            self.relations.push(OrgElementsIndexRelation::At(ids));
        }
        self
    }

    /// Returns true when a flat index record satisfies all query predicates.
    pub fn matches<A>(&self, record: &OrgElementsIndexRecord<A>) -> bool {
        self.matches_header(record)
            && self.matches_outline(record)
            && self.matches_properties(record)
            && self.matches_summary(record)
    }

    fn matches_header<A>(&self, record: &OrgElementsIndexRecord<A>) -> bool {
        if let Some(category) = self.category
            && record.category != category
        {
            return false;
        }
        if let Some(kind) = &self.kind
            && record.kind != *kind
        {
            return false;
        }
        if let Some(name) = &self.affiliated_name
            && record.affiliated.name.as_ref() != Some(name)
        {
            return false;
        }
        if let Some(context) = &self.context
            && record.context != *context
        {
            return false;
        }
        true
    }

    fn matches_outline<A>(&self, record: &OrgElementsIndexRecord<A>) -> bool {
        if !self.outline_path_prefix.is_empty()
            && !record.outline_path.starts_with(&self.outline_path_prefix)
        {
            return false;
        }
        if let Some(outline_path_exact_len) = self.outline_path_exact_len
            && record.outline_path.len() != outline_path_exact_len
        {
            return false;
        }
        true
    }

    fn matches_properties<A>(&self, record: &OrgElementsIndexRecord<A>) -> bool {
        self.property_equals.iter().all(|predicate| {
            record_property(record, &predicate.key).is_some_and(|value| value == &predicate.value)
        }) && self.property_contains.iter().all(|predicate| {
            record_property(record, &predicate.key)
                .is_some_and(|value| value.contains_text(&predicate.needle))
        })
    }

    fn matches_summary<A>(&self, record: &OrgElementsIndexRecord<A>) -> bool {
        self.summary_equals.iter().all(|predicate| {
            record
                .summary
                .get(&predicate.key)
                .is_some_and(|value| value == &predicate.value)
        }) && self.summary_contains.iter().all(|predicate| {
            record
                .summary
                .get(&predicate.key)
                .is_some_and(|value| value.contains_text(&predicate.needle))
        })
    }
}

impl<A> OrgElementGraph<A> {
    /// Returns records that satisfy the query predicates and graph relations.
    pub fn query(&self, query: &OrgElementsIndexQuery) -> Vec<&OrgElementsIndexRecord<A>> {
        let mut records = self
            .records
            .iter()
            .filter(|record| query.matches(*record))
            .filter(|record| {
                query
                    .relations
                    .iter()
                    .all(|relation| relation.matches(self, record.id))
            })
            .collect::<Vec<_>>();
        if let Some(limit) = query.limit {
            records.truncate(limit);
        }
        records
    }
}

impl OrgElementsIndexRelation {
    fn matches<A>(&self, graph: &OrgElementGraph<A>, id: OrgElementId) -> bool {
        match self {
            Self::ChildOf(parent_ids) => graph
                .parent(id)
                .is_some_and(|parent| parent_ids.contains(&parent.id)),
            Self::DescendantOf(ancestor_ids) => graph
                .ancestors(id)
                .iter()
                .any(|ancestor| ancestor_ids.contains(&ancestor.id)),
            Self::AncestorOf(descendant_ids) => descendant_ids.iter().any(|descendant_id| {
                graph
                    .ancestors(*descendant_id)
                    .iter()
                    .any(|ancestor| ancestor.id == id)
            }),
            Self::At(ids) => ids.contains(&id),
        }
    }
}

impl OrgElementsIndexSummaryValue {
    fn contains_text(&self, needle: &str) -> bool {
        match self {
            Self::Text(value) => value.contains(needle),
            Self::StringList(values) => values.iter().any(|value| value.contains(needle)),
            Self::Null | Self::Bool(_) | Self::Integer(_) => false,
        }
    }
}

fn record_property<'a, A>(
    record: &'a OrgElementsIndexRecord<A>,
    key: &str,
) -> Option<&'a OrgElementsIndexSummaryValue> {
    record
        .properties
        .get(key)
        .or_else(|| {
            key.strip_prefix(':')
                .and_then(|key| record.properties.get(key))
        })
        .or_else(|| {
            (!key.starts_with(':'))
                .then(|| record.properties.get(&format!(":{key}")))
                .flatten()
        })
}
