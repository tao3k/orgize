;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; Typed Org Element read helpers built directly over parser-projected graphs.
;;; Eliminates downstream record filtering, property parsing, and checkbox hacking.

(import (only-in :clan/poo/object .ref)
        (only-in :std/list/list every filter find)
        (only-in :std/string/misc string-trim)
        (only-in :gerbil-parser/graph-support/edit utf8-byte-slice)
        (only-in "values.ss" org-element-graph-records)
        (only-in "query.ss" org-element-property))
(export org-source-slice
        org-graph-records-ordered
        org-graph-headlines
        org-headline-level
        org-headline-property
        org-headline-tags
        org-headline-has-tag?
        org-headline-todo-state
        org-find-headline
        org-headline-property-drawer
        org-drawer-node-properties
        org-headline-planning-records
        org-checkbox-state
        org-element-enclosing-headline
        org-headline-checkbox-items
        org-headline-checklist-stats
        org-graph-checklist-stats)

(def (org-source-slice source-or-bytes start end)
  (let ((bytes (if (u8vector? source-or-bytes)
                 source-or-bytes
                 (string->utf8 source-or-bytes))))
    (utf8-byte-slice bytes start end)))

(def (org-graph-records-ordered graph)
  (list-sort (lambda (a b) (< (.ref a 'start) (.ref b 'start)))
             (org-element-graph-records graph)))

(def (org-graph-headlines graph)
  (filter (lambda (r) (member (.ref r 'kind) '("headline" "inlinetask")))
          (org-graph-records-ordered graph)))

(def (org-headline-level ctx record)
  (let ((lvl (org-element-property ctx record "level")))
    (if (and (exact-integer? lvl) (> lvl 0))
      lvl
      1)))

(def (org-headline-property ctx record key)
  (let ((val (org-element-property ctx record (string-append "property:" key))))
    (and (string? val)
         (> (string-length (string-trim val)) 0)
         (string-trim val))))

(def (org-headline-tags ctx record)
  (let ((raw (org-element-property ctx record "tags")))
    (cond
     ((list? raw) raw)
     ((vector? raw) (vector->list raw))
     (else '()))))

(def (org-headline-has-tag? ctx record tag)
  (and (string? tag)
       (find (lambda (t) (and (string? t) (string-ci=? t tag)))
             (org-headline-tags ctx record))
       #t))

(def (org-headline-todo-state ctx record)
  (let ((kw (org-element-property ctx record "todo-keyword"))
        (tt (org-element-property ctx record "todo-type")))
    (cond
     ((string? kw)
      (let ((u (string-upcase (string-trim kw))))
        (cond
         ((member u '("DONE" "COMPLETED" "VERIFIED")) 'done)
         ((member u '("FAILED" "ERROR")) 'failed)
         ((member u '("CANCELLED" "CANCELED" "ABORTED")) 'cancelled)
         ((member u '("BLOCKED" "HOLD" "WAITING")) 'blocked)
         ((member u '("IN_PROGRESS" "NEXT" "STARTED")) 'in-progress)
         ((member u '("REVIEW")) 'review)
         ((equal? tt "done") 'done)
         (else 'todo))))
     ((equal? tt "done") 'done)
     (else 'todo))))

;; Target may be 'first, 'last, a predicate (lambda (ctx record) ...), or an
;; :ID: / :CUSTOM_ID: string.
(def (org-find-headline ctx graph target)
  (let ((headlines (org-graph-headlines graph)))
    (cond
     ((null? headlines) #f)
     ((eq? target 'first) (car headlines))
     ((eq? target 'last) (car (reverse headlines)))
     ((procedure? target)
      (find (lambda (h) (target ctx h)) headlines))
     ((string? target)
      (find (lambda (h)
              (or (equal? (org-headline-property ctx h "ID") target)
                  (equal? (org-headline-property ctx h "CUSTOM_ID") target)))
            headlines))
     (else (error "invalid Org headline target" target)))))

(def (org-headline-property-drawer graph headline-record)
  (let ((hid (.ref headline-record 'id)))
    (find (lambda (r)
            (and (equal? (.ref r 'kind) "property-drawer")
                 (equal? (.ref r 'parent) hid)))
          (org-graph-records-ordered graph))))

(def (org-drawer-node-properties graph drawer-record)
  (let ((did (.ref drawer-record 'id)))
    (filter (lambda (r)
              (and (equal? (.ref r 'kind) "node-property")
                   (equal? (.ref r 'parent) did)))
            (org-graph-records-ordered graph))))

(def (org-headline-planning-records graph headline-record)
  (let ((hid (.ref headline-record 'id)))
    (filter (lambda (r)
              (and (equal? (.ref r 'kind) "planning")
                   (equal? (.ref r 'parent) hid)))
            (org-graph-records-ordered graph))))

(def (org-checkbox-state raw)
  (cond
   ((or (not raw) (equal? raw "")) 'none)
   ((or (equal? raw "X") (equal? raw "x") (equal? raw "[X]")
        (equal? raw "on") (eq? raw 'on) (equal? raw "checked"))
    'checked)
   ((or (equal? raw "-") (equal? raw "[-]") (equal? raw "partial"))
    'partial)
   ((or (equal? raw " ") (equal? raw "[ ]") (equal? raw "off")
        (eq? raw 'off) (equal? raw "unchecked"))
    'unchecked)
   (else 'other)))

(def (org-element-enclosing-headline record by-id)
  (let loop ((pid (.ref record 'parent)))
    (and pid
         (let ((p (hash-get by-id pid)))
           (and p
                (if (member (.ref p 'kind) '("headline" "inlinetask"))
                  p
                  (loop (.ref p 'parent))))))))

(def (org-headline-checkbox-items ctx graph headline-record (direct-only? #f))
  (let* ((records (org-graph-records-ordered graph))
         (hid (.ref headline-record 'id))
         (h-start (.ref headline-record 'start))
         (h-end (.ref headline-record 'end))
         (by-id (and direct-only?
                     (let ((t (make-hash-table)))
                       (for-each (lambda (r) (hash-put! t (.ref r 'id) r)) records)
                       t))))
    (filter (lambda (r)
              (and (equal? (.ref r 'kind) "item")
                   (not (eq? (org-checkbox-state (org-element-property ctx r "checkbox")) 'none))
                   (if direct-only?
                     (let ((eh (org-element-enclosing-headline r by-id)))
                       (and eh (equal? (.ref eh 'id) hid)))
                     (and (>= (.ref r 'start) h-start)
                          (<= (.ref r 'end) h-end)))))
            records)))

(def (org-headline-checklist-stats ctx graph headline-record (direct-only? #f))
  (let* ((boxes (org-headline-checkbox-items ctx graph headline-record direct-only?))
         (done (length (filter (lambda (it)
                                 (eq? (org-checkbox-state
                                       (org-element-property ctx it "checkbox"))
                                      'checked))
                               boxes)))
         (total (length boxes)))
    (values done total (= done total))))

(def (org-graph-checklist-stats ctx graph)
  (let* ((records (org-graph-records-ordered graph))
         (boxes (filter (lambda (r)
                          (and (equal? (.ref r 'kind) "item")
                               (not (eq? (org-checkbox-state
                                          (org-element-property ctx r "checkbox"))
                                         'none))))
                        records))
         (done (length (filter (lambda (it)
                                 (eq? (org-checkbox-state
                                       (org-element-property ctx it "checkbox"))
                                      'checked))
                               boxes)))
         (total (length boxes)))
    (values done total (= done total))))
