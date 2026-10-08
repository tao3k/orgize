;;; -*- Gerbil -*-
;;; Org inline-link recognition follows the POO-declared text-line rule.

(import (only-in :gerbil-parser/src/modules/parser/interface
                 source-pattern-end source-pattern-at?)
        (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure-text text-line-inline-link
                 inline-link-opening inline-link-separator inline-link-closing
                 inline-link-node inline-link-target-token
                 inline-link-description-token inline-link-trivia-token)
        (only-in "../../parser.ss" org-line-structure)
        (only-in "event-inline-primitives.ss"
                 link-index))
(export link-open link-scan-forms inline-link-event-initial)

(def link-rule
  (text-line-inline-link (line-structure-text org-line-structure)))
(def link-open (inline-link-opening link-rule))
(def link-separator (inline-link-separator link-rule))
(def link-close (inline-link-closing link-rule))

(def inline-link-event-initial
  '((inline-open #f) (inline-has-separator #f)
    (inline-failed #f) (inline-open-at 0) (inline-target-start 0)
    (inline-separator-at 0)))

(def (link-valid? target-end)
  `(offset-less? (state-offset inline-target-start) ,target-end))

(def (link-events nested-description?)
  (let ((trivia (inline-link-trivia-token link-rule))
        (target (inline-link-target-token link-rule))
        (description (inline-link-description-token link-rule))
        (separator-end (source-pattern-end '(state-offset inline-separator-at)
                                    link-separator))
        (close-end (source-pattern-end link-index link-close)))
    `((token TextLine (state-offset inline-cursor)
             (state-offset inline-open-at))
      (start-node ,(inline-link-node link-rule))
      (token ,trivia (state-offset inline-open-at)
             (state-offset inline-target-start))
      (if (state inline-has-separator)
          ((token ,target (state-offset inline-target-start)
                  (state-offset inline-separator-at))
           (token ,trivia (state-offset inline-separator-at) ,separator-end)
           ,@(if nested-description?
               `((token ,description ,separator-end ,link-index))
               `((start-node OrgLinkDescription)
                 (call-source-helper link-description-span
                                     ,separator-end ,link-index
                                     ((state inline-script-policy)))
                 (finish-node))))
          ((token ,target (state-offset inline-target-start) ,link-index)))
      (token ,trivia ,link-index ,close-end)
      (finish-node)
      (set-uint inline-cursor (offset ,close-end))
      (set-bool inline-open (bool #f)))))

(def (link-close-forms nested-description?)
  `((if (state inline-has-separator)
        ((if ,(link-valid? '(state-offset inline-separator-at))
             ,(link-events nested-description?)
             ((set-bool inline-failed (bool #t)))))
        ((if ,(link-valid? link-index)
             ,(link-events nested-description?)
             ((set-bool inline-failed (bool #t))))))))

(def (link-scan-forms (nested-description? #f))
  `((if (and (not (state inline-failed))
             (not (state inline-open)))
        ((if ,(source-pattern-at? link-index link-open)
             ((set-bool inline-open (bool #t))
              (set-bool inline-has-separator (bool #f))
              (set-uint inline-open-at (offset ,link-index))
              (set-uint inline-target-start
                        (offset ,(source-pattern-end link-index link-open)))) ()))
        ((if (and (state inline-open)
                  ,(source-pattern-at? link-index link-close))
             ,(link-close-forms nested-description?)
             ((if (and (state inline-open)
                       (not (state inline-has-separator))
                       ,(source-pattern-at? link-index link-separator))
                  ((set-bool inline-has-separator (bool #t))
                   (set-uint inline-separator-at (offset ,link-index))) ())))))))
