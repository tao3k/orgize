//! Scheme-owned LOGBOOK line classification lowered to a typed Rust function.

include!(concat!(env!("OUT_DIR"), "/logbook_line_kind.rs"));
include!(concat!(env!("OUT_DIR"), "/logbook_content_line.rs"));
include!(concat!(env!("OUT_DIR"), "/logbook_state_quote_shape.rs"));
include!(concat!(env!("OUT_DIR"), "/logbook_state_to.rs"));
include!(concat!(env!("OUT_DIR"), "/logbook_state_from.rs"));
include!(concat!(env!("OUT_DIR"), "/logbook_clock_duration_shape.rs"));
include!(concat!(env!("OUT_DIR"), "/logbook_clock_duration_value.rs"));
