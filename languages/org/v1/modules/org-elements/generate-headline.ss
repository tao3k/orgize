#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Generate the Org headline state function from its executable Scheme source.

(import (only-in :gerbil-parser/src/compiler/rust-syntax rust-render)
        (only-in "headline-properties.ss"
                 todo-state-from-directives-rust))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-headline.ss OUTPUT.rs"))

(call-with-output-file (car (reverse arguments))
  (lambda (port)
    (write-string (rust-render todo-state-from-directives-rust) port)))
