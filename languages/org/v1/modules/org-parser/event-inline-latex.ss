;;; -*- Gerbil -*-
;;; Org-owned LaTeX math delimiters; the emitted span is compiled to Rowan.

(import (only-in "event-inline-primitives.ss"
                 link-index inline-next pattern-end pattern-at?))
(export latex-event-initial latex-prescan-forms
        latex-open-forms latex-scan-forms)

(def latex-single-border-invalid '(9 10 13 32 44 46 59))
(def latex-single-right-boundary
  '(9 10 13 32 33 34 39 40 41 44 46 58 59 63 91 93 123 125))
(def latex-look-index '(line-index inline-latex-look-index))
(def latex-look-next `(line-step ,latex-look-index))

(def (latex-prescan-forms from)
  `((if (line-bytes-any-in? ,from (line-content-end) (36 92))
        ((for-line-bytes inline-latex-look-index ,from (line-content-end)
           ((if ,(pattern-at? latex-look-index "\\)")
                 ((set-uint inline-latex-last-round
                            (offset ,latex-look-index))) ())
            (if ,(pattern-at? latex-look-index "\\]")
                ((set-uint inline-latex-last-square
                           (offset ,latex-look-index))) ())
            (if ,(pattern-at? latex-look-index "$$")
                ((set-uint inline-latex-last-double
                           (offset ,latex-look-index))) ())
            (if (and (line-byte-equal? ,latex-look-index 36)
                     (not (state inline-latex-look-previous-invalid))
                     (or (line-bytes-all-in?
                          ,latex-look-next (line-content-end) ())
                         (line-bytes-any-in?
                          ,latex-look-next (line-step ,latex-look-next)
                          ,latex-single-right-boundary)))
                ((set-uint inline-latex-last-single
                           (offset ,latex-look-index))) ())
            (set-bool inline-latex-look-previous-invalid
                      (line-bytes-any-in?
                       ,latex-look-index ,latex-look-next
                       ,latex-single-border-invalid))))) ())))

(def (latex-complete-forms closing)
  (let (after (pattern-end link-index closing))
    `((token TextLine (state-offset inline-cursor)
             (state-offset inline-latex-open-at))
      (start-node OrgLaTeXFragment)
      (token LatexFragmentValue (state-offset inline-latex-open-at) ,after)
      (finish-node)
      (set-uint inline-cursor (offset ,after))
      (set-uint inline-latex-mode (uint 0)))))

(def (latex-open-forms)
  `((if (and ,(pattern-at? link-index "\\(")
             (offset-less? ,link-index
                           (state-offset inline-latex-last-round)))
        ((set-uint inline-latex-mode (uint 1))
         (set-uint inline-latex-open-at (offset ,link-index)))
        ((if (and ,(pattern-at? link-index "\\[")
                  (offset-less? ,link-index
                                (state-offset inline-latex-last-square)))
             ((set-uint inline-latex-mode (uint 2))
              (set-uint inline-latex-open-at (offset ,link-index)))
             ((if (and ,(pattern-at? link-index "$$")
                       (offset-less? ,(pattern-end link-index "$")
                                     (state-offset inline-latex-last-double)))
                  ((set-uint inline-latex-mode (uint 3))
                   (set-uint inline-latex-open-at (offset ,link-index)))
                  ((if (and (line-byte-equal? ,link-index 36)
                            (offset-less?
                             ,link-index
                             (state-offset inline-latex-last-single))
                            (state inline-left-boundary)
                            (not (line-bytes-any-in?
                                  ,inline-next (line-step ,inline-next)
                                  ,latex-single-border-invalid)))
                       ((set-uint inline-latex-mode (uint 4))
                        (set-uint inline-latex-open-at
                                  (offset ,link-index)))
                       ())))))))))

(def (latex-scan-forms)
  `((if (uint-equal? (state inline-latex-mode) (uint 1))
        ((if ,(pattern-at? link-index "\\)")
             ,(latex-complete-forms "\\)") ()))
        ((if (uint-equal? (state inline-latex-mode) (uint 2))
             ((if ,(pattern-at? link-index "\\]")
                  ,(latex-complete-forms "\\]") ()))
             ((if (uint-equal? (state inline-latex-mode) (uint 3))
                  ((if (and (offset-less?
                             ,(pattern-end
                               '(state-offset inline-latex-open-at) "$")
                             ,link-index)
                            ,(pattern-at? link-index "$$"))
                       ,(latex-complete-forms "$$") ()))
                  ((if (and (line-byte-equal? ,link-index 36)
                            (offset-less?
                             (state-offset inline-latex-open-at) ,link-index)
                            (not (state inline-latex-previous-invalid))
                            (or (line-bytes-all-in?
                                 ,inline-next (line-content-end) ())
                                (line-bytes-any-in?
                                 ,inline-next (line-step ,inline-next)
                                 ,latex-single-right-boundary)))
                       ,(latex-complete-forms "$") ())))))))))

(def latex-event-initial
  '((inline-latex-mode 0) (inline-latex-open-at 0)
    (inline-latex-previous-invalid #f)
    (inline-latex-last-round 0) (inline-latex-last-square 0)
    (inline-latex-last-double 0) (inline-latex-last-single 0)
    (inline-latex-look-previous-invalid #t)))
