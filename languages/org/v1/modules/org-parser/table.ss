;;; -*- Gerbil -*-
;;; Org table rows are source-backed; the generic Rowan sink only sees events.

(import (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure-table table-line-delimiter
                 table-line-table-node table-line-row-node
                 table-line-rule-row-node table-line-cell-node
                 table-line-separator-token table-line-cell-token
                 table-line-trivia-token table-line-rule-token)
        (only-in "../../parser.ss" org-v1-line-structure)
        (only-in "funs.ss" skip-horizontal line-content-end
                 token-if-nonempty))
(export org-table-line? org-table-node emit-org-table-row)

(def table-rule (line-structure-table org-v1-line-structure))
(def table-delimiter (u8vector-ref (string->utf8 (table-line-delimiter table-rule)) 0))
(def org-table-node (table-line-table-node table-rule))

(def (org-table-line? bytes span)
  (let* ((end (line-content-end bytes span))
         (start (skip-horizontal bytes (car span) end)))
    (and (< start end) (= (u8vector-ref bytes start) table-delimiter))))

(def (table-rule-row? bytes start end)
  (let loop ((offset start) (has-dash? #f))
    (if (= offset end)
      has-dash?
      (let (byte (u8vector-ref bytes offset))
        (and (or (= byte table-delimiter)
                 (memv byte '(43 45 58 9 32)))
             (loop (+ offset 1) (or has-dash? (= byte 45))))))))

;; The escaped flag is the parity of immediately preceding backslashes.
(def (table-separators bytes start end)
  (let loop ((offset start) (escaped? #f) (positions '()))
    (if (= offset end)
      (reverse positions)
      (let (byte (u8vector-ref bytes offset))
        (cond
         ((= byte 92) (loop (+ offset 1) (not escaped?) positions))
         ((and (= byte table-delimiter) (not escaped?))
          (loop (+ offset 1) #f (cons offset positions)))
         (else (loop (+ offset 1) #f positions)))))))

(def (emit-events reversed events)
  (foldl cons reversed events))

(def (horizontal-only? bytes start end)
  (let loop ((offset start))
    (or (= offset end)
        (and (memv (u8vector-ref bytes offset) '(9 32))
             (loop (+ offset 1))))))

(def (emit-table-cell reversed bytes start end has-next?)
  (if (and (not has-next?) (horizontal-only? bytes start end))
    (emit-events reversed
                 (token-if-nonempty (table-line-trivia-token table-rule)
                                    start end))
    (emit-events reversed
                 (append
                  (list (list 'start (table-line-cell-node table-rule)))
                  (token-if-nonempty (table-line-cell-token table-rule)
                                     start end)
                  (list '(finish))))))

(def (emit-table-cells reversed bytes positions content-end)
  (if (null? positions)
    reversed
    (let* ((separator (car positions))
           (next (and (pair? (cdr positions)) (cadr positions)))
           (after-separator
            (cons (list 'token (table-line-separator-token table-rule)
                        separator (+ separator 1)) reversed))
           (after-cell (emit-table-cell after-separator bytes
                                        (+ separator 1)
                                        (or next content-end)
                                        (if next #t #f))))
      (emit-table-cells after-cell bytes (cdr positions) content-end))))

(def (emit-org-table-row reversed bytes span)
  (let* ((start (car span))
         (end (cdr span))
         (content-end (line-content-end bytes span))
         (indent (skip-horizontal bytes start content-end)))
    (if (table-rule-row? bytes indent content-end)
      (emit-events reversed
                   (list (list 'start (table-line-rule-row-node table-rule))
                         (list 'token (table-line-rule-token table-rule)
                               start end)
                         '(finish)))
      (let* ((opened (emit-events reversed
                                  (append
                                   (list (list 'start
                                               (table-line-row-node table-rule)))
                                   (token-if-nonempty
                                    (table-line-trivia-token table-rule)
                                    start indent))))
             (cells (emit-table-cells opened bytes
                                      (table-separators bytes indent content-end)
                                      content-end)))
        (emit-events cells
                     (append
                      (token-if-nonempty (table-line-trivia-token table-rule)
                                         content-end end)
                      (list '(finish))))))))
