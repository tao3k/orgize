//! Scheme-authored Org event algorithm compiled to a Rust function at build time.

use gerbil_parser_rowan::TreeEvent;

include!(concat!(env!("OUT_DIR"), "/org_rowan_events.rs"));
