#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; AOT projection of Scheme-owned document keyword algorithms.

(import (only-in :gerbil-parser/src/compiler/rust-syntax
                 write-rust-function-ir)
        (only-in "document-keyword-properties.ss"
                 keyword-word-rust keyword-words-rust
                 keyword-tag-words-rust keyword-first-word-rust
                 keyword-rest-rust keyword-option-value-rust
                 keyword-option-present-rust))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-document-keyword-ir.ss OUTPUT_DIR"))

(def output-dir (car (reverse arguments)))
(for-each
 (lambda (entry)
   (write-rust-function-ir
    (path-expand (car entry) output-dir)
    (cdr entry)))
 (list (cons "keyword_word.ir.json" keyword-word-rust)
       (cons "keyword_words.ir.json" keyword-words-rust)
       (cons "keyword_tag_words.ir.json" keyword-tag-words-rust)
       (cons "keyword_first_word.ir.json" keyword-first-word-rust)
       (cons "keyword_rest.ir.json" keyword-rest-rust)
       (cons "keyword_option_value.ir.json" keyword-option-value-rust)
       (cons "keyword_option_present_p.ir.json" keyword-option-present-rust)))
