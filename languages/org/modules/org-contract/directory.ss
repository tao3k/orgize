;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; Directory composition kernel. Filesystem loading is an explicit boundary.

(import (only-in :clan/poo/object .o .ref .slot?)
        (only-in :std/list/list every)
        (only-in "types.ss" org-contract-definition? org-directory-profile?)
        (only-in "objects.ss" make-org-contract-definition
                 make-org-contract-expectation make-org-contract-assertion
                 org-contract-definition-id
                 org-contract-definition-assertions org-contract-assertion-id
                 org-contract-result-passed?)
        (only-in "funs.ss" org-contract-evaluate-definition)
        (only-in "../org-elements/runtime-interface.ss" make-org-element-query))
(export org-directory-contracts org-directory-evaluate)

(def mandatory-document-contract
  (make-org-contract-definition "orgize.directory.document-root.v1" 'document
    (list (make-org-contract-assertion "document.one-root" 'error
            (make-org-element-query "org-data" #f #f 'at 'scope)
            (make-org-contract-expectation 'exactly 1)))))

(def (definitions? values)
  (and (list? values) (every org-contract-definition? values)))

(def (org-directory-contracts profile common-definitions)
  (unless (and (org-directory-profile? profile)
               (definitions? common-definitions))
    (error "invalid Org Directory Profile or common Contract input"))
  (let ((seen (make-hash-table))
        (definitions
         (append (list mandatory-document-contract) (.ref profile 'builtin) common-definitions
                 (if (.slot? (.ref profile 'contracts) 'definitions)
                     (.ref (.ref profile 'contracts) 'definitions) '())
                 (.ref profile 'local))))
    (for-each
     (lambda (definition)
       (let ((id (org-contract-definition-id definition))
             (assertions (make-hash-table)))
         (when (hash-get seen id)
           (error "conflicting Org Contract identity" id))
         (hash-put! seen id #t)
         (for-each
          (lambda (assertion)
            (let (id (org-contract-assertion-id assertion))
              (when (hash-get assertions id)
                (error "duplicate Org Contract assertion identity" id))
              (hash-put! assertions id #t)))
          (org-contract-definition-assertions definition))))
     definitions)
    definitions))

;; Pure conjunction over admitted definitions and an exact source-derived graph.
;; This result is validation, not Governance authorization or owner authority.
(def (org-directory-evaluate definitions graph scope-id)
  (unless (definitions? definitions)
    (error "Org Directory evaluation requires admitted Contracts"))
  (let (results-value
        (apply append
               (map (lambda (definition)
                      (org-contract-evaluate-definition definition graph scope-id))
                    definitions)))
    (.o results: results-value
        passed?: (every org-contract-result-passed? results-value)
        action-authority?: #f)))
