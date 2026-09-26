//! Test-only structural oracle for the Scheme event parser cutover.

#[rustfmt::skip]
#[path = "../../languages/org/v1/generated/structure.rs"]
mod generated;

pub(crate) use generated::STRUCTURE;
