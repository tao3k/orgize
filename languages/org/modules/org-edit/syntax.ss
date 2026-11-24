;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; Hygienic declarative Org structural edit DSL expanding strictly to POO values.

(import (only-in "ops.ss"
                 org-edit-set-todo org-edit-add-tag org-edit-remove-tag
                 org-edit-set-planning org-edit-remove-planning
                 org-edit-upsert-properties org-edit-delete-properties
                 org-edit-set-checkbox org-edit-upsert-babel-results
                 org-edit-upsert-section org-edit-append-section))
(export org-edit
        set-todo add-tag remove-tag
        set-planning remove-planning
        upsert-properties delete-properties
        set-checkbox upsert-babel-results
        upsert-section append-section)

(defsyntax (org-edit stx)
  (syntax-case stx ()
    ((_ clause ...)
     (syntax (list clause ...)))))

(defsyntax (set-todo stx)
  (syntax-case stx ()
    ((_ target kw) (syntax (org-edit-set-todo target kw)))))

(defsyntax (add-tag stx)
  (syntax-case stx ()
    ((_ target tag) (syntax (org-edit-add-tag target tag)))))

(defsyntax (remove-tag stx)
  (syntax-case stx ()
    ((_ target tag) (syntax (org-edit-remove-tag target tag)))))

(defsyntax (set-planning stx)
  (syntax-case stx ()
    ((_ target key ts) (syntax (org-edit-set-planning target key ts)))))

(defsyntax (remove-planning stx)
  (syntax-case stx ()
    ((_ target key) (syntax (org-edit-remove-planning target key)))))

(defsyntax (upsert-properties stx)
  (syntax-case stx ()
    ((_ target pairs) (syntax (org-edit-upsert-properties target pairs)))
    ((_ target pairs preserve?) (syntax (org-edit-upsert-properties target pairs preserve?)))))

(defsyntax (delete-properties stx)
  (syntax-case stx ()
    ((_ target keys) (syntax (org-edit-delete-properties target keys)))))

(defsyntax (set-checkbox stx)
  (syntax-case stx ()
    ((_ target idx checked?) (syntax (org-edit-set-checkbox target idx checked?)))))

(defsyntax (upsert-babel-results stx)
  (syntax-case stx ()
    ((_ name body) (syntax (org-edit-upsert-babel-results name body)))
    ((_ name body thinking)
     (syntax (org-edit-upsert-babel-results name body thinking-text: thinking)))))

(defsyntax (upsert-section stx)
  (syntax-case stx ()
    ((_ target spec) (syntax (org-edit-upsert-section target spec)))
    ((_ target spec preamble)
     (syntax (org-edit-upsert-section target spec empty-preamble: preamble)))))

(defsyntax (append-section stx)
  (syntax-case stx ()
    ((_ spec) (syntax (org-edit-append-section spec)))
    ((_ spec preamble)
     (syntax (org-edit-append-section spec empty-preamble: preamble)))))
