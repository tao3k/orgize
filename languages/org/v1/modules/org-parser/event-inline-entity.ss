;;; -*- Gerbil -*-
;;; Org Entity recognition is one bounded source-backed inline strategy.

(import (only-in "entity-names.ss" org-entity-names)
        (only-in "event-inline-primitives.ss"
                 link-index inline-next pattern-end pattern-at?))
(export entity-name-bytes entity-space-choices entity-events
        entity-finish-forms entity-name-scan-forms)

(def entity-name-bytes
  (map char->integer
       (string->list
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789")))

(def (entity-space-choices count)
  (let* ((spaces-start (pattern-end link-index "\\_"))
         (limit (pattern-end spaces-start (make-string count #\space)))
         (space-index '(line-index entity-space-byte-index)))
    `((set-uint inline-entity-mode (uint 3))
      (set-uint inline-entity-post-end (offset ,spaces-start))
      (for-line-bytes entity-space-byte-index ,spaces-start ,limit
        ((if (and (uint-equal? (state inline-entity-mode) (uint 3))
                  (line-byte-equal? ,space-index 32))
             ((set-uint inline-entity-post-end
                        (offset (line-step ,space-index))))
             ((set-uint inline-entity-mode (uint 0))))))
      (if (offset-less? ,spaces-start (state-offset inline-entity-post-end))
          ((set-uint inline-entity-mode (uint 2))
           (set-uint inline-entity-open-at (offset ,link-index))
           (set-uint inline-entity-name-start (offset ,inline-next))
           (set-uint inline-entity-name-end (offset ,spaces-start)))
          ((set-uint inline-entity-mode (uint 0)))))))

(def (entity-events)
  `((token TextLine (state-offset inline-cursor)
           (state-offset inline-entity-open-at))
    (start-node OrgEntity)
    (token EntityDelimiter (state-offset inline-entity-open-at)
           (state-offset inline-entity-name-start))
    (token EntityName (state-offset inline-entity-name-start)
           (state-offset inline-entity-name-end))
    (if (offset-less? (state-offset inline-entity-name-end)
                      (state-offset inline-entity-post-end))
        ((token EntityPost (state-offset inline-entity-name-end)
                (state-offset inline-entity-post-end))) ())
    (finish-node)
    (set-uint inline-cursor
              (offset (state-offset inline-entity-post-end)))))

(def (entity-finish-forms (reset-mode? #t))
  `((if (line-bytes-in-set? (state-offset inline-entity-name-start)
                            (state-offset inline-entity-name-end)
                            ,org-entity-names)
        ((set-uint inline-entity-post-end
                   (offset (state-offset inline-entity-name-end)))
         (if ,(pattern-at? '(state-offset inline-entity-name-end) "{}")
             ((set-uint inline-entity-post-end
                        (offset ,(pattern-end
                                  '(state-offset inline-entity-name-end) "{}"))))
             ())
         ,@(entity-events))
        ())
    ,@(if reset-mode? '((set-uint inline-entity-mode (uint 0))) '())))

(def (entity-name-scan-forms)
  `((if (line-bytes-any-in? ,link-index ,inline-next ,entity-name-bytes)
        ((set-uint inline-entity-name-end (offset ,inline-next)))
        ,(entity-finish-forms))))
