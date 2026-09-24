//! Typed rule vocabulary emitted by the Org Elements Scheme AOT compiler.

/// Field comparison admitted by the Org Elements Scheme module.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrgElementFieldMatch {
    /// The complete property equals the requested value.
    Exact,
    /// The property contains the requested text.
    Contains,
}

/// Relationship to the requested scope Element.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrgElementRelation {
    /// Any Element in the scope, including the scope itself.
    Any,
    /// The scope Element itself.
    At,
    /// A direct child of the scope Element.
    ChildOf,
    /// A strict descendant of the scope Element.
    DescendantOf,
}

/// One property comparison in a Scheme-AOT Element query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrgElementPropertyRule {
    /// Projected or AOT-derived property name.
    pub name: &'static str,
    /// Expected property value.
    pub value: &'static str,
    /// Property comparison.
    pub matcher: OrgElementFieldMatch,
}

/// One named query compiled from `scheme :org-elements`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrgElementQueryRule {
    /// Stable query identity declared by the Org heading.
    pub id: &'static str,
    /// Generated Element kind.
    pub node_kind: &'static str,
    /// Disjunction of conjunctions; an empty conjunction matches every Element.
    pub groups: &'static [&'static [OrgElementPropertyRule]],
    /// Relationship to the scope.
    pub relation: OrgElementRelation,
    /// Whether the relation targets the supplied scope.
    pub target_scope: bool,
}

/// AOT query pack tied to one Org Element projection digest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrgElementQueryPack {
    /// Digest of the graph declaration that generated these rules.
    pub graph_digest: &'static str,
    /// Ordered named queries.
    pub rules: &'static [OrgElementQueryRule],
}

/// Query admission or lookup failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrgElementQueryError {
    /// The query pack belongs to another Element projection.
    StaleGraph,
    /// No query has this ID.
    UnknownQuery,
    /// The scope record is absent.
    InvalidScope,
    /// The query shape is not admitted by this executor.
    InvalidRule,
    /// This derived property lacks an AOT implementation.
    UnsupportedField,
}
