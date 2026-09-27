;;; -*- Gerbil -*-
;;; Org-owned high-level inline tasks; generic event IR only supplies lookahead.

(import (only-in "event-headline.ss"
                 headline-line-forms heading-marker heading-separator
                 offset-after))
(export inlinetask-event-initial inlinetask-start-condition
        inlinetask-end-condition inlinetask-open-forms inlinetask-end-forms
        inlinetask-pending-close-form)

(def inlinetask-min-level 15)
(def heading-level `(line-marker-level ,heading-marker ,heading-separator))
(def title-start
  `(line-skip-horizontal
    (line-marker-end ,heading-marker ,heading-separator)))

(def inlinetask-start-condition
  `(uint-greater? (uint-add ,heading-level (uint 1))
                  (state inlinetask-min-level)))

(def inlinetask-end-condition
  `(and ,inlinetask-start-condition
        (line-byte-equal? ,title-start 69)
        (line-byte-equal? (line-step ,title-start) 78)
        (line-byte-equal? ,(offset-after title-start 2) 68)
        (line-bytes-all-in? ,(offset-after title-start 3) end
                            (9 10 13 32))))

(def inlinetask-event-initial
  `((inlinetask-min-level ,inlinetask-min-level)
    (inlinetask-open #f) (inlinetask-pending #f)
    (inlinetask-body-levels (uint-stack))))

(def (inlinetask-pending-close-form keep-condition)
  `(if (and (state inlinetask-pending) (not ,keep-condition))
       ((finish-node) (set-bool inlinetask-pending (bool #f))) ()))

(def (inlinetask-open-forms)
  `((start-node OrgInlinetask)
    ,@(headline-line-forms)
    (if (future-heading-title?
         ,heading-marker ,heading-separator (state inlinetask-min-level) "END")
        ((set-bool inlinetask-open (bool #t)))
        ((set-bool inlinetask-pending (bool #t))))
    (set-bool after-heading (bool #t))))

(def (inlinetask-end-forms)
  `((close-all inlinetask-body-levels)
    (start-node OrgInlinetaskEnd)
    (token HeadlineLine start
           (line-marker-end ,heading-marker ,heading-separator))
    (token InlinetaskEndLine
           (line-marker-end ,heading-marker ,heading-separator) end)
    (finish-node)
    (finish-node)
    (set-bool inlinetask-open (bool #f))
    (set-bool inlinetask-pending (bool #f))
    (set-bool after-heading (bool #f))))
