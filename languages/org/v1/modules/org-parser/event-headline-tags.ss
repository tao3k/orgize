;;; -*- Gerbil -*-
;;; Org headline tag suffixes are Scheme-owned, source-backed event tokens.

(export event-headline-tags-initial event-headline-title-forms)

(def tag-index '(line-index headline-tag-byte-index))
(def tag-next `(line-step ,tag-index))
(def tag-space? `(line-bytes-any-in? ,tag-index ,tag-next (9 32)))
(def tag-name-bytes
  (append
   (map char->integer
        (string->list
         "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_@#%"))
   (let loop ((byte 128))
     (if (= byte 256) '() (cons byte (loop (+ byte 1)))))))
(def tag-name-byte?
  `(line-bytes-any-in? ,tag-index ,tag-next ,tag-name-bytes))

(def (headline-tag-scan-step)
  `((if ,tag-space?
        ((if (line-byte-equal? ,tag-next 58)
             ((set-uint headline-tag-start (offset ,tag-next))
              (set-uint headline-tag-mode (uint 1))
              (set-uint headline-tag-count (uint 0)))
             ((set-uint headline-tag-mode (uint 0)))))
        ((if (uint-equal? (state headline-tag-mode) (uint 1))
             ((if (line-byte-equal? ,tag-index 58)
                  ((set-uint headline-tag-mode (uint 3)))
                  ((set-uint headline-tag-mode (uint 0)))))
             ((if (uint-equal? (state headline-tag-mode) (uint 2))
                  ((if (line-byte-equal? ,tag-index 58)
                       ((set-uint headline-tag-count
                                  (uint-add (state headline-tag-count) (uint 1)))
                        (set-uint headline-tag-mode (uint 3)))
                       ((if ,tag-name-byte? ()
                            ((set-uint headline-tag-mode (uint 0)))))))
                  ((if (uint-equal? (state headline-tag-mode) (uint 3))
                       ((if ,tag-name-byte?
                            ((set-uint headline-tag-mode (uint 2)))
                            ((set-uint headline-tag-mode (uint 0)))))
                       ())))))))))

(def (headline-tag-token-step)
  `((if (line-byte-equal? ,tag-index 58)
        ((token HeadlineTagValue
                (state-offset headline-tag-start) ,tag-index)
         (token HeadlineTagTrivia ,tag-index ,tag-next)
         (set-uint headline-tag-start (offset ,tag-next))) ())))

(def (event-headline-title-forms from until)
  `((if (line-bytes-any-in? ,from ,until (58))
        ((if (uint-positive? (state headline-tag-cursor))
             ((set-uint headline-tag-mode (uint 0))
              (set-uint headline-tag-count (uint 0))) ())
         (set-uint headline-tag-cursor (offset ,from))
         (if (line-byte-equal? ,from 58)
             ((set-uint headline-tag-start (offset ,from))
              (set-uint headline-tag-mode (uint 1))) ())
         (for-line-bytes headline-tag-byte-index ,from ,until
           ,(headline-tag-scan-step))
         (if (and (uint-equal? (state headline-tag-mode) (uint 3))
                  (uint-positive? (state headline-tag-count)))
             ((token HeadlineTitle ,from
                     (state-offset headline-tag-start))
              (token HeadlineTagTrivia
                     (state-offset headline-tag-start)
                     (line-step (state-offset headline-tag-start)))
              (set-uint headline-tag-start
                        (offset (line-step (state-offset headline-tag-start))))
              (for-line-bytes headline-tag-byte-index
                (state-offset headline-tag-start) ,until
                ,(headline-tag-token-step)))
             ((token HeadlineTitle ,from ,until))))
        ((token HeadlineTitle ,from ,until)))))

(def event-headline-tags-initial
  '((headline-tag-mode 0) (headline-tag-cursor 0)
    (headline-tag-start 0) (headline-tag-count 0)))
