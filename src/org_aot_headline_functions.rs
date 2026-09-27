//! Scheme-AOT headline functions share one Rust module for typed composition.

include!(concat!(env!("OUT_DIR"), "/todo_state_from_directives.rs"));
include!(concat!(env!("OUT_DIR"), "/todo_keyword_from_directives.rs"));
include!(concat!(env!("OUT_DIR"), "/headline_content_after_todo.rs"));
include!(concat!(env!("OUT_DIR"), "/priority_token_p.rs"));
include!(concat!(env!("OUT_DIR"), "/headline_display_title.rs"));
include!(concat!(env!("OUT_DIR"), "/headline_comment_p.rs"));
include!(concat!(env!("OUT_DIR"), "/memory_headline_state.rs"));
