;;; -*- Gerbil -*-
;;; Source-backed Org citation Objects; the Scheme state machine AOT-lowers
;;; alongside the other inline event strategies.

(import (only-in "event-inline-primitives.ss"
                 link-index inline-next pattern-end pattern-at?))
(export citation-event-initial citation-open-forms citation-scan-forms)

(def citation-event-initial
  '((inline-citation-mode 0)
    (inline-citation-open-at 0) (inline-citation-body-start 0)
    (inline-citation-scan-start 0)
    (inline-citation-style-part #f)
    (inline-citation-has-key #f) (inline-citation-await-key #f)
    (inline-citation-escaped #f)
    (inline-citation-opens 0) (inline-citation-closes 0)))

(def (citation-start-forms mode marker)
  `((set-uint inline-citation-mode (uint ,mode))
    (set-uint inline-citation-open-at (offset ,link-index))
    (set-uint inline-citation-scan-start
              (offset ,(pattern-end link-index marker)))
    (set-bool inline-citation-style-part (bool #f))
    (set-bool inline-citation-has-key (bool #f))
    (set-bool inline-citation-await-key (bool #f))
    (set-bool inline-citation-escaped (bool #f))
    (set-uint inline-citation-opens (uint 0))
    (set-uint inline-citation-closes (uint 0))))

(def (citation-open-forms)
  `((if ,(pattern-at? link-index "[cite:")
        (,@(citation-start-forms 2 "[cite:")
         (set-uint inline-citation-body-start
                   (offset ,(pattern-end link-index "[cite:"))))
        ((if ,(pattern-at? link-index "[cite/")
             ,(citation-start-forms 1 "[cite/") ())))))

(def (citation-events)
  `((token TextLine (state-offset inline-cursor)
           (state-offset inline-citation-open-at))
    (start-node OrgCitation)
    (token CitationDelimiter (state-offset inline-citation-open-at)
           (state-offset inline-citation-body-start))
    (token CitationBody (state-offset inline-citation-body-start) ,link-index)
    (token CitationDelimiter ,link-index ,inline-next)
    (finish-node)
    (set-uint inline-cursor (offset ,inline-next))))

(def (citation-style-scan-forms)
  `((if (line-byte-equal? ,link-index 58)
        ((if (state inline-citation-style-part)
             ((set-uint inline-citation-mode (uint 2))
              (set-uint inline-citation-body-start (offset ,inline-next)))
             ((set-uint inline-citation-mode (uint 0)))))
        ((if (line-byte-equal? ,link-index 47)
             ((if (state inline-citation-style-part)
                  ((set-bool inline-citation-style-part (bool #f)))
                  ((set-uint inline-citation-mode (uint 0)))))
             ((if (line-bytes-any-in? ,link-index ,inline-next
                                       (9 10 13 32 91 93))
                  ((set-uint inline-citation-mode (uint 0)))
                  ((set-bool inline-citation-style-part (bool #t))))))))))

(def (citation-key-scan-forms)
  `((if (state inline-citation-await-key)
        ((if (not (line-bytes-any-in? ,link-index ,inline-next
                                     (9 10 13 32 93)))
             ((set-bool inline-citation-has-key (bool #t))) ())
         (set-bool inline-citation-await-key (bool #f))) ())
    (if (line-byte-equal? ,link-index 64)
        ((set-bool inline-citation-await-key (bool #t))) ())))

(def (citation-body-scan-forms)
  `(,@(citation-key-scan-forms)
    (if (state inline-citation-escaped)
        ((set-bool inline-citation-escaped (bool #f)))
        ((if (line-byte-equal? ,link-index 92)
             ((set-bool inline-citation-escaped (bool #t)))
             ((if (line-byte-equal? ,link-index 91)
                  ((set-uint inline-citation-opens
                             (uint-add (state inline-citation-opens) (uint 1))))
                  ((if (line-byte-equal? ,link-index 93)
                       ((if (uint-equal? (state inline-citation-opens)
                                         (state inline-citation-closes))
                            ((if (state inline-citation-has-key)
                                 ,(citation-events) ())
                             (set-uint inline-citation-mode (uint 0)))
                            ((set-uint inline-citation-closes
                                       (uint-add (state inline-citation-closes)
                                                 (uint 1)))))) ())))))))))

(def (citation-scan-forms)
  `((if (offset-less? ,link-index (state-offset inline-citation-scan-start))
        ()
        ((if (uint-equal? (state inline-citation-mode) (uint 1))
             ,(citation-style-scan-forms)
             ,(citation-body-scan-forms))))))
