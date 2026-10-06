#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Org parser's native module graph; std/make owns ordering and compilation.
(import (only-in :std/build-script defbuild-script))
(def orgize-native-include
  (string-append "-I" (path-expand "bindings/c/include")))
(defbuild-script
 `("languages/org/v1/grammar"
   "languages/org/v1/parser"
   "languages/org/v1/modules/org-parser/entity-names"
   "languages/org/v1/modules/org-parser/event-babel-call"
   "languages/org/v1/modules/org-parser/event-headline-tags"
   "languages/org/v1/modules/org-parser/event-headline"
   "languages/org/v1/modules/org-parser/event-include"
   "languages/org/v1/modules/org-parser/event-inline-citation-reference"
   "languages/org/v1/modules/org-parser/event-inline-citation"
   "languages/org/v1/modules/org-parser/event-inline-cloze"
   "languages/org/v1/modules/org-parser/event-inline-entity"
   "languages/org/v1/modules/org-parser/event-inline-latex"
   "languages/org/v1/modules/org-parser/event-inline-link"
   "languages/org/v1/modules/org-parser/event-inline-primitives"
   "languages/org/v1/modules/org-parser/event-inline-script"
   "languages/org/v1/modules/org-parser/event-inline-timestamp"
   "languages/org/v1/modules/org-parser/event-inline-url"
   "languages/org/v1/modules/org-parser/event-inline"
   "languages/org/v1/modules/org-parser/event-inlinetask"
   "languages/org/v1/modules/org-parser/event-latex-environment"
   "languages/org/v1/modules/org-parser/event-list"
   "languages/org/v1/modules/org-parser/event-macro"
   "languages/org/v1/modules/org-parser/event-paragraph"
   "languages/org/v1/modules/org-parser/event-source-header"
   "languages/org/v1/modules/org-parser/event-source-content"
   "languages/org/v1/modules/org-parser/event-special-block"
   "languages/org/v1/modules/org-parser/event-strategy"
   "languages/org/v1/modules/org-parser/event-table-formula"
   "languages/org/v1/modules/org-parser/event-table"
   "languages/org/v1/modules/org-parser/event-tag-vocabulary"
   "languages/org/v1/modules/org-parser/runtime-funs"
   "languages/org/v1/modules/org-parser/objects"
   "languages/org/v1/modules/org-parser/types"
   "languages/org/v1/rowan-event-runtime"
   "languages/org/v1/modules/org-parser/macro-funs"
   "languages/org/v1/modules/org-parser/text-funs"
   "languages/org/v1/modules/org-parser/block-line-funs"
   "languages/org/v1/modules/org-parser/keyword-funs"
   "languages/org/v1/modules/org-parser/dir-path-funs"
   "languages/org/v1/modules/org-parser/family-funs"
   "languages/org/v1/modules/org-elements/headline-properties"
   "languages/org/v1/modules/org-elements/link-properties"
   "languages/org/v1/modules/org-elements/logbook-properties"
   "languages/org/v1/modules/org-elements/table-properties"
   "languages/org/v1/modules/org-elements/affiliated-properties"
   "languages/org/v1/modules/org-parser/duration-funs"
   "languages/org/v1/modules/org-parser/time-funs"
   "languages/org/v1/modules/org-parser/metadata-value-funs"
   "languages/org/v1/modules/org-parser/agenda-match-funs"
   "languages/org/v1/modules/org-parser/interactive-value-funs"
   "languages/org/v1/modules/org-parser/publishing-value-funs"
   "languages/org/v1/modules/org-parser/citation-export-value-funs"
   "languages/org/v1/modules/org-parser/unicode-context-data"
   "languages/org/v1/modules/org-parser/unicode-lower-data"
   "languages/org/v1/modules/org-parser/unicode-funs"
   "languages/org/v1/modules/org-parser/property-token-funs"
   "languages/org/v1/modules/org-parser/clock-window-funs"
   "languages/org/v1/modules/org-parser/link-protocol-funs"
   "languages/org/v1/modules/org-parser/value-funs"
   "languages/org/v1/modules/org-parser/source-value-funs"
   "languages/org/v1/modules/org-parser/contract-reference-funs"
   "languages/org/v1/modules/org-parser/source-header-policy-funs"
   "languages/org/v1/modules/org-elements/radio-match"
   "languages/org/v1/modules/org-contract/grammar"
   "languages/org/v1/modules/org-contract/parser"
   "languages/org/v1/modules/org-contract/contract-normalize"
   "languages/org/v1/modules/org-contract/expectation-grammar"
   "languages/org/v1/modules/org-contract/expectation-parser"
   ,@(cond-expand
      (darwin '((gxc: "bindings/c/native-clock" "-ld-options" "-Wl,-undefined,dynamic_lookup")
                (gxc: "bindings/c/native-runtime-statistics" "-ld-options" "-Wl,-undefined,dynamic_lookup")))
      (else '("bindings/c/native-clock" "bindings/c/native-runtime-statistics")))
   "languages/org/v1/rowan-event-tape"
   "bindings/c/orgize-actor"
   ,@(cond-expand
      (darwin `((gxc: "bindings/c/orgize-scheme-runtime" "-cc-options" ,orgize-native-include "-ld-options" "-Wl,-undefined,dynamic_lookup")))
      (else `((gxc: "bindings/c/orgize-scheme-runtime" "-cc-options" ,orgize-native-include))))
   ,@(cond-expand
      (darwin '((gxc: "bindings/c/orgize-parser" "-ld-options" "-Wl,-undefined,dynamic_lookup")))
      (else '("bindings/c/orgize-parser")))))
