;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; Explicit filesystem boundary; Scheme owns composition and validation.
(import (only-in :clan/poo/object .o .ref .slot?)
        (only-in :std/list/list filter every find)
        (only-in :std/misc/ports read-all-as-string)
        (only-in :orgize/languages/org/modules/org-elements/values
                 org-element-graph-records)
        (only-in :orgize/languages/org/modules/org-elements/source-graph
                 org-source-element-graph org-source-element-field)
        (only-in "value-types.ss" org-directory-profile?)
        (only-in "values.ss" org-contract-definition-id org-contract-definition-scope)
        (only-in "source.ss" org-source-contract-definitions)
        (only-in "directory.ss" org-directory-contracts
                 org-directory-evaluate))
(export org-directory-validate org-directory-validation-current?)

;; No symlink traversal: a cycle or out-of-root link cannot silently enlarge
;; coverage. Hidden directories are not omitted; directory-files supplies all.
(def (org-files root)
  (let ((entries 0))
    (def (walk path)
      (set! entries (+ entries 1))
      (when (> entries 16384) (error "Org Directory scan exceeds 16384 entries"))
      (let (kind (file-info-type (file-info path #f)))
        (case kind
          ((directory)
           (apply append
             (map (lambda (name) (walk (path-expand name path)))
                  (list-sort string<? (directory-files path)))))
          ((regular) (if (equal? (path-extension path) ".org") (list path) '()))
          (else (error "unsupported entry in Org Directory scan" path kind)))))
    (unless (eq? (file-info-type (file-info root #f)) 'directory)
      (error "Org Directory root is not a directory" root))
    (walk root)))

(def (read-document path)
  (when (> (file-info-size (file-info path #f)) 1048576)
    (error "Org document exceeds one MiB" path))
  (let (text (call-with-input-file path read-all-as-string))
    (when (> (u8vector-length (string->utf8 text)) 1048576)
      (error "Org document exceeds one MiB" path))
    (let ((graph-value (org-source-element-graph text)) (path-value path))
      (.o path: path-value source: text graph: graph-value
          contracts: (org-source-contract-definitions graph-value)))))

(def (snapshot profile)
  (unless (org-directory-profile? profile) (error "invalid Org Directory Profile"))
  (let* ((workspace (or (.ref profile 'workspace) (current-directory)))
         (root-value (path-normalize (path-expand (.ref profile 'directory) workspace)))
         (contract-directory (.ref (.ref profile 'contracts) 'directory))
         (common-root (and contract-directory (path-normalize (path-expand contract-directory workspace))))
         (documents-value (map read-document (org-files root-value)))
         (common-value (if common-root (map read-document (org-files common-root)) '())))
    (when (null? documents-value) (error "Org Directory contains no Org documents" root-value))
    ;; Keep source snapshots separate from input documents even if DI lives
    ;; beneath the adopted root. Their own declarations are not work inputs.
    (when (and common-root
               (null? (apply append (map (lambda (d) (.ref d 'contracts)) common-value))))
      (error "common Contract Directory contains no declarations" common-root))
    (let (common-root-value common-root)
      (.o root: root-value documents: documents-value common-root: common-root-value common: common-value))))

(def (identity-cut snapshot-value)
  (list (.ref snapshot-value 'root) (.ref snapshot-value 'common-root)
        (map (lambda (d) (list (.ref d 'path) (.ref d 'source))) (.ref snapshot-value 'documents))
        (map (lambda (d) (list (.ref d 'path) (.ref d 'source))) (.ref snapshot-value 'common))))

(def (validate-snapshot profile snapshot-value)
  (let* ((common (.ref snapshot-value 'common))
         (definitions (org-directory-contracts profile
                        (apply append (map (lambda (d) (.ref d 'contracts)) common))))
         (common-paths (map (lambda (d) (.ref d 'path)) common))
         (documents (filter (lambda (d) (not (member (.ref d 'path) common-paths)))
                            (.ref snapshot-value 'documents)))
         (stable-ids (make-hash-table)) (evaluations '()))
    (when (null? documents)
      (error "Org Directory has no work documents after common DI exclusion"))
    (for-each
     (lambda (document)
       (let* ((graph (.ref document 'graph)) (records (org-element-graph-records graph))
              (local (.ref document 'contracts))
              (combined (org-directory-contracts profile
                          (append (apply append (map (lambda (d) (.ref d 'contracts)) common)) local))))
         (for-each
          (lambda (record)
            (let (id (org-source-element-field record "property:ID"))
              (when id
                (when (or (equal? id "") (hash-get stable-ids id))
                  (error "duplicate or empty stable Org ID" id))
                (hash-put! stable-ids id #t)))) records)
         (for-each
          (lambda (definition)
            (let* ((id (org-contract-definition-id definition))
                   (targets
                    (if (eq? (org-contract-definition-scope definition) 'document)
                      '(0)
                      (map (lambda (e) (.ref e 'id))
                        (filter (lambda (e)
                                  (and (equal? (.ref e 'kind) "headline")
                                       (or (equal? (org-source-element-field e "property:CONTRACT_REF") id)
                                           (and (memq definition local)
                                                (.slot? definition 'source-scope-id)
                                                (equal? (.ref e 'id) (.ref definition 'source-scope-id)))
                                           (and (.slot? definition 'target-property)
                                                (.ref definition 'target-property)
                                                (equal? (org-source-element-field e
                                                          (string-append "property:" (.ref definition 'target-property)))
                                                        (.ref definition 'target-value))))))
                                records)))))
              (for-each
               (lambda (target)
                 (let* ((record (find (lambda (e) (equal? (.ref e 'id) target)) records))
                        (stable-id-value (and record (org-source-element-field record "property:ID"))))
                   (when (and (not (= target 0)) (not stable-id-value))
                     (error "addressed Org Contract subtree lacks stable ID" id target))
                 (let (validation-value (org-directory-evaluate (list definition) graph target))
                   (set! evaluations
                     (cons (.o path: (.ref document 'path) contract-id: id
                               scope-id: target scope-stable-id: stable-id-value
                               validation: validation-value) evaluations))))) targets)))
          combined)
         ;; A reference is an obligation: an unknown contract is not skipped.
         (for-each
          (lambda (record)
            (let (ref (org-source-element-field record "property:CONTRACT_REF"))
              (when (and ref
                         (not (member ref (map org-contract-definition-id combined))))
                (error "unresolved Org Contract reference" ref)))) records))) documents)
    (let ((evaluations-value (reverse evaluations))
          (profile-value profile)
          (cut-value (identity-cut snapshot-value)))
      (.o kind: 'org-directory-validation
          schema: "orgize.directory-validation.v1" producer: "orgize"
          root: (.ref snapshot-value 'root)
          source-cut: cut-value profile: profile-value
          files: (map (lambda (d) (.ref d 'path)) documents)
          evaluations: evaluations-value
          complete?: #t
          passed?: (every (lambda (e) (.ref (.ref e 'validation) 'passed?)) evaluations-value)
          action-authority?: #f))))

(def (org-directory-validate profile)
  (let* ((before (snapshot profile)) (validation (validate-snapshot profile before))
         (after (snapshot profile)))
    (unless (equal? (identity-cut before) (identity-cut after))
      (error "Org Directory changed during validation; obligations reopened"))
    validation))

;; Reuse is conditional on the whole current input inventory and source cut.
;; This does not claim an atomic filesystem snapshot or Governance grant.
(def (org-directory-validation-current? validation profile)
  (and (.slot? validation 'kind) (eq? (.ref validation 'kind) 'org-directory-validation)
       (eq? (.ref validation 'profile) profile)
       (equal? (.ref validation 'source-cut) (identity-cut (snapshot profile)))))
