//! Cargo-only execution of Scheme-AOT Org Element query packs.

mod execute;
mod model;
mod query_plan;
mod source_observation;

pub use execute::org_element_query_pack;
pub(crate) use execute::property_matches as element_property_matches;
pub use model::{
    OrgElementFieldMatch, OrgElementPropertyRule, OrgElementQueryError, OrgElementQueryPack,
    OrgElementQueryRule, OrgElementRelation,
};
pub use source_observation::{
    OrgElementQueryMatch, OrgElementQueryRecheckError, OrgElementQuerySourceObservation,
};
