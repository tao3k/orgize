;;; -*- Gerbil -*-
;;; Source-block header arguments are Scheme-owned, source-backed event tokens.

(export event-source-header-initial event-source-header-forms)

(def header-index '(line-index source-header-byte-index))
(def header-next `(line-step ,header-index))
(def header-space? `(line-bytes-any-in? ,header-index ,header-next (9 32)))
(def header-after-switch `(line-step ,header-next))
(def header-switch-boundary?
  `(or (line-bytes-any-in? ,header-after-switch
                           (line-step ,header-after-switch) (9 32))
       (line-bytes-all-in? ,header-after-switch (line-content-end) ())))
(def header-switch-sign?
  `(and ,header-switch-boundary?
        (or (and (line-byte-equal? ,header-index 45)
                 (line-bytes-any-in? ,header-next (line-step ,header-next)
                                     (105 107 108 110 114)))
            (and (line-byte-equal? ,header-index 43)
                 (line-byte-equal? ,header-next 110)))))
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
    (if (or (uint-equal? (state source-header-mode) (uint 9))
            (uint-equal? (state source-header-mode) (uint 10))
            (uint-equal? (state source-header-mode) (uint 11)))
        ((token SourceSwitchValue (state-offset source-header-value-start)
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
                            ((if (uint-equal? (state source-header-mode) (uint 7))
                                 ,(source-header-switch-end-forms)
                                 ((if (uint-equal? (state source-header-mode) (uint 8))
                                      ,(source-header-switch-value-start-forms)
                                      ((if (uint-equal? (state source-header-mode) (uint 9))
                                           ,(source-header-switch-bare-value-forms)
                                           ((if (or (uint-equal? (state source-header-mode) (uint 10))
                                                    (uint-equal? (state source-header-mode) (uint 11)))
                                                ,(source-header-switch-quoted-value-forms)
                                                ,(source-header-quoted-value-forms))))))))))))))))))))

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
             ((if ,header-switch-sign?
                  ((token SourceHeaderTrivia
                          (state-offset source-header-cursor) ,header-index)
                   (token SourceSwitchName ,header-index
                          (line-step ,header-next))
                   (set-uint source-header-cursor
                             (offset (line-step ,header-next)))
                   (set-uint source-header-switch-argument
                             (uint 0))
                   (if (or (line-byte-equal? ,header-next 108)
                           (line-byte-equal? ,header-next 110))
                       ((set-uint source-header-switch-argument (uint 1))) ())
                   (set-uint source-header-mode (uint 7)))
                  ((set-uint source-header-mode (uint 6))))))))))

(def (source-header-switch-end-forms)
  `((if ,header-space?
        ((if (uint-positive? (state source-header-switch-argument))
             ((set-uint source-header-mode (uint 8)))
             ((set-uint source-header-mode (uint 0))))) ())))

(def (source-header-switch-value-start-forms)
  `((if ,header-space?
        ()
        ((if (and (line-byte-equal? ,header-index 58)
                  (line-bytes-any-in? ,header-next
                                      (line-step ,header-next)
                                      ,header-key-bytes))
             ((token SourceHeaderTrivia
                     (state-offset source-header-cursor) ,header-next)
              (set-uint source-header-cursor (offset ,header-next))
              (set-uint source-header-key-start (offset ,header-next))
              (set-uint source-header-mode (uint 1)))
             ((token SourceHeaderTrivia
                     (state-offset source-header-cursor) ,header-index)
              (set-uint source-header-value-start (offset ,header-index))
              (if (line-byte-equal? ,header-index 34)
                  ((set-uint source-header-mode (uint 10)))
                  ((set-uint source-header-mode (uint 9))))))))))

(def (source-header-switch-bare-value-forms)
  `((if ,header-space?
        ((token SourceSwitchValue (state-offset source-header-value-start)
                ,header-index)
         (set-uint source-header-cursor (offset ,header-index))
         (set-uint source-header-mode (uint 0))) ())))

(def (source-header-switch-quoted-value-forms)
  `((if (uint-equal? (state source-header-mode) (uint 11))
        ((set-uint source-header-mode (uint 10)))
        ((if (line-byte-equal? ,header-index 92)
             ((set-uint source-header-mode (uint 11)))
             ((if (line-byte-equal? ,header-index 34)
                  ((token SourceSwitchValue
                          (state-offset source-header-value-start) ,header-next)
                   (set-uint source-header-cursor (offset ,header-next))
                   (set-uint source-header-mode (uint 0))) ())))))))

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
    (source-header-key-start 0) (source-header-value-start 0)
    (source-header-switch-argument 0)))
