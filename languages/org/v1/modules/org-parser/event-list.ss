;;; -*- Gerbil -*-
;;; Org list transitions projected from the declared POO list rule.

(import (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure-list list-line-unordered-markers list-line-ordered
                 list-line-tab-width list-line-list-node list-line-item-node
                 list-line-bullet-token list-line-trivia-token)
        (only-in "../../parser.ss" org-v1-line-structure)
        (only-in "event-headline.ss" heading-marker heading-separator))
(export list-close-all list-close-paragraph-form list-or-element-form)

(def list-rule (line-structure-list org-v1-line-structure))

(def list-column '(state list-column))
(def list-top '(uint-divide (stack-top list-frames) (uint 2)))
(def list-frame-base `(uint-multiply ,list-column (uint 2)))
(def list-marker-form
  `(scan-list-marker ,(list-line-unordered-markers list-rule)
                     ,(list-line-ordered list-rule)
                     ,(list-line-tab-width list-rule)
                     list-present list-column list-ordered
                     list-bullet-start list-bullet-end list-content-start))

(def (list-frame-value ordered?)
  (if ordered? `(uint-add ,list-frame-base (uint 1)) list-frame-base))

(def (list-close-paragraph-form reset-state?)
  `(if (state list-paragraph-open)
       ((call-source-helper inline-span
                            (state-offset list-paragraph-start)
                            (state-offset list-paragraph-end))
        (finish-node)
        ,@(if reset-state? '((set-bool list-paragraph-open (bool #f))) '())) ()))

(def list-close-paragraph (list-close-paragraph-form #t))

(def list-close-all
  `(,list-close-paragraph
    (close-all-frames list-frames 2)
    (set-uint list-blank-count (uint 0))))

(def (list-text-line from)
  `((if (not (state list-paragraph-open))
        ((start-node OrgParagraph)
         (set-bool list-paragraph-open (bool #t))
         (set-uint list-paragraph-start (offset ,from))) ())
    (set-uint list-paragraph-end (offset end))))

(def list-counter-alnum-bytes
  (map char->integer
       (string->list "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz")))

(def (list-text-or-trivia from)
  `(if (offset-less? ,from (line-content-end))
       ,(list-text-line from)
       ((token ListTrivia ,from end))))

(def (list-tag-forms from ordered?)
  (let* ((tag-end `(line-scan-until ,from ":"))
         (separator-end `(line-step (line-step ,tag-end)))
         (body-start `(line-skip-horizontal ,separator-end)))
    (if ordered?
      (list (list-text-or-trivia from))
      `((if (and (offset-less? ,from ,tag-end)
                 (line-byte-equal? ,tag-end 58)
                 (line-byte-equal? (line-step ,tag-end) 58))
            ((token ListTagValue ,from ,tag-end)
             (token ListTrivia ,tag-end ,body-start)
             ,(list-text-or-trivia body-start))
            (,(list-text-or-trivia from)))))))

(def (list-checkbox-forms from ordered?)
  (let* ((value-start `(line-step ,from))
         (value-end `(line-step ,value-start))
         (marker-end `(line-step ,value-end))
         (next `(line-skip-horizontal ,marker-end)))
    `((if (and (line-byte-equal? ,from 91)
               (line-bytes-any-in? ,value-start ,value-end (32 45 88))
               (line-byte-equal? ,value-end 93))
          ((token ListTrivia ,from ,value-start)
           (token ListCheckboxValue ,value-start ,value-end)
           (token ListTrivia ,value-end ,next)
           ,@(list-tag-forms next ordered?))
          ,(list-tag-forms from ordered?)))))

(def (list-content-forms from ordered?)
  (let* ((value-start `(line-step (line-step ,from)))
         (value-end `(line-scan-until ,value-start "]"))
         (marker-end `(line-step ,value-end))
         (next `(line-skip-horizontal ,marker-end)))
    `((if (and (line-byte-equal? ,from 91)
               (line-byte-equal? (line-step ,from) 64)
               (offset-less? ,value-start ,value-end)
               (line-bytes-all-in? ,value-start ,value-end
                                   ,list-counter-alnum-bytes)
               (line-byte-equal? ,value-end 93))
          ((token ListTrivia ,from ,value-start)
           (token ListCounterValue ,value-start ,value-end)
           (token ListTrivia ,value-end ,next)
           ,@(list-checkbox-forms next ordered?))
          ,(list-checkbox-forms from ordered?)))))

(def (list-marker-body ordered?)
  (let ((frame (list-frame-value ordered?))
        (list-node (list-line-list-node list-rule))
        (item-node (list-line-item-node list-rule))
        (trivia (list-line-trivia-token list-rule))
        (bullet (list-line-bullet-token list-rule)))
    `((close-frames-while list-frames
        (or (uint-greater? ,list-top ,list-column)
            (and (uint-equal? ,list-top ,list-column)
                 (uint-not-equal? (stack-top list-frames) ,frame))) 2)
      (if (and (stack-nonempty? list-frames)
               (uint-equal? ,list-top ,list-column))
          ((finish-node))
          ((start-node ,list-node)
           (push-frame list-frames ,frame)))
      (start-node ,item-node)
      (token ,trivia start (state-offset list-bullet-start))
      (token ,bullet (state-offset list-bullet-start)
             (state-offset list-bullet-end))
      (token ,trivia (state-offset list-bullet-end)
             (state-offset list-content-start))
      ,@(list-content-forms '(state-offset list-content-start) ordered?)
      (set-uint list-blank-count (uint 0)))))

(def (list-marker-forms table-close-form fixed-width-close close-paragraph)
  `(,table-close-form
    ,fixed-width-close
    ,close-paragraph
    ,list-close-paragraph
    (if (state list-ordered)
        ,(list-marker-body #t)
        ,(list-marker-body #f))
    (set-bool after-heading (bool #f))))

(def (list-or-element-form table-close-form fixed-width-close close-paragraph
                           comment-line? comment-line-forms
                           org-table-or-element-form)
  `(,list-marker-form
    (if (and (state list-present)
             (uint-equal?
              (line-marker-level ,heading-marker ,heading-separator) (uint 0)))
        ,(list-marker-forms table-close-form fixed-width-close close-paragraph)
        ((if (stack-nonempty? list-frames)
             ((if (line-blank?)
                  ((if (uint-equal? (state list-blank-count) (uint 0))
                       (,list-close-paragraph
                        (token ,(list-line-trivia-token list-rule) start end)
                        (set-uint list-blank-count (uint 1)))
                       (,@list-close-all ,(org-table-or-element-form))))
                  ((if (uint-greater?
                        (line-indent-column ,(list-line-tab-width list-rule))
                        ,list-top)
                       ((if ,comment-line?
                            (,list-close-paragraph ,@(comment-line-forms))
                            ,(list-text-line 'start))
                        (set-uint list-blank-count (uint 0)))
                       (,@list-close-all ,(org-table-or-element-form))))))
             (,(org-table-or-element-form)))))))
