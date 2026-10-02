;;; -*- Gerbil -*-
;;; Org TBLFM syntax is composed from admitted POO source-fragment policies.
;;; The same helpers run in Scheme tests and lower through Rowan event AOT.

(import (only-in :gerbil-parser/rust-rowan-event-support
                 make-source-delimited-fragment
                 source-delimited-fragment-initial source-delimited-fragment-forms
                 make-source-first-split
                 source-first-split-initial source-first-split-forms
                 make-source-reference-scan
                 source-reference-scan-initial source-reference-scan-forms)
        (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure-key-lines key-line-node
                 key-line-prefix key-line-separator)
        (only-in "../../parser.ss" org-v1-line-structure)
        (only-in "objects.ss" make-org-event-helper))
(export table-formula-marker table-formula-event-helpers)

(def keyword-rule
  (or (ormap (lambda (rule)
               (and (eq? (key-line-node rule) 'OrgKeyword) rule))
             (line-structure-key-lines org-v1-line-structure))
      (error "Org keyword line policy is missing")))
(def table-formula-marker
  (string-append (key-line-prefix keyword-rule) "TBLFM"
                 (key-line-separator keyword-rule)))

(def assignment-sequence
  (make-source-delimited-fragment "::" 'FormulaSeparator
                                  'table-formula-assignment 'FormulaTrivia))
(def assignment-equals
  (make-source-first-split "=" 'FormulaEquals
                           'table-formula-lhs 'table-formula-rhs-flags))
(def rhs-flags
  (make-source-first-split ";" 'FormulaFlagSeparator
                           'table-formula-rhs 'table-formula-flags))
(def flag-sequence
  (make-source-delimited-fragment ";" 'FormulaFlagSeparator
                                  'table-formula-flag 'FormulaTrivia))
(def references
  (make-source-reference-scan
   '((#\$ . FormulaFieldReference) (#\@ . FormulaRowReference))
   "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789$@#<>_+-"
   "remote(" 'FormulaRemoteReference 'OrgTableFormulaReference 'FormulaText))

(def (policy-helper name policy initial forms)
  (make-org-event-helper name (initial policy) (forms policy)))

(def table-formula-event-helpers
  (list
   (policy-helper 'table-formula-assignments assignment-sequence
                  source-delimited-fragment-initial
                  source-delimited-fragment-forms)
   (make-org-event-helper
    'table-formula-assignment '()
    '((start-node OrgTableFormulaAssignment)
      (call-source-helper table-formula-equals start end)
      (finish-node)))
   (policy-helper 'table-formula-equals assignment-equals
                  source-first-split-initial source-first-split-forms)
   (make-org-event-helper
    'table-formula-lhs '()
    '((start-node OrgTableFormulaLhs)
      (call-source-helper table-formula-references start end)
      (finish-node)))
   (policy-helper 'table-formula-rhs-flags rhs-flags
                  source-first-split-initial source-first-split-forms)
   (make-org-event-helper
    'table-formula-rhs '()
    '((start-node OrgTableFormulaRhs)
      (call-source-helper table-formula-references start end)
      (finish-node)))
   (policy-helper 'table-formula-flags flag-sequence
                  source-delimited-fragment-initial
                  source-delimited-fragment-forms)
   (make-org-event-helper
    'table-formula-flag '()
    '((token FormulaFlag start end)))
   (policy-helper 'table-formula-references references
                  source-reference-scan-initial
                  source-reference-scan-forms)))
