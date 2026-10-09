#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Org parser's native module graph; std/make owns ordering and compilation.
(import (only-in :std/build-script defbuild-script))
(def orgize-native-include
  (string-append "-I" (path-expand "bindings/c/include")))
(def orgize-syntax-modules
 '("languages/org/grammar"
   "languages/org/parser"
   "languages/org/modules/org-parser/entity-names"
   "languages/org/modules/org-parser/event-babel-call"
   "languages/org/modules/org-parser/event-headline-tags"
   "languages/org/modules/org-parser/event-headline"
   "languages/org/modules/org-parser/event-include"
   "languages/org/modules/org-parser/event-inline-citation-reference"
   "languages/org/modules/org-parser/event-inline-citation"
   "languages/org/modules/org-parser/event-inline-cloze"
   "languages/org/modules/org-parser/event-inline-entity"
   "languages/org/modules/org-parser/event-inline-latex"
   "languages/org/modules/org-parser/event-inline-link"
   "languages/org/modules/org-parser/event-inline-primitives"
   "languages/org/modules/org-parser/event-inline-script"
   "languages/org/modules/org-parser/event-inline-timestamp"
   "languages/org/modules/org-parser/event-inline-url"
   "languages/org/modules/org-parser/event-inline"
   "languages/org/modules/org-parser/event-inlinetask"
   "languages/org/modules/org-parser/event-latex-environment"
   "languages/org/modules/org-parser/event-list"
   "languages/org/modules/org-parser/event-macro"
   "languages/org/modules/org-parser/event-paragraph"
   "languages/org/modules/org-parser/event-source-header"
   "languages/org/modules/org-parser/event-source-content"
   "languages/org/modules/org-parser/event-special-block"
   "languages/org/modules/org-parser/event-strategy"
   "languages/org/modules/org-parser/event-table-formula"
   "languages/org/modules/org-parser/event-table"
   "languages/org/modules/org-parser/event-tag-vocabulary"
   "languages/org/modules/org-parser/objects"
   "languages/org/modules/org-parser/types"))

(defbuild-script
 (if (getenv "ORGIZE_BUILD_SYNTAX_ONLY" #f)
   orgize-syntax-modules
 `(,@orgize-syntax-modules
   "languages/org/native-event-runtime"
   "languages/org/modules/org-parser/macro-funs"
   "languages/org/modules/org-parser/text-funs"
   "languages/org/modules/org-parser/block-line-funs"
   "languages/org/modules/org-parser/keyword-funs"
   "languages/org/modules/org-parser/dir-path-funs"
   "languages/org/modules/org-parser/family-funs"
   "languages/org/modules/org-elements/headline-properties"
   "languages/org/modules/org-elements/link-properties"
   "languages/org/modules/org-elements/logbook-properties"
   "languages/org/modules/org-elements/table-properties"
   "languages/org/modules/org-elements/affiliated-properties"
   "languages/org/modules/org-parser/duration-funs"
   "languages/org/modules/org-parser/time-funs"
   "languages/org/modules/org-parser/metadata-value-funs"
   "languages/org/modules/org-parser/agenda-match-funs"
   "languages/org/modules/org-parser/interactive-value-funs"
   "languages/org/modules/org-parser/publishing-value-funs"
   "languages/org/modules/org-parser/citation-export-value-funs"
   "languages/org/modules/org-parser/unicode-context-data"
   "languages/org/modules/org-parser/unicode-lower-data"
   "languages/org/modules/org-parser/unicode-funs"
   "languages/org/modules/org-parser/property-token-funs"
   "languages/org/modules/org-parser/clock-window-funs"
   "languages/org/modules/org-parser/link-protocol-funs"
   "languages/org/modules/org-parser/value-funs"
   "languages/org/modules/org-parser/source-value-funs"
   "languages/org/modules/org-parser/contract-reference-funs"
   "languages/org/modules/org-parser/source-header-policy-funs"
   "languages/org/modules/org-elements/radio-match"
   "languages/org/modules/org-contract/grammar"
   "languages/org/modules/org-contract/parser"
   "languages/org/modules/org-contract/contract-normalize"
   "languages/org/modules/org-contract/document-plan"
   "languages/org/modules/org-contract/expectation-grammar"
   "languages/org/modules/org-contract/expectation-parser"
   ,@(cond-expand
      (darwin '((gxc: "bindings/c/native-clock" "-ld-options" "-Wl,-undefined,dynamic_lookup")
                (gxc: "bindings/c/native-runtime-statistics" "-ld-options" "-Wl,-undefined,dynamic_lookup")))
      (else '("bindings/c/native-clock" "bindings/c/native-runtime-statistics")))
   "languages/org/native-event-tape"
   "bindings/c/orgize-actor"
   ,@(cond-expand
      (darwin `((gxc: "bindings/c/orgize-scheme-runtime" "-cc-options" ,orgize-native-include "-ld-options" "-Wl,-undefined,dynamic_lookup")))
      (else `((gxc: "bindings/c/orgize-scheme-runtime" "-cc-options" ,orgize-native-include))))
   ,@(cond-expand
      (darwin '((gxc: "bindings/c/orgize-parser" "-ld-options" "-Wl,-undefined,dynamic_lookup")))
      (else '("bindings/c/orgize-parser"))))))
