;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; Native event projection driven by the maintained Org graph declaration.
(import (only-in :clan/poo/object .o .ref)
        (only-in :std/list/list find)
        (only-in :std/string/misc string-trim)
        (only-in :orgize/languages/org/modules/org-parser/event-runtime parse-org-native-events)
        (only-in "graph-shape.ss" org-graph-shape)
        (only-in :gerbil-parser/src/graph/graph-projection-objects graph-node-syntax-kind graph-node-label graph-node-fields
                 graph-field-token graph-field-name graph-field-mode)
        (only-in "values.ss" make-org-element-graph-view)
        (only-in "headline-properties.ss" org-element-with-headline-properties))
(export org-source-element-graph org-source-element-field)

(def (org-source-element-field record name)
  (hash-get (.ref record 'fields) name))

(def (org-source-element-graph source)
  (unless (string? source) (error "Org graph requires source text"))
  (let* ((bytes (string->utf8 source)) (size (u8vector-length bytes))
         (rules (make-hash-table)) (stack '()) (records '())
         (offset 0) (roots 0) (next-id 0))
    (when (> size 1048576) (error "Org graph exceeds one MiB"))
    (for-each (lambda (rule) (hash-put! rules (graph-node-syntax-kind rule) rule))
              org-graph-shape)
    (def (slice start end) (utf8->string (subu8vector bytes start end)))
    ;; frame: native kind, rule, ID, mapped parent, start, mutable fields.
    (def (put frame key text mode)
      (let* ((fields (vector-ref frame 5)) (previous (hash-get fields key)))
        (hash-put! fields key
          (case mode
            ((each each-node-text) (append (or previous '()) (list text)))
            ((node-text) text)
            (else (string-append (or previous "") text))))))
    (def (project frame kind text node?)
      (let (rule (vector-ref frame 1))
        (when rule
          (for-each
           (lambda (field)
             (let (mode (graph-field-mode field))
               (when (and (eq? kind (graph-field-token field))
                          (if node? (memq mode '(node-text each-node-text))
                              (not (memq mode '(node-text each-node-text)))))
                 (put frame (graph-field-name field) text mode))))
           (graph-node-fields rule)))))
    (for-each
     (lambda (event)
       (unless (pair? event) (error "invalid Org event" event))
       (case (car event)
         ((start)
          (unless (and (= (length event) 2) (symbol? (cadr event)))
            (error "invalid Org start event" event))
          (when (null? stack)
            (set! roots (+ roots 1))
            (unless (eq? (cadr event) 'OrgFile) (error "OrgFile root required")))
          (let* ((rule (hash-get rules (cadr event)))
                 (parent (find (lambda (frame) (vector-ref frame 1)) stack))
                 (id (and rule next-id)))
            (when rule (set! next-id (+ next-id 1)))
            (when (> next-id 4096) (error "Org graph exceeds 4096 Elements"))
            (set! stack
                  (cons (vector (cadr event) rule id
                                (and parent (vector-ref parent 2)) offset
                                (make-hash-table)) stack))))
         ((token)
          (unless (and (= (length event) 4) (pair? stack)
                       (exact-integer? (caddr event)) (exact-integer? (cadddr event))
                       (= offset (caddr event)) (<= offset (cadddr event) size))
            (error "Org tokens do not partition source" event))
          (let (text (slice offset (cadddr event)))
            (let (owner (find (lambda (frame) (vector-ref frame 1)) stack))
              (when owner (project owner (cadr event) text #f))))
          (set! offset (cadddr event)))
         ((finish)
          (unless (and (= (length event) 1) (pair? stack))
            (error "unbalanced Org finish event"))
          (let* ((frame (car stack)) (kind (vector-ref frame 0))
                 (rule (vector-ref frame 1)) (id-value (vector-ref frame 2))
                 (parent-value (vector-ref frame 3)) (start-value (vector-ref frame 4))
                 (fields-value (vector-ref frame 5)) (end-value offset)
                 (text (slice start-value offset)))
            ;; Node-text fields belong to the enclosing mapped owner, including
            ;; a mapped node's own declaration, not an independent text parser.
            (let (owner
                  (find (lambda (candidate)
                          (let (declaration (vector-ref candidate 1))
                            (and declaration
                                 (find (lambda (field)
                                         (and (eq? kind (graph-field-token field))
                                              (memq (graph-field-mode field)
                                                    '(node-text each-node-text))))
                                       (graph-node-fields declaration))))) stack))
              (when owner (project owner kind text #t)))
            (set! stack (cdr stack))
            (when (eq? kind 'OrgNodeProperty)
              (let* ((owner (find (lambda (f)
                                   (memq (vector-ref f 0) '(OrgSection OrgFile))) stack))
                     (drawer (find (lambda (f)
                                    (eq? (vector-ref f 0) 'OrgPropertyDrawer)) stack))
                     (key (hash-get fields-value "key"))
                     (value (hash-get fields-value "value")))
                (when (and owner drawer key value
                           (equal? (vector-ref drawer 3) (vector-ref owner 2)))
                  (let* ((owner-fields (vector-ref owner 5))
                         (name (string-append "property:" key)))
                    (when (hash-get owner-fields name)
                      (error "duplicate Org property" key))
                    (hash-put! owner-fields name (string-trim value))))))
            (when rule
              (set! records
                    (cons (.o id: id-value parent: parent-value
                              kind: (graph-node-label rule)
                              fields: fields-value start: start-value end: end-value)
                          records)))))
         (else (error "unknown Org event" event))))
     (parse-org-native-events source))
    (unless (and (= roots 1) (null? stack) (= size offset))
      (error "incomplete Org source coverage"))
    (org-element-with-headline-properties (make-org-element-graph-view
     (reverse records)
     (lambda (record) (.ref record 'id))
     (lambda (record) (.ref record 'parent))
     (lambda (record) (.ref record 'kind))
     org-source-element-field))))
