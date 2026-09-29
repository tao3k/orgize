//! Scheme-owned document keyword algorithms lowered to typed Rust functions.

include!(concat!(env!("OUT_DIR"), "/keyword_word.rs"));
include!(concat!(env!("OUT_DIR"), "/keyword_words.rs"));
include!(concat!(env!("OUT_DIR"), "/keyword_tag_words.rs"));
include!(concat!(env!("OUT_DIR"), "/keyword_first_word.rs"));
include!(concat!(env!("OUT_DIR"), "/keyword_rest.rs"));
include!(concat!(env!("OUT_DIR"), "/keyword_option_value.rs"));
include!(concat!(env!("OUT_DIR"), "/keyword_option_present_p.rs"));
include!(concat!(env!("OUT_DIR"), "/keyword_boolean_value.rs"));
