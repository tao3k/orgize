;;; -*- Gerbil -*-
;;; Scheme-owned angle and plain URL Objects for the Org inline event fold.

(import (only-in "event-inline-primitives.ss"
                 link-index inline-next pattern-at? pattern-end))
(export url-link-event-initial url-link-open-forms
        url-link-scan-forms url-link-final-forms)

(def url-link-event-initial
  '((inline-url-kind 0) (inline-url-open-at 0)
    (inline-url-value-start 0)))

(def (url-link-events target-end angle? (reset? #t))
  (let (close-end (if angle? inline-next target-end))
    `((token TextLine (state-offset inline-cursor)
             (state-offset inline-url-open-at))
      (start-node OrgLink)
      ,@(if angle?
          `((token LinkTrivia (state-offset inline-url-open-at)
                   (state-offset inline-url-value-start)))
          '())
      (token LinkTarget (state-offset inline-url-value-start) ,target-end)
      ,@(if angle? `((token LinkTrivia ,target-end ,close-end)) '())
      (finish-node)
      (set-uint inline-cursor (offset ,close-end))
      ,@(if reset? '((set-uint inline-url-kind (uint 0))) '()))))

(def (url-link-open-forms)
  `((if (and (state inline-left-boundary)
             (or ,(pattern-at? link-index "<https://")
                 ,(pattern-at? link-index "<http://")))
        ((set-uint inline-url-kind (uint 1))
         (set-uint inline-url-open-at (offset ,link-index))
         (set-uint inline-url-value-start (offset ,inline-next)))
        ((if (and (state inline-left-boundary)
                  (or ,(pattern-at? link-index "https://")
                      ,(pattern-at? link-index "http://")))
             ((set-uint inline-url-kind (uint 2))
              (set-uint inline-url-open-at (offset ,link-index))
              (set-uint inline-url-value-start (offset ,link-index))) ())))))

(def url-link-terminal-bytes '(46 44 59 33 63 41))
(def url-link-boundary-bytes '(9 10 13 32 60 62))
(def url-link-next-boundary-bytes '(9 10 13 32))

(def (url-link-scan-forms)
  `((if (uint-equal? (state inline-url-kind) (uint 1))
        ((if (line-byte-equal? ,link-index 62)
             ,(url-link-events link-index #t)
             ((if (line-bytes-any-in? ,link-index ,inline-next
                                      ,url-link-boundary-bytes)
                  ((set-uint inline-url-kind (uint 0))) ()))))
        ((if (or (line-bytes-any-in? ,link-index ,inline-next
                                     ,url-link-boundary-bytes)
                 (and (line-bytes-any-in? ,link-index ,inline-next
                                          ,url-link-terminal-bytes)
                      (or (line-bytes-all-in? ,inline-next
                                              (line-content-end) ())
                          (line-bytes-any-in? ,inline-next
                                              (line-step ,inline-next)
                                              ,url-link-next-boundary-bytes))))
             ,(url-link-events link-index #f) ())))))

(def (url-link-final-forms)
  `((if (uint-equal? (state inline-url-kind) (uint 2))
        ,(url-link-events '(line-content-end) #f #f) ())))
