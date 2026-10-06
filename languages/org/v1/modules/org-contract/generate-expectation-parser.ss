#!/usr/bin/env gxi
;;; Developer-only Rust metadata projection, never a second parser engine.
(import (only-in :gerbil-parser/rust-rowan-support generate-language-rust-rowan-module)
        (only-in "expectation-grammar.ss" org-expectation-language-grammar))
(generate-language-rust-rowan-module
 (car (reverse (command-line))) org-expectation-language-grammar)
