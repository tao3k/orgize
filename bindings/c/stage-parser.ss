;;; -*- Gerbil -*-
;;; The existing native-build owner stages the compiler's program closure.
(import (only-in :gerbil-scheme-rust/scheme/program-build gerbil-rs-stage-program))
(export main)
(def (main output-dir)
  (displayln "ORG-PARSER-STAGE begin")
  (force-output)
  ;; Headers are explicit native build contract inputs, not an extension to
  ;; the upstream compiler manifest schema.
  (gerbil-rs-stage-program "bindings/c/orgize-parser.ss" output-dir)
  (displayln "ORG-PARSER-STAGE OK"))
