;;; -*- Gerbil -*-
;;; Paragraph lifetime and cross-line inline strategy owned by Org.

(import (only-in "event-inline.ss" event-inline-initial event-text-line-forms))
(export paragraph-event-initial paragraph-close-form paragraph-finish-form
        paragraph-line-form
        paragraph-event-helpers)

(def paragraph-event-initial
  '((paragraph-open #f) (paragraph-start 0) (paragraph-end 0)
    (paragraph-blank-end 0) (paragraph-post-blank #f)))

(def (paragraph-close-form* reset-state?)
  `(if (state paragraph-open)
       ((call-source-helper inline-span
                            (state-offset paragraph-start)
                            (state-offset paragraph-end))
        (if (state paragraph-post-blank)
            ((start-node OrgTextLine)
             (token TextLine (state-offset paragraph-end)
                    (state-offset paragraph-blank-end))
             (finish-node)) ())
        (finish-node)
        ,@(if reset-state?
            '((set-bool paragraph-open (bool #f))
              (set-bool paragraph-post-blank (bool #f)))
            '())) ()))

(def paragraph-close-form (paragraph-close-form* #t))
(def paragraph-finish-form (paragraph-close-form* #f))

(def (paragraph-line-form)
  `(if (line-blank?)
       ((if (state paragraph-open)
            ((set-bool paragraph-post-blank (bool #t))
             (set-uint paragraph-blank-end (offset end)))
            ((start-node OrgTextLine)
             (token TextLine start end) (finish-node))))
       ((if (state paragraph-post-blank) (,paragraph-close-form) ())
        (if (not (state paragraph-open))
            ((start-node OrgParagraph)
             (set-bool paragraph-open (bool #t))
             (set-uint paragraph-start (offset start))) ())
        (set-uint paragraph-end (offset end)))))

(def paragraph-event-helpers
  `((inline-span ,event-inline-initial ,(event-text-line-forms 'start))))
