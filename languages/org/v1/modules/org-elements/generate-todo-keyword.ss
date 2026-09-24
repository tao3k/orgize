#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Generate the Org TODO keyword predicate from executable Scheme source.

(import (only-in :gerbil-parser/src/compiler/rust-syntax rust-render)
        (only-in "headline-properties.ss" todo-keyword-matches-rust))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-todo-keyword.ss OUTPUT.rs"))

(call-with-output-file (car (reverse arguments))
  (lambda (port)
    (write-string (rust-render todo-keyword-matches-rust) port)))
