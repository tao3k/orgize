;;; -*- Gerbil -*-
;;; Org table event policy projected from the declared line structure.

(import (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure-table table-line-delimiter
                 table-line-table-node table-line-row-node
                 table-line-rule-row-node table-line-cell-node
                 table-line-separator-token table-line-cell-token
                 table-line-trivia-token table-line-rule-token)
        (only-in "../../parser.ss" org-v1-line-structure))
(export table-event-initial table-close-form table-or-element-form)

(def table-rule (line-structure-table org-v1-line-structure))
(def table-byte
  (char->integer (string-ref (table-line-delimiter table-rule) 0)))
(def table-content-end '(line-content-end))
(def table-indent '(line-skip-horizontal start))
(def table-index '(line-index table-byte-index))
(def table-cell-start '(state-offset table-cell-start))
(def table-line-predicate `(line-byte-equal? ,table-indent ,table-byte))

(def table-event-initial
  '((table-open #f) (table-seen-separator #f) (table-escaped #f)
    (table-cell-start 0)))

(def table-close-form
  '(if (state table-open)
       ((finish-node) (set-bool table-open (bool #f))) ()))

(def (table-cell-forms until)
  `((start-node ,(table-line-cell-node table-rule))
    (token ,(table-line-cell-token table-rule) ,table-cell-start ,until)
    (finish-node)))

(def (table-row-form)
  `(if (and (line-bytes-all-in? ,table-indent ,table-content-end
                                 (,table-byte 43 45 58 9 32))
            (line-bytes-any-in? ,table-indent ,table-content-end (45)))
       ((start-node ,(table-line-rule-row-node table-rule))
        (token ,(table-line-rule-token table-rule) start end)
        (finish-node))
       ((start-node ,(table-line-row-node table-rule))
        (for-line-bytes table-byte-index ,table-indent ,table-content-end
          ((if (line-byte-equal? ,table-index 92)
               ((set-bool table-escaped (not (state table-escaped))))
               ((if (and (line-byte-equal? ,table-index ,table-byte)
                         (not (state table-escaped)))
                    ((if (state table-seen-separator)
                         ,(table-cell-forms table-index)
                         ((token ,(table-line-trivia-token table-rule)
                                 start ,table-index)))
                     (token ,(table-line-separator-token table-rule)
                            ,table-index (line-step ,table-index))
                     (set-uint table-cell-start
                               (offset (line-step ,table-index)))
                     (set-bool table-seen-separator (bool #t))) ())
                (set-bool table-escaped (bool #f))))))
        (if (state table-seen-separator)
            ((if (line-bytes-all-in? ,table-cell-start ,table-content-end
                                     (9 32))
                 ((token ,(table-line-trivia-token table-rule)
                         ,table-cell-start ,table-content-end))
                 ,(table-cell-forms table-content-end))) ())
        (token ,(table-line-trivia-token table-rule) ,table-content-end end)
        (finish-node)
        (set-bool table-seen-separator (bool #f))
        (set-bool table-escaped (bool #f)))))

(def (table-or-element-form close-paragraph fixed-width-close otherwise)
  `(if ,table-line-predicate
       (,fixed-width-close ,close-paragraph
        (if (not (state table-open))
            ((start-node ,(table-line-table-node table-rule))
             (set-bool table-open (bool #t))) ())
        ,(table-row-form)
        (set-bool after-heading (bool #f)))
       (,table-close-form ,otherwise)))
