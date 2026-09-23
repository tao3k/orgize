#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Development-time POO graph projection AOT; Cargo reads the committed table.

(import (only-in :gerbil-parser/graph-projection-support
                 generate-graph-projection-rowan-module)
        (only-in "grammar.ss" org-v1-language-grammar)
        (only-in "graph.ss" org-v1-graph-projection))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-graph.ss OUTPUT.rs"))

(generate-graph-projection-rowan-module
 (car (reverse arguments))
 org-v1-language-grammar
 org-v1-graph-projection)
