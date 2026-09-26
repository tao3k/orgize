;;; -*- Gerbil -*-
;;; Citation-reference Objects are parsed only after their enclosing citation
;;; has passed its Org-owned validity scan.  The bounded helper shares no
;;; mutable state with its caller and AOT-compiles to one Rust function.

(import (only-in "objects.ss" make-org-event-helper))
(export citation-reference-helper)

(def citation-ref-index '(line-index citation-ref-byte-index))
(def citation-ref-next `(line-step ,citation-ref-index))
(def citation-reference-key-bytes
  (append
   (map char->integer
        (string->list
         "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-.:?!`'/*@+|(){}<>&^$#%~"))
   (iota 128 128)))

(def (citation-reference-segment-events until)
  `((if (state citation-ref-has-key)
        ((start-node OrgCitationReference)
         (token CitationReferencePrefix
                (state-offset citation-ref-segment-start)
                (state-offset citation-ref-key-at))
         (token CitationReferenceMarker
                (state-offset citation-ref-key-at)
                (state-offset citation-ref-key-start))
         (token CitationReferenceKey
                (state-offset citation-ref-key-start)
                (state-offset citation-ref-key-end))
         (token CitationReferenceSuffix
                (state-offset citation-ref-key-end) ,until)
         (finish-node))
        ((token CitationText
                (state-offset citation-ref-segment-start) ,until)))))

(def (citation-reference-separator-forms)
  `(,@(citation-reference-segment-events citation-ref-index)
    (token CitationSeparator ,citation-ref-index ,citation-ref-next)
    (set-uint citation-ref-segment-start (offset ,citation-ref-next))
    (set-bool citation-ref-has-key (bool #f))
    (set-bool citation-ref-key-scanning (bool #f))
    (set-uint citation-ref-opens (uint 0))
    (set-uint citation-ref-closes (uint 0))))

(def (citation-reference-scan-forms)
  `((if (state citation-ref-key-scanning)
        ((if (line-bytes-any-in? ,citation-ref-index ,citation-ref-next
                                 ,citation-reference-key-bytes)
             ((set-uint citation-ref-key-end (offset ,citation-ref-next)))
             ((set-bool citation-ref-key-scanning (bool #f))))) ())
    (if (state citation-ref-escaped)
        ((set-bool citation-ref-escaped (bool #f)))
        ((if (line-byte-equal? ,citation-ref-index 92)
             ((set-bool citation-ref-escaped (bool #t)))
             ((if (line-byte-equal? ,citation-ref-index 91)
                  ((set-uint citation-ref-opens
                             (uint-add (state citation-ref-opens) (uint 1))))
                  ((if (and (line-byte-equal? ,citation-ref-index 93)
                            (uint-greater? (state citation-ref-opens)
                                           (state citation-ref-closes)))
                       ((set-uint citation-ref-closes
                                  (uint-add (state citation-ref-closes)
                                            (uint 1)))) ())))
              (if (and (not (state citation-ref-has-key))
                       (uint-equal? (state citation-ref-opens)
                                    (state citation-ref-closes))
                       (line-byte-equal? ,citation-ref-index 64)
                       (line-bytes-any-in? ,citation-ref-next
                                           (line-step ,citation-ref-next)
                                           ,citation-reference-key-bytes))
                  ((set-bool citation-ref-has-key (bool #t))
                   (set-bool citation-ref-key-scanning (bool #t))
                   (set-uint citation-ref-key-at
                             (offset ,citation-ref-index))
                   (set-uint citation-ref-key-start
                             (offset ,citation-ref-next))
                   (set-uint citation-ref-key-end
                             (offset ,citation-ref-next))) ())
              (if (and (line-byte-equal? ,citation-ref-index 59)
                       (uint-equal? (state citation-ref-opens)
                                    (state citation-ref-closes)))
                  ,(citation-reference-separator-forms) ())))))))

(def citation-reference-helper
  (make-org-event-helper
   'citation-references
   '((citation-ref-segment-start 0)
     (citation-ref-key-at 0) (citation-ref-key-start 0)
     (citation-ref-key-end 0)
     (citation-ref-has-key #f) (citation-ref-key-scanning #f)
     (citation-ref-escaped #f)
     (citation-ref-opens 0) (citation-ref-closes 0))
   `((set-uint citation-ref-segment-start
               (uint-add (state citation-ref-segment-start) (offset start)))
     (for-line-bytes citation-ref-byte-index start end
                     ,(citation-reference-scan-forms))
     ,@(citation-reference-segment-events 'end))))
