;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; Native graph-query API compatibility, independent of grammar generation.
(import :std/test
        (only-in :clan/poo/object .o .cc .ref)
        (only-in :clan/poo/mop TypeError?)
        (only-in :gerbil-parser/graph-query-support graph-query-context?)
        (only-in :orgize/languages/org/modules/org-elements/objects
                 make-org-element-graph-view)
        (only-in :orgize/languages/org/modules/org-elements/funs
                 make-org-element-query-context org-element-map org-element-lineage?))
(export org-elements-graph-types-test)

(def graph
  (make-org-element-graph-view
   (list (.o id: 0 parent: #f kind: "org-data")
         (.o id: 1 parent: 0 kind: "headline"))
   (lambda (row) (.ref row 'id))
   (lambda (row) (.ref row 'parent))
   (lambda (row) (.ref row 'kind))
   (lambda (row field) #f)))

(def org-elements-graph-types-test
  (test-suite "Org graph binds language Types to native Parser indexing"
    (test-case "unsigned identities and textual kinds retain map and lineage"
      (let (context (make-org-element-query-context graph))
        (check (graph-query-context? (.ref context 'index)) => #t)
        (check (length (org-element-map context "headline" (lambda (row) #t))) => 1)
        (check (org-element-lineage? context 0 1) => #t)))
    (test-case "the absent-parent sentinel cannot become a node identity"
      ;; The checked native callback rejects the result before indexing it.
      (check-exception
       (make-org-element-query-context (.cc graph 'id-of (lambda (row) #f)))
       TypeError?))))
