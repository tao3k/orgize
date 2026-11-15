#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Development-time POO graph projection AOT; Cargo reads the committed table.

(import (only-in :std/io/tempfile call-with-temporary-directory)
        (only-in :std/misc/ports read-all-as-string)
        (only-in :gerbil-parser/graph-projection-support
                 generate-graph-projection-runtime-module)
        (only-in :orgize/languages/org/modules/org-parser/grammar org-mode-language-grammar)
        (only-in "graph.ss" org-graph-projection))

(export main)

(def (main output-path)
  (let-values (((comment declarations)
                (call-with-temporary-directory "orgize-graph"
                  (lambda (directory)
                    (let (path (path-expand "graph.rs" directory))
                      (generate-graph-projection-runtime-module
                       path org-mode-language-grammar org-graph-projection)
                      (call-with-input-file path
                        (lambda (port)
                          (let (comment (read-line port))
                            (unless (string-prefix? "use gerbil_parser_runtime::" (read-line port))
                              (error "unexpected graph declaration imports"))
                            (values comment (read-all-as-string port))))))))))
    (call-with-output-file output-path
      (lambda (port)
        (display comment port)
        (newline port)
        (display "use crate::org_product::{GraphFieldMode, GraphFieldRule, GraphNodeRule, GraphProjectionSpec};\n" port)
        (display declarations port)))))
