//! Scheme-authored Org event algorithm compiled to a Rust function at build time.

// The event compiler declares a uniform mutable state tuple for each source
// helper; a restricted helper may leave some state slots unchanged or unused.
#![allow(unused_mut, unused_variables, unused_assignments)]
// The AOT compiler preserves Scheme condition blocks and exclusive branches.
// Rewriting these mechanically for Clippy would change the generated hot path.
#![allow(
    clippy::blocks_in_conditions,
    clippy::collapsible_if,
    clippy::ifs_same_cond,
    clippy::if_same_then_else
)]

use gerbil_parser_rowan::TreeEvent;

include!(concat!(env!("OUT_DIR"), "/org_rowan_events.rs"));
