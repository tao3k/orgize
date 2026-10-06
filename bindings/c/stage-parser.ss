;;; -*- Gerbil -*-
;;; The existing native-build owner stages the compiler's program closure.
(import (only-in :gerbil-scheme-rust/scheme/program-build gerbil-rs-stage-program))
(export main)
(def (main output-dir)
  (displayln "ORG-PARSER-STAGE begin")
  (force-output)
  (gerbil-rs-stage-program "bindings/c/orgize-parser.ss" output-dir
                          c-headers: '("bindings/c/include/orgize.h"
                                       "bindings/c/include/orgize_runtime.h"))
  (displayln "ORG-PARSER-STAGE OK"))
