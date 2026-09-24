;;; -*- Gerbil -*-
;;; POO-declared inline links are folded as source-backed text-line events.

(import (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure-text text-line-inline-link
                 inline-link-opening inline-link-separator inline-link-closing
                 inline-link-node inline-link-target-token
                 inline-link-description-token inline-link-trivia-token)
        (only-in "../../parser.ss" org-v1-line-structure))
(export event-inline-initial event-text-line-forms)

(def link-rule
  (text-line-inline-link (line-structure-text org-v1-line-structure)))
(def link-open (inline-link-opening link-rule))
(def link-separator (inline-link-separator link-rule))
(def link-close (inline-link-closing link-rule))
(def link-index '(line-index inline-byte-index))

(def (pattern-end from pattern)
  (let loop ((offset from) (remaining (string-length pattern)))
    (if (= remaining 0) offset
      (loop `(line-step ,offset) (- remaining 1)))))

(def (pattern-at? from pattern)
  (let ((bytes (string->utf8 pattern)))
    (unless (> (u8vector-length bytes) 0)
      (error "inline link marker must not be empty" pattern))
    (let loop ((offset from) (index 0) (predicate #f))
      (if (= index (u8vector-length bytes)) predicate
        (let (check `(line-byte-equal? ,offset ,(u8vector-ref bytes index)))
          (loop `(line-step ,offset) (+ index 1)
                (if predicate `(and ,predicate ,check) check)))))))

(def (link-valid? target-end)
  `(offset-less? (state-offset inline-target-start) ,target-end))

(def (link-events)
  (let ((trivia (inline-link-trivia-token link-rule))
        (target (inline-link-target-token link-rule))
        (description (inline-link-description-token link-rule))
        (separator-end (pattern-end '(state-offset inline-separator-at)
                                    link-separator))
        (close-end (pattern-end link-index link-close)))
    `((token TextLine (state-offset inline-cursor)
             (state-offset inline-open-at))
      (start-node ,(inline-link-node link-rule))
      (token ,trivia (state-offset inline-open-at)
             (state-offset inline-target-start))
      (if (state inline-has-separator)
          ((token ,target (state-offset inline-target-start)
                  (state-offset inline-separator-at))
           (token ,trivia (state-offset inline-separator-at) ,separator-end)
           (token ,description ,separator-end ,link-index))
          ((token ,target (state-offset inline-target-start) ,link-index)))
      (token ,trivia ,link-index ,close-end)
      (finish-node)
      (set-uint inline-cursor (offset ,close-end))
      (set-bool inline-open (bool #f)))))

(def (link-close-forms)
  `((if (state inline-has-separator)
        ((if ,(link-valid? '(state-offset inline-separator-at))
             ,(link-events)
             ((set-bool inline-failed (bool #t)))))
        ((if ,(link-valid? link-index)
             ,(link-events)
             ((set-bool inline-failed (bool #t))))))))

(def (link-scan-forms)
  `((if (and (not (state inline-failed))
             (not (state inline-open)))
        ((if ,(pattern-at? link-index link-open)
             ((set-bool inline-open (bool #t))
              (set-bool inline-has-separator (bool #f))
              (set-uint inline-open-at (offset ,link-index))
              (set-uint inline-target-start
                        (offset ,(pattern-end link-index link-open)))) ()))
        ((if (and (state inline-open)
                  ,(pattern-at? link-index link-close))
             ,(link-close-forms)
             ((if (and (state inline-open)
                       (not (state inline-has-separator))
                       ,(pattern-at? link-index link-separator))
                  ((set-bool inline-has-separator (bool #t))
                   (set-uint inline-separator-at (offset ,link-index))) ())))))))

(def (event-text-line-forms from)
  `((start-node OrgTextLine)
    (if (offset-less? (state-offset inline-cursor) ,from)
        ((set-uint inline-cursor (offset ,from))) ())
    (for-line-bytes inline-byte-index ,from end ,(link-scan-forms))
    (token TextLine (state-offset inline-cursor) end)
    (set-uint inline-cursor (offset end))
    (set-bool inline-open (bool #f))
    (set-bool inline-failed (bool #f))
    (finish-node)))

(def event-inline-initial
  '((inline-cursor 0) (inline-open #f) (inline-has-separator #f)
    (inline-failed #f) (inline-open-at 0) (inline-target-start 0)
    (inline-separator-at 0)))
