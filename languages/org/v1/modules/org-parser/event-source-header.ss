;;; -*- Gerbil -*-
;;; Source-block header arguments are Scheme-owned, source-backed event tokens.

(export event-source-header-initial event-source-header-forms)

(def header-index '(line-index source-header-byte-index))
(def header-next `(line-step ,header-index))
(def header-space? `(line-bytes-any-in? ,header-index ,header-next (9 32)))
(def header-key-bytes
  (map char->integer
       (string->list
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-")))

(def (event-source-header-forms from)
  `((if (uint-positive? (state source-header-cursor))
        ((set-uint source-header-mode (uint 0))) ())
    (set-uint source-header-cursor (offset ,from))
    (for-line-bytes source-header-byte-index ,from (line-content-end)
      ,(source-header-step-forms))
    (if (uint-equal? (state source-header-mode) (uint 1))
        ((token SourceHeaderKey (state-offset source-header-key-start)
                (line-content-end))
         (set-uint source-header-cursor (offset (line-content-end)))) ())
    (if (or (uint-equal? (state source-header-mode) (uint 3))
            (uint-equal? (state source-header-mode) (uint 4))
            (uint-equal? (state source-header-mode) (uint 5)))
        ((token SourceHeaderValue (state-offset source-header-value-start)
                (line-content-end))
         (set-uint source-header-cursor (offset (line-content-end)))) ())
    (token SourceHeaderTrivia (state-offset source-header-cursor) end)))

(def (source-header-step-forms)
  `((if (uint-equal? (state source-header-mode) (uint 0))
        ,(source-header-trivia-forms)
        ((if (uint-equal? (state source-header-mode) (uint 1))
             ,(source-header-key-forms)
             ((if (uint-equal? (state source-header-mode) (uint 2))
                  ,(source-header-value-start-forms)
                  ((if (uint-equal? (state source-header-mode) (uint 3))
                       ,(source-header-bare-value-forms)
                       ((if (uint-equal? (state source-header-mode) (uint 6))
                            ,(source-header-unrecognized-forms)
                            ,(source-header-quoted-value-forms))))))))))))

(def (source-header-trivia-forms)
  `((if ,header-space?
        ()
        ((if (and (line-byte-equal? ,header-index 58)
                  (line-bytes-any-in? ,header-next
                                      (line-step ,header-next)
                                      ,header-key-bytes))
             ((token SourceHeaderTrivia (state-offset source-header-cursor)
                     ,header-next)
              (set-uint source-header-cursor (offset ,header-next))
              (set-uint source-header-key-start (offset ,header-next))
              (set-uint source-header-mode (uint 1)))
             ((set-uint source-header-mode (uint 6))))))))

(def (source-header-unrecognized-forms)
  `((if ,header-space?
        ((set-uint source-header-mode (uint 0))) ())))

(def (source-header-key-forms)
  `((if ,header-space?
        ((token SourceHeaderKey (state-offset source-header-key-start)
                ,header-index)
         (set-uint source-header-cursor (offset ,header-index))
         (set-uint source-header-mode (uint 2)))
        ((if (line-bytes-any-in? ,header-index ,header-next
                                 ,header-key-bytes)
             ()
             ((set-uint source-header-mode (uint 6))))))))

(def (source-header-value-start-forms)
  `((if ,header-space?
        ()
        ((token SourceHeaderTrivia (state-offset source-header-cursor)
                ,header-index)
         (set-uint source-header-value-start (offset ,header-index))
         (if (line-byte-equal? ,header-index 34)
             ((set-uint source-header-mode (uint 4)))
             ((set-uint source-header-mode (uint 3))))))))

(def (source-header-bare-value-forms)
  `((if ,header-space?
        ((token SourceHeaderValue (state-offset source-header-value-start)
                ,header-index)
         (set-uint source-header-cursor (offset ,header-index))
         (set-uint source-header-mode (uint 0))) ())))

(def (source-header-quoted-value-forms)
  `((if (uint-equal? (state source-header-mode) (uint 5))
        ((set-uint source-header-mode (uint 4)))
        ((if (line-byte-equal? ,header-index 92)
             ((set-uint source-header-mode (uint 5)))
             ((if (line-byte-equal? ,header-index 34)
                  ((token SourceHeaderValue
                          (state-offset source-header-value-start) ,header-next)
                   (set-uint source-header-cursor (offset ,header-next))
                   (set-uint source-header-mode (uint 0))) ())))))))

(def event-source-header-initial
  '((source-header-mode 0) (source-header-cursor 0)
    (source-header-key-start 0) (source-header-value-start 0)))
