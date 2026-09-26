;;; -*- Gerbil -*-
;;; Org-owned subscript/superscript strategies projected to source-backed events.

(import (only-in "event-inline-primitives.ss" link-index inline-next)
        (only-in "objects.ss" make-org-inline-script
                 org-inline-script-byte org-inline-script-id
                 org-inline-script-node))
(export inline-script-trigger-bytes inline-script-upper-digit-bytes
        inline-script-open-forms
        inline-script-scan-forms inline-script-final-forms
        inline-script-event-initial)

(def inline-script-rules
  (list (make-org-inline-script 95 1 'OrgSubscript)
        (make-org-inline-script 94 2 'OrgSuperscript)))
(def inline-script-trigger-bytes
  (map org-inline-script-byte inline-script-rules))
(def inline-script-alnum-bytes
  (map char->integer
       (string->list
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789")))
(def inline-script-upper-bytes
  (map char->integer (string->list "ABCDEFGHIJKLMNOPQRSTUVWXYZ")))
(def inline-script-upper-digit-bytes
  (map char->integer (string->list "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789")))
(def inline-script-unbraced-bytes
  (append inline-script-alnum-bytes '(44 46 92)))

(def (script-node-forms body)
  (foldr (lambda (rule otherwise)
           `((if (uint-equal? (state inline-script-kind)
                              (uint ,(org-inline-script-id rule)))
                 ((start-node ,(org-inline-script-node rule))
                  ,@body
                  (finish-node))
                 ,otherwise)))
         '() inline-script-rules))

(def (script-events end delimiter-end (reset? #t))
  `((token TextLine (state-offset inline-cursor)
           (state-offset inline-script-open-at))
    ,@(script-node-forms
       `((token InlineScriptDelimiter (state-offset inline-script-open-at)
                (state-offset inline-script-value-start))
         (token InlineScriptValue (state-offset inline-script-value-start)
                ,end)
         ,@(if delimiter-end
               `((token InlineScriptDelimiter ,end ,delimiter-end))
               '())))
    (set-uint inline-cursor (offset ,(or delimiter-end end)))
    ,@(if reset? '((set-uint inline-script-mode (uint 0))) '())))

(def (script-open-unbraced rule identifier-tail?)
  `((set-uint inline-script-kind (uint ,(org-inline-script-id rule)))
    (set-uint inline-script-open-at (offset ,link-index))
    (set-uint inline-script-value-start (offset ,inline-next))
    (set-uint inline-script-end (offset ,inline-next))
    (set-bool inline-script-last-alnum (bool #f))
    (set-bool inline-script-upper-prefix
              ,(if identifier-tail?
                 '(uint-greater? (state inline-upper-run) (uint 1))
                 '(bool #f)))
    (set-bool inline-script-upper-tail (bool ,identifier-tail?))
    (set-bool inline-script-has-upper (bool #f))
    (set-uint inline-script-mode (uint 1))))

(def (inline-script-open-forms)
  `((if (and (state inline-previous-present)
             (not (state inline-previous-space)))
        ,(foldr
          (lambda (rule otherwise)
            `((if (line-byte-equal? ,link-index
                                    ,(org-inline-script-byte rule))
                  ((if (line-byte-equal? ,inline-next 42)
                       ((set-uint inline-script-kind
                                  (uint ,(org-inline-script-id rule)))
                        (set-uint inline-script-open-at (offset ,link-index))
                        (set-uint inline-script-value-start
                                  (offset ,inline-next))
                        ,@(script-events `(line-step ,inline-next) #f))
                       ((if (line-byte-equal? ,inline-next 123)
                            ((set-uint inline-script-kind
                                       (uint ,(org-inline-script-id rule)))
                             (set-uint inline-script-open-at
                                       (offset ,link-index))
                             (set-uint inline-script-value-start
                                       (offset (line-step ,inline-next)))
                             (set-uint inline-script-opens (uint 1))
                             (set-uint inline-script-closes (uint 0))
                             (set-uint inline-script-mode (uint 2)))
                            ((if (line-bytes-any-in? ,inline-next
                                                      (line-step ,inline-next)
                                                      ,inline-script-alnum-bytes)
                                 ,(script-open-unbraced rule #t)
                                 ((if (line-bytes-any-in? ,inline-next
                                                           (line-step ,inline-next)
                                                           (43 45))
                                      ,(script-open-unbraced rule #f)
                                      ()))))))))
                  ,otherwise)))
          '() inline-script-rules)
        ())))

(def (inline-script-finish-forms)
  `((if (uint-equal? (state inline-script-mode) (uint 1))
        ((if (and (state inline-script-last-alnum)
                  (not (and (state inline-script-upper-prefix)
                            (state inline-script-upper-tail)
                            (state inline-script-has-upper))))
             ,(script-events '(state-offset inline-script-end) #f)
             ((set-uint inline-script-mode (uint 0)))))
        ((set-uint inline-script-mode (uint 0))))))

(def (inline-script-final-forms)
  `((if (and (uint-equal? (state inline-script-mode) (uint 1))
             (state inline-script-last-alnum)
             (not (and (state inline-script-upper-prefix)
                       (state inline-script-upper-tail)
                       (state inline-script-has-upper))))
        ,(script-events '(state-offset inline-script-end) #f #f)
        ())))

(def (inline-script-scan-forms)
  `((if (offset-less? ,link-index (state-offset inline-script-value-start))
        ()
        ((if (uint-equal? (state inline-script-mode) (uint 2))
        ((if (line-byte-equal? ,link-index 123)
             ((set-uint inline-script-opens
                        (uint-add (state inline-script-opens) (uint 1))))
             ((if (line-byte-equal? ,link-index 125)
                  ((if (uint-equal? (state inline-script-opens)
                                    (uint-add (state inline-script-closes)
                                              (uint 1)))
                       ,(script-events link-index inline-next)
                       ((set-uint inline-script-closes
                                  (uint-add (state inline-script-closes)
                                            (uint 1))))))
                  ()))))
        ((if (line-bytes-any-in? ,link-index ,inline-next
                                 ,inline-script-unbraced-bytes)
             ((set-uint inline-script-end (offset ,inline-next))
              (set-bool inline-script-last-alnum
                        (line-bytes-any-in? ,link-index ,inline-next
                                            ,inline-script-alnum-bytes))
              (set-bool inline-script-upper-tail
                        (and (state inline-script-upper-tail)
                             (line-bytes-any-in?
                              ,link-index ,inline-next
                              ,inline-script-upper-digit-bytes)))
              (set-bool inline-script-has-upper
                        (or (state inline-script-has-upper)
                            (line-bytes-any-in? ,link-index ,inline-next
                                                ,inline-script-upper-bytes))))
             ,(inline-script-finish-forms))))))))

(def inline-script-event-initial
  '((inline-script-mode 0) (inline-script-kind 0)
    (inline-script-open-at 0) (inline-script-value-start 0)
    (inline-script-end 0) (inline-script-opens 0)
    (inline-script-closes 0)
    (inline-script-last-alnum #f)
    (inline-script-upper-prefix #f) (inline-script-upper-tail #f)
    (inline-script-has-upper #f) (inline-upper-run 0)
    (inline-previous-present #f)))
