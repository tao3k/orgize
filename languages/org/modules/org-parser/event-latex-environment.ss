;;; -*- Gerbil -*-
;;; Org-owned, source-named LaTeX environment with an opaque body.

(import (only-in :gerbil-parser/src/modules/parser/interface
                 source-ascii-ci-pattern-at? source-pattern-end
                 make-source-named-boundary line-structure-heading)
        (only-in "../../parser.ss" org-line-structure)
        (only-in "objects.ss"
                 make-org-named-block org-named-block-opening
                 org-named-block-closing org-named-block-node
                 org-named-block-name-token))
(export latex-environment-initial latex-boundary-query
        latex-open-form latex-body-form)

(def latex-rule
  (make-org-named-block "\\begin{" "\\end{"
                        'OrgLatexEnvironment 'LatexEnvironmentName))
(def latex-indent '(line-skip-horizontal start))
(def latex-opening (org-named-block-opening latex-rule))
(def latex-closing (org-named-block-closing latex-rule))
(def latex-name-start
  (source-pattern-end latex-indent latex-opening))
(def latex-name-end `(line-scan-until ,latex-name-start "}"))
(def latex-body-start `(line-step ,latex-name-end))
(def latex-close-name-start
  (source-pattern-end latex-indent latex-closing))
(def latex-close-name-end
  `(line-scan-until ,latex-close-name-start "}"))
(def latex-close-after `(line-step ,latex-close-name-end))
(def latex-name-bytes
  (map char->integer
       (string->list
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789*")))

(def latex-environment-initial
  '((latex-open #f) (latex-name-start 0) (latex-name-end 0)
    (latex-same-line-close #f) (latex-same-line-close-at 0)))

(def latex-boundary-query
  (make-source-named-boundary latex-opening latex-closing "}"
                             (line-structure-heading org-line-structure)
                             "}" #t #f #t))

(def latex-opening?
  `(and ,(source-ascii-ci-pattern-at? latex-indent latex-opening)
        (offset-less? ,latex-name-start ,latex-name-end)
        (line-bytes-all-in? ,latex-name-start ,latex-name-end
                            ,latex-name-bytes)
        (line-byte-equal? ,latex-name-end 125)))

(def latex-look-index '(line-index latex-look-index))
(def latex-look-name-start
  (source-pattern-end latex-look-index latex-closing))
(def latex-look-name-end
  `(line-scan-until ,latex-look-name-start "}"))
(def latex-look-after `(line-step ,latex-look-name-end))
(def latex-same-line-close?
  `(and ,(source-ascii-ci-pattern-at? latex-look-index latex-closing)
        (line-byte-equal? ,latex-look-name-end 125)
        (source-slices-equal-ascii-ci?
         ,latex-name-start ,latex-name-end
         ,latex-look-name-start ,latex-look-name-end)
        (line-bytes-all-in? ,latex-look-after end (9 10 13 32))))

(def latex-prescan-form
  `(if ,latex-opening?
       ((set-bool latex-same-line-close (bool #f))
        (for-line-bytes latex-look-index ,latex-body-start (line-content-end)
          ((if (and (not (state latex-same-line-close))
                    ,latex-same-line-close?)
               ((set-bool latex-same-line-close (bool #t))
                (set-uint latex-same-line-close-at
                          (offset ,latex-look-index))) ()))))
       ()))

(def (latex-open-form close-paragraph future otherwise)
  `(,latex-prescan-form
    (if (and ,latex-opening?
             (or (state latex-same-line-close) ,future))
        (,close-paragraph
         (start-node ,(org-named-block-node latex-rule))
         (token LatexEnvironmentBegin start ,latex-name-start)
         (token ,(org-named-block-name-token latex-rule)
                ,latex-name-start ,latex-name-end)
         (token LatexEnvironmentBeginSuffix
                ,latex-name-end ,latex-body-start)
         (if (state latex-same-line-close)
             ((token LatexEnvironmentBody ,latex-body-start
                     (state-offset latex-same-line-close-at))
              (token LatexEnvironmentEnd
                     (state-offset latex-same-line-close-at) end)
              (finish-node))
             ((token LatexEnvironmentBody ,latex-body-start end)
              (set-uint latex-name-start (offset ,latex-name-start))
              (set-uint latex-name-end (offset ,latex-name-end))
              (set-bool latex-open (bool #t))))
         (set-bool after-heading (bool #f)))
        (,otherwise))))

(def latex-closing?
  `(and ,(source-ascii-ci-pattern-at? latex-indent latex-closing)
        (offset-less? ,latex-close-name-start ,latex-close-name-end)
        (line-byte-equal? ,latex-close-name-end 125)
        (source-slices-equal-ascii-ci?
         (state-offset latex-name-start) (state-offset latex-name-end)
         ,latex-close-name-start ,latex-close-name-end)
        (line-bytes-all-in? ,latex-close-after end (9 10 13 32))))

(def latex-body-form
  `(if ,latex-closing?
       ((token LatexEnvironmentEnd start end)
        (finish-node)
        (set-bool latex-open (bool #f)))
       ((token LatexEnvironmentBody start end))))
