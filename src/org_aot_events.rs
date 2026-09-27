//! Scheme-authored Org event algorithm compiled to a Rust function at build time.

// The event compiler declares a uniform mutable state tuple for each source
// helper; a restricted helper may leave some state slots unchanged.
#![allow(unused_mut)]

use gerbil_parser_rowan::TreeEvent;

include!(concat!(env!("OUT_DIR"), "/org_rowan_events.rs"));
