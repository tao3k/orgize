#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Development-time POO graph projection AOT; Cargo reads the committed table.

(import (only-in :gerbil-parser/graph-projection-support
                 generate-graph-projection-runtime-module)
        (only-in "grammar.ss" org-mode-language-grammar)
        (only-in "graph.ss" org-graph-projection))

(export main)

(def (main output-path)
  (generate-graph-projection-runtime-module
   output-path org-mode-language-grammar org-graph-projection))
