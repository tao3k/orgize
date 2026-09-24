//! Scheme-AOT headline functions share one Rust module for typed composition.

include!(concat!(env!("OUT_DIR"), "/todo_state_from_directives.rs"));
include!(concat!(env!("OUT_DIR"), "/todo_keyword_matches_p.rs"));
include!(concat!(env!("OUT_DIR"), "/todo_keyword_from_directives.rs"));
include!(concat!(env!("OUT_DIR"), "/headline_content_after_todo.rs"));
