;;; -*- Gerbil -*-
;;; Org-owned section containment over source-backed line events.
;;; This pure pass is the algorithm to lower into Rust; it is not a runtime
;;; dependency of Cargo consumers.

(import (only-in "line-event-parser.ss" parse-org-line-events))
(export parse-org-outline-events org-headline-level)

(def (org-headline-level bytes start end)
  (let loop ((offset start) (level 0))
    (cond
     ((>= offset end) 0)
     ((= (u8vector-ref bytes offset) 42)
      (loop (+ offset 1) (+ level 1)))
     ((and (> level 0) (= (u8vector-ref bytes offset) 32)) level)
     (else 0))))

(def (close-sections levels reversed)
  (if (null? levels)
    reversed
    (close-sections (cdr levels) (cons '(finish) reversed))))

(def (close-through-level levels reversed level)
  (if (or (null? levels) (< (car levels) level))
    (values levels reversed)
    (close-through-level (cdr levels) (cons '(finish) reversed) level)))

(def (emit-line reversed node token start end)
  (cons '(finish)
        (cons (list 'token token start end)
              (cons (list 'start node) reversed))))

(def (parse-org-outline-events source)
  (let* ((bytes (string->utf8 source))
         (flat (parse-org-line-events source)))
    (let loop ((rest (cdr flat)) (levels '())
               (reversed '((start OrgFile))))
      (if (equal? (car rest) '(finish))
        (reverse (cons '(finish) (close-sections levels reversed)))
        (let* ((line-token (cadr rest))
               (start (caddr line-token))
               (end (cadddr line-token))
               (level (org-headline-level bytes start end)))
          (if (> level 0)
            (let-values (((parents closed)
                          (close-through-level levels reversed level)))
              (loop (cdddr rest)
                    (cons level parents)
                    (emit-line (cons '(start OrgSection) closed)
                               'OrgHeadline 'HeadlineLine start end)))
            (loop (cdddr rest) levels
                  (emit-line reversed 'OrgTextLine 'TextLine start end))))))))
