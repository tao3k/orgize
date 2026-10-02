//! The public Org facade is the Scheme-generated AOT Rowan document.

pub use crate::org_aot::OrgAotDocument as Org;

use crate::{config::ParseConfig, org_aot::parse_org_aot_with_config};

impl ParseConfig {
    /// Parse Org with this configuration through the Scheme-generated algorithm.
    #[must_use]
    pub fn parse(self, source: impl AsRef<str>) -> Org {
        parse_org_aot_with_config(source.as_ref(), &self)
            .expect("Scheme-generated Org parser rejected input")
    }
}
