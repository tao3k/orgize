#!/usr/bin/env gxi
;;; Developer-only Rust metadata projection, never a second parser engine.
(import (only-in :gerbil-parser/rust-runtime-support generate-language-rust-runtime-module)
        (only-in "expectation-grammar.ss" org-expectation-language-grammar))
(generate-language-rust-runtime-module
 (car (reverse (command-line))) org-expectation-language-grammar)
