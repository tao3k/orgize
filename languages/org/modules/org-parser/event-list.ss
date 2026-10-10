;;; -*- Gerbil -*-
;;; Org list transitions projected from the declared POO list rule.

(import (only-in :gerbil-parser/src/modules/parser/interface
                 make-source-event-scope source-event-name
                 line-structure-list line-structure-text list-line-trivia-token
                 source-list-initial source-list-scan-form source-list-present-condition
                 source-list-ordered-condition source-list-active-condition
                 source-list-first-blank-condition source-list-continuation-condition
                 source-list-content-offset source-list-item-open-forms
                 source-list-paragraph-span-forms source-list-paragraph-close-form
                 source-list-close-forms source-list-first-blank-forms source-list-reset-blank-form)
        (only-in "../../parser.ss" org-line-structure)
        (only-in "event-headline.ss" heading-marker heading-separator))
(export list-event-initial list-close-paragraph list-close-all list-finish-forms list-or-element-form)

(def list-rule (line-structure-list org-line-structure))
(def list-text-rule (line-structure-text org-line-structure))
(def list-scope (make-source-event-scope 'org-list))
(def list-handled (source-event-name list-scope 'line-handled))
(def list-event-initial (source-list-initial list-rule list-scope list-text-rule))
(def list-close-paragraph
  (source-list-paragraph-close-form list-text-rule list-scope 'inline-span '(inline-script-policy)))
(def list-close-all
  (source-list-close-forms list-text-rule list-scope 'inline-span '(inline-script-policy)))
(def list-finish-forms
  (source-list-close-forms list-text-rule list-scope 'inline-span '(inline-script-policy) #f))
(def (list-text-line from)
  (source-list-paragraph-span-forms list-text-rule list-scope from 'end))

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
  `(,@(source-list-item-open-forms list-rule list-scope ordered? bullet-span: 'with-separator)
    ,@(list-content-forms (source-list-content-offset list-scope) ordered?)
    ,(source-list-reset-blank-form list-scope)))

(def (list-marker-forms table-close-form fixed-width-close close-paragraph)
  `(,table-close-form
    ,fixed-width-close
    ,close-paragraph
    ,list-close-paragraph
    (if ,(source-list-ordered-condition list-scope)
        ,(list-marker-body #t)
        ,(list-marker-body #f))
    (set-bool after-heading (bool #f))))

(def (list-or-element-form table-close-form fixed-width-close close-paragraph
                           comment-line? comment-line-forms
                           org-table-or-element-form continuation-element-forms)
  `(,(source-list-scan-form list-rule list-scope)
    (join-once ,list-handled
      ((if (and ,(source-list-present-condition list-scope)
                (uint-equal?
                 (line-marker-level ,heading-marker ,heading-separator) (uint 0)))
           (,@(list-marker-forms table-close-form fixed-width-close close-paragraph)
            (set-bool ,list-handled (bool #t)))
           ((if ,(source-list-active-condition list-scope)
                ((if (line-blank?)
                     ((if ,(source-list-first-blank-condition list-scope)
                          (,@(source-list-first-blank-forms list-rule list-text-rule list-scope
                                                       'inline-span '(inline-script-policy))
                           (set-bool ,list-handled (bool #t)))
                          (,@list-close-all)))
                     ((if ,(source-list-continuation-condition list-rule list-scope)
                          ((if ,comment-line?
                               (,list-close-paragraph ,@(comment-line-forms))
                               (,@(continuation-element-forms)
                                (if (uint-positive? (state active-opaque-block))
                                    () ,(list-text-line 'start))))
                           ,(source-list-reset-blank-form list-scope)
                           (set-bool ,list-handled (bool #t)))
                          (,@list-close-all)))))
                ()))))
      (,(org-table-or-element-form)))))
