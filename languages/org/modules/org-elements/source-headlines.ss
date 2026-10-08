;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later

;;; A bounded, source-derived headline Element projection. The Rowan event
;;; parser owns structure and byte spans; Org Elements owns TODO interpretation.
;;; This API accepts text, not a caller-supplied graph or a WorkTree claim.
(import (only-in :clan/poo/object .o .ref .slot? object?)
        (only-in :std/crypto/digest sha256)
        (only-in :std/encoding/hex hex-encode)
        (only-in :std/list/list filter)
        (only-in :std/string/misc string-trim)
        (only-in "../../rowan-event-runtime.ss" parse-org-rowan-events)
        (only-in "headline-properties.ss"
                 todo-directive? todo-state-from-directives))

(export org-source-headline-elements
        org-source-headline-elements?
        org-source-headline-parser-identity)

(def org-source-headline-parser-identity
  "orgize.org-v1.rowan-events+headline-properties.v1")

(def (source-slice bytes start end)
  (utf8->string (subu8vector bytes start end)))

(def (source-digest bytes)
  (string-append "sha256:" (hex-encode (sha256 bytes))))

(def (nearest-frame stack kind)
  (let loop ((rest stack))
    (cond ((null? rest) #f)
          ((eq? (vector-ref (car rest) 0) kind) (car rest))
          (else (loop (cdr rest))))))

(def (org-source-headline-elements source)
  (unless (string? source)
    (error "Org source headline projection requires text"))
  (let* ((bytes (string->utf8 source))
         (size (u8vector-length bytes)))
    (when (> size 1048576)
      (error "Org source headline projection exceeds one MiB"))
    (let ((events (parse-org-rowan-events source))
          (stack '()) (offset 0) (roots 0)
          (headlines '()) (keywords '()))
      (for-each
       (lambda (event)
         (unless (pair? event)
           (error "invalid Org parser event" event))
         (case (car event)
           ((start)
            (unless (and (= (length event) 2) (symbol? (cadr event)))
              (error "invalid Org parser start" event))
            (when (null? stack)
              (set! roots (+ roots 1))
              (unless (eq? (cadr event) 'OrgFile)
                (error "Org source projection requires an OrgFile root")))
            ;; kind, start, marker-end, key-start/end, value-start/end
            (set! stack
                  (cons (vector (cadr event) offset #f #f #f #f #f)
                        stack)))
           ((token)
            (unless (and (= (length event) 4)
                         (symbol? (cadr event))
                         (exact-integer? (caddr event))
                         (exact-integer? (cadddr event))
                         (pair? stack)
                         (= (caddr event) offset)
                         (<= offset (cadddr event) size))
              (error "Org parser tokens do not partition source" event))
            (let ((headline (nearest-frame stack 'OrgHeadline))
                  (keyword (nearest-frame stack 'OrgKeyword)))
              (when (and headline (eq? (cadr event) 'HeadlineLine))
                (vector-set! headline 2 (cadddr event)))
              (when keyword
                (case (cadr event)
                  ((KeywordKey)
                   (vector-set! keyword 3 (caddr event))
                   (vector-set! keyword 4 (cadddr event)))
                  ((KeywordValue)
                   (unless (vector-ref keyword 5)
                     (vector-set! keyword 5 (caddr event)))
                   (vector-set! keyword 6 (cadddr event))))))
            (set! offset (cadddr event)))
           ((finish)
            (unless (and (= (length event) 1) (pair? stack))
              (error "unbalanced Org parser finish" event))
            (let ((frame (car stack)))
              (set! stack (cdr stack))
              (case (vector-ref frame 0)
                ((OrgHeadline)
                 (unless (and (vector-ref frame 2)
                              (<= (vector-ref frame 1)
                                  (vector-ref frame 2) offset))
                   (error "Org headline lacks source marker span"))
                 ;; OrgInlinetask embeds the same header node but is a
                 ;; different Element kind. This narrow projection selects
                 ;; ordinary OrgSection headlines only.
                 (when (and (pair? stack)
                            (eq? (vector-ref (car stack) 0) 'OrgSection))
                   (when (>= (length headlines) 256)
                     (error "Org source headline projection exceeds 256 elements"))
                   (set! headlines
                         (cons (vector (vector-ref frame 1) offset
                                       (string-trim
                                        (source-slice bytes
                                                      (vector-ref frame 2)
                                                      offset)))
                               headlines))))
                ((OrgKeyword)
                 (when (and (vector-ref frame 3) (vector-ref frame 5))
                   (set! keywords
                         (cons (cons (source-slice bytes
                                                   (vector-ref frame 3)
                                                   (vector-ref frame 4))
                                     (source-slice bytes
                                                   (vector-ref frame 5)
                                                   (vector-ref frame 6)))
                               keywords)))))))
           (else (error "unknown Org parser event" event))))
       events)
      (unless (and (null? stack) (= roots 1) (= offset size))
        (error "Org parser events do not cover one source root"))
      (let* ((digest (source-digest bytes))
             (directives
              (map cdr
                   (filter (lambda (keyword)
                             (todo-directive? (car keyword)))
                           (reverse keywords))))
             (records
              (map (lambda (headline)
                     (let* ((start (vector-ref headline 0))
                            (end (vector-ref headline 1))
                            (title-value (vector-ref headline 2))
                            (state (todo-state-from-directives
                                    title-value directives '("TODO") '("DONE")))
                            (element-id
                             (string-append digest ":"
                                            (number->string start) ":"
                                            (number->string end))))
                       (.o kind: 'orgize.org-source-headline-element
                           label: 'OrgHeadline
                           identity: element-id
                           byte-start: start
                           byte-end: end
                           title: title-value
                           todo-type: state)))
                   (reverse headlines))))
        (.o kind: 'orgize.org-source-headline-elements
            parser-identity: org-source-headline-parser-identity
            source-sha256: digest
            source-size: size
            elements: records
            complete?: #t
            worktree-bound?: #f
            action-authority?: #f)))))

(def (org-source-headline-elements? value)
  (and (object? value) (.slot? value 'kind)
       (eq? (.ref value 'kind) 'orgize.org-source-headline-elements)))
