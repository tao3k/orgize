#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Development-time POO graph projection AOT; Cargo reads the committed table.

(import (only-in :gerbil-parser/graph-projection-support
                 generate-graph-projection-rowan-module)
        (only-in "grammar.ss" org-v1-language-grammar)
        (only-in "graph.ss" org-v1-graph-projection))

(export main)

(def (main output-path)
  (generate-graph-projection-rowan-module
   output-path org-v1-language-grammar org-v1-graph-projection))
