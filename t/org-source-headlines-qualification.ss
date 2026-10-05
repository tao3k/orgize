;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later

;;; A separate process keeps the source parser and Rust-pure headline module
;;; out of the legacy all-suites gxtest image. Each marker follows a real check.
;;; Each completion follows an actual compiled owner dependency import.
;;; The outer five-second watchdog also applies to every import and first case.
(for-each (lambda (module)
  (eval `(import ,module))
  (displayln "SOURCE-HEADLINE-IMPORT-OK " module) (force-output))
  '(
    :std/error
    :std/func
    :clan/poo/support/base
    :std/values
    :std/stxparam
    :std/string/symbol
    :std/list/list-builder
    :std/list/list
    :std/iter/interface
    :std/interface
    :std/iter/iterators
    :std/iter/macros
    :std/iter/api
    :std/iter
    :std/hash/misc
    :clan/poo/object
    :clan/poo/brace
    :clan/poo/support/repr
    :std/serde/scan
    :std/encoding/json/env
    :std/text/parser/char-set
    :std/encoding/hex
    :std/number/misc
    :std/ffi
    :std/vector/u8vector
    :std/text/parser/base
    :std/io/interface/base
    :std/string/utf8
    :std/net/address/types
    :std/time/time
    :std/time/timeout
    :std/os/error
    :std/os/time
    :std/os/sockaddr
    :std/os/device
    :std/os/fcntl
    :std/os/socket
    :std/os/sockopt
    :std/io/interface/socket
    :std/io/interface/bio
    :std/io/interface
    :std/os/file
    :std/os/flock
    :std/io/file
    :std/encoding/base64
    :std/crypto/libcrypto
    :std/crypto/error
    :std/crypto/random
    :std/io/tempfile
    :std/io/bio/types
    :std/io/bio/macros
    :std/sync/spinlock
    :std/cache
    :std/io/bio/cache
    :std/io/bio/buffer
    :std/io/bio/output
    :std/io/bio/input
    :std/io/bio/delimited
    :std/io/bio/port
    :std/io/bio/srcsnk
    :std/io/bio/memory
    :std/io/bio/writer
    :std/io/bio/reader
    :std/io/bio/api
    :std/io/util
    :std/io/detachable
    :std/io/delimited
    :std/io/dummy
    :std/io/counter
    :std/io/socket/sockaddr
    :std/sync/rwlock
    :std/serde/util
    :std/serde/interface
    :std/hash/types
    :std/serde/deserialize
    :std/net/address/serde
    :std/format/ascii
    :std/format/env
    :std/number/writer
    :std/serde/interned
    :std/serde/scanner
    :std/serde/serialize
    :std/format/ioutil
    :std/format/reader
    :std/format/writer
    :std/format/io
    :std/format/format-string
    :std/format/format
    :std/format/api
    :std/format
    :std/string/stringer
    :std/net/address/stringer
    :std/list/random
    :std/net/address/resolver
    :std/text/pregexp
    :std/net/address/parser
    :std/net/address/api
    :std/net/address
    :std/io/socket/types
    :std/io/socket/basic
    :std/io/socket/datagram
    :std/io/socket/server
    :std/io/socket/client
    :std/io/socket/stream
    :std/io/socket/socket
    :std/io/socket/api
    :std/io/api
    :std/io
    :std/text/parser/ll1
    :std/list/alist
    :std/list/walist
    :std/encoding/json/reader
    :std/encoding/json/writer
    :std/encoding/json/io
    :std/misc/ports
    :std/list/plist
    :std/encoding/json/util
    :std/encoding/json/api
    :std/encoding/json
    :clan/poo/support/json
    :std/assert
    :clan/poo/support/io
    :clan/poo/mop
    :orgize/languages/org/v1/modules/org-elements/graph-types
    :orgize/languages/org/v1/modules/org-elements/graph-objects
    :orgize/languages/org/v1/graph-shape
    :core/types
    :gerbil-parser/src/modules/parser/graph-query-types
    :core/object-family/syntax
    :core/object-family/funcs
    :core/object-family/indexed
    :core/object-family/interface
    :gerbil-parser/src/modules/parser/graph-query-objects
    :gerbil-parser/src/modules/parser/graph-query-funs
    :gerbil-parser/graph-query-support
    :orgize/languages/org/v1/modules/org-elements/types
    :orgize/languages/org/v1/modules/org-elements/objects
    :gerbil-parser/src/compiler/rust-syntax
    :gerbil-parser/src/utilities/strings
    :std/string/misc
    :gerbil-parser/src/compiler/rust-pure-aot
    :orgize/languages/org/v1/modules/org-elements/headline-properties
    :gerbil-parser/src/modules/parser/types
    :gerbil-parser/src/modules/parser/line-structure-types
    :gerbil-parser/src/modules/parser/line-structure-objects
    :orgize/languages/org/v1/modules/org-parser/types
    :orgize/languages/org/v1/modules/org-parser/objects
    :gerbil-parser/src/modules/parser/source-fragment-types
    :gerbil-parser/src/modules/parser/source-fragment-objects
    :gerbil-parser/src/modules/parser/source-fragment-funs
    :gerbil-parser/src/runtime/token
    :std/vector/vector
    :gerbil-parser/src/runtime/scan
    :core/observability/types
    :core/observability/funcs
    :clan/poo/io
    :clan/poo/number
    :clan/poo/type
    :clan/poo/support/debug
    :clan/poo/debug
    :core/observability/slot-debug
    :core/observability/debug
    :gerbil-parser/src/runtime/observability
    :gerbil-parser/src/runtime/lr-action-index
    :gerbil-parser/src/compiler/funcs
    :gerbil-parser/src/compiler/lr
    :gerbil-parser/src/runtime/layout
    :gerbil-parser/src/runtime/event-program
    :gerbil-parser/src/runtime/recognition
    :gerbil-parser/src/runtime/funcs
    :gerbil-parser/src/runtime/reduce
    :gerbil-parser/src/runtime/event-reduce
    :gerbil-parser/src/runtime/lr-parser
    :gerbil-parser/src/compiler/machine
    :std/crypto/digest
    :gerbil-parser/src/runtime/identity
    :gerbil-parser/src/language/descriptor
    :gerbil-parser/src/compiler/event-strategy-aot
    :gerbil-parser/src/compiler/event-fold-future
    :gerbil-parser/src/compiler/event-list-marker
    :gerbil-parser/src/compiler/event-fold-runtime
    :gerbil-parser/src/compiler/event-fold-ir
    :gerbil-parser/src/compiler/event-fold-aot
    :gerbil-parser/rust-rowan-event-support
    :orgize/languages/org/v1/modules/org-parser/funs
    :orgize/languages/org/v1/modules/org-parser/event-source-header
    :orgize/languages/org/v1/modules/org-parser/event-inline-primitives
    :orgize/languages/org/v1/modules/org-parser/event-inline-cloze
    :orgize/languages/org/v1/modules/org-parser/event-inline-script
    :orgize/languages/org/v1/modules/org-parser/event-inline-latex
    :orgize/languages/org/v1/modules/org-parser/entity-names
    :orgize/languages/org/v1/modules/org-parser/event-inline-entity
    :orgize/languages/org/v1/modules/org-parser/event-inline-timestamp
    :orgize/languages/org/v1/modules/org-parser/event-inline-citation-reference
    :orgize/languages/org/v1/modules/org-parser/event-inline-citation
    :orgize/languages/org/v1/modules/org-parser/event-inline-url
    :orgize/languages/org/v1/parser
    :orgize/languages/org/v1/modules/org-parser/event-inline-link
    :orgize/languages/org/v1/modules/org-parser/event-inline
    :orgize/languages/org/v1/modules/org-parser/event-headline-tags
    :orgize/languages/org/v1/modules/org-parser/event-table-formula
    :orgize/languages/org/v1/modules/org-parser/event-babel-call
    :orgize/languages/org/v1/modules/org-parser/event-paragraph
    :orgize/languages/org/v1/modules/org-parser/event-headline
    :orgize/languages/org/v1/modules/org-parser/event-latex-environment
    :orgize/languages/org/v1/modules/org-parser/event-special-block
    :orgize/languages/org/v1/modules/org-parser/event-list
    :orgize/languages/org/v1/modules/org-parser/event-tag-vocabulary
    :orgize/languages/org/v1/modules/org-parser/event-table
    :orgize/languages/org/v1/modules/org-parser/event-include
    :orgize/languages/org/v1/modules/org-parser/event-inlinetask
    :orgize/languages/org/v1/modules/org-parser/event-strategy
    :gerbil-parser/src/runtime/artifact
    :gerbil-parser/src/language/source
    :gerbil-parser/src/runtime/significant
    :gerbil-parser/src/runtime/contextual-scanner
    :gerbil-parser/src/runtime/lexer
    :gerbil-parser/src/runtime/parser-ir-data
    :gerbil-parser/src/runtime/parser
    :gerbil-parser/src/language/entry
    :std/encoding/zlib
    :gerbil-parser/src/runtime/language-artifact
    :gerbil-parser/src/language/grammar
    :gerbil-parser/language-support/iso-bnf
    :gerbil-parser/language-support/antlr4-language
    :gerbil-parser/language-support/iso-bnf-language
    :gerbil-parser/language-support/javacc-source
    :gerbil-parser/language-support/antlr4-source-lexer
    :gerbil-parser/language-support/antlr4-source
    :gerbil-parser/language-support/grammar-source
    :gerbil-parser/language-support/fixture
    :gerbil-parser/language-support
    :orgize/languages/org/v1/grammar
    :orgize/languages/org/v1/rowan-event-parser
    :orgize/languages/org/v1/modules/org-elements/source-headlines))

(import (only-in :clan/poo/object .ref)
        :orgize/languages/org/v1/modules/org-elements/source-interface)

(def (require-equal name actual expected)
  (unless (equal? actual expected)
    (error "Org source headline qualification mismatch" name actual expected)))

(def (case-ok name)
  (display "SOURCE-HEADLINE-CASE-OK ")
  (display name)
  (newline)
  (force-output))

(let* ((source
        "#+SEQ_TODO: WAIT | DONE\n* WAIT α\n#+BEGIN_SRC text\n* fake\n#+END_SRC\n* DONE β\n")
       (view (org-source-headline-elements source))
       (records (.ref view 'elements))
       (first (car records))
       (second (cadr records)))
  (require-equal 'view (org-source-headline-elements? view) #t)
  (require-equal 'count (length records) 2)
  (require-equal 'first-title (.ref first 'title) "WAIT α")
  (require-equal 'first-state (.ref first 'todo-type) "todo")
  (require-equal 'first-start (.ref first 'byte-start) 24)
  (require-equal 'first-end (.ref first 'byte-end) 34)
  (require-equal
   'first-id (.ref first 'identity)
   (string-append (.ref view 'source-sha256) ":24:34"))
  (require-equal 'second-title (.ref second 'title) "DONE β")
  (require-equal 'second-state (.ref second 'todo-type) "done")
  (require-equal 'worktree-bound (.ref view 'worktree-bound?) #f)
  (require-equal 'action-authority (.ref view 'action-authority?) #f)
  (case-ok 'source-structure))

(let ((old (org-source-headline-elements "* TODO Open\n"))
      (new (org-source-headline-elements "* DONE Open\n")))
  (when (equal? (.ref old 'source-sha256) (.ref new 'source-sha256))
    (error "changed source retained its identity"))
  (require-equal 'changed-state
                 (.ref (car (.ref new 'elements)) 'todo-type) "done")
  (case-ok 'changed-bytes))

(let (view
      (org-source-headline-elements
       "*************** TODO Inline\nBody.\n*************** END\n* TODO Next\n"))
  (require-equal 'inlinetask-exclusion (length (.ref view 'elements)) 1)
  (require-equal 'remaining-title
                 (.ref (car (.ref view 'elements)) 'title) "TODO Next")
  (case-ok 'inlinetask-exclusion))

(require-equal
 'source-bound
 (with-catch
  (lambda (error-value) (error-message error-value))
  (lambda ()
    (org-source-headline-elements (make-string 1048577 #\x))
    'accepted))
 "Org source headline projection exceeds one MiB")
(case-ok 'source-bound)

(display "SOURCE-HEADLINE-OK")
(newline)
(force-output)
