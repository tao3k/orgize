;;; -*- Gerbil -*-
;;; Source-named Org special blocks, owned by Orgize rather than the engine.

(import (only-in :gerbil-parser/src/modules/parser/interface
                 source-ascii-ci-pattern-at? source-pattern-end
                 make-source-named-boundary line-structure-heading)
        (only-in "../../parser.ss" org-line-structure)
        (only-in "objects.ss"
                 make-org-named-block org-named-block-opening
                 org-named-block-closing org-named-block-node
                 org-named-block-name-token))
(export special-event-initial special-block-id special-closing special-boundary-query
        special-open-form special-close-form)

(def special-block-id 6)
(def special-rule
  (make-org-named-block "#+BEGIN_" "#+END_"
                        'OrgSpecialBlock 'SpecialBlockName))
(def special-indent '(line-skip-horizontal start))
(def special-opening (org-named-block-opening special-rule))
(def special-closing (org-named-block-closing special-rule))
(def special-opening-end
  (source-pattern-end special-indent special-opening))
(def special-name-start special-opening-end)
(def special-name-end `(line-scan-key ,special-name-start))
(def special-closing-end
  (source-pattern-end special-indent special-closing))
(def special-close-name-end `(line-scan-key ,special-closing-end))
(def special-letter-bytes
  (map char->integer
       (string->list "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz")))

(def special-event-initial
  '((special-name-start 0) (special-name-end 0)
    (special-name-start-frames (uint-stack))
    (special-name-end-frames (uint-stack))))

(def special-boundary-query
  (make-source-named-boundary special-opening special-closing ""
                             (line-structure-heading org-line-structure)))

(def (special-open-condition future)
  `(and ,(source-ascii-ci-pattern-at? special-indent special-opening)
        (offset-less? ,special-name-start ,special-name-end)
        (line-bytes-any-in? ,special-name-start
                            (line-step ,special-name-start)
                            ,special-letter-bytes)
        ,future))

(def (special-open-form close-paragraph future)
  `(if ,(special-open-condition future)
       (,close-paragraph
        (start-node ,(org-named-block-node special-rule))
        (token BlockBeginLine start ,special-opening-end)
        (token ,(org-named-block-name-token special-rule)
               ,special-name-start ,special-name-end)
        (token BlockHeaderTrivia ,special-name-end end)
        (push-frame special-name-start-frames (state special-name-start))
        (push-frame special-name-end-frames (state special-name-end))
        (set-uint special-name-start (offset ,special-name-start))
        (set-uint special-name-end (offset ,special-name-end))
        (push-frame container-frames (uint ,special-block-id))
        (set-bool after-heading (bool #f))
        (set-bool container-opened (bool #t)))
       ()))

(def special-close-condition
  `(and ,(source-ascii-ci-pattern-at? special-indent special-closing)
        (offset-less? ,special-closing-end ,special-close-name-end)
        (source-slices-equal-ascii-ci?
         (state-offset special-name-start)
         (state-offset special-name-end)
         ,special-closing-end ,special-close-name-end)
        (line-bytes-all-in? ,special-close-name-end end (9 10 13 32))))

(def (special-close-form close-paragraph list-close-all
                         fixed-width-close table-close-form)
  `(if (and (uint-equal? (stack-top container-frames)
                         (uint ,special-block-id))
            ,special-close-condition)
       (,close-paragraph
        ,@list-close-all
        ,fixed-width-close
        ,table-close-form
        (token BlockEndLine start end)
        (close-frame container-frames 1)
        (set-uint special-name-start (stack-top special-name-start-frames))
        (set-uint special-name-end (stack-top special-name-end-frames))
        (pop-frame special-name-start-frames)
        (pop-frame special-name-end-frames)
        (set-bool container-closed (bool #t)))
       ()))
