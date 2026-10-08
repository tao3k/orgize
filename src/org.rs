//! Public Org documents parsed by native Gerbil AOT/FFI, with source-backed navigation.

pub use crate::org_aot::OrgAotDocument as Org;

use crate::{config::ParseConfig, org_aot::parse_org_aot_with_config};

impl ParseConfig {
    /// Parse Org with this configuration through native Gerbil AOT/FFI.
    #[must_use]
    pub fn parse(self, source: impl AsRef<str>) -> Org {
        parse_org_aot_with_config(source.as_ref(), &self).expect("native Gerbil Org parser failed")
    }
}
