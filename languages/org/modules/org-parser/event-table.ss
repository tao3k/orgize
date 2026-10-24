;;; -*- Gerbil -*-
;;; Org table event policy projected from the declared line structure.

(import (only-in :gerbil-parser/src/modules/parser/interface
                 source-table-row-initial source-table-row-forms make-source-event-scope
                 line-structure-table table-line-delimiter
                 table-line-table-node)
        (only-in "../../parser.ss" org-line-structure))
(import (only-in "event-table-formula.ss" table-formula-marker)
        (only-in "event-source-content.ss" source-content-initial source-content-forms)
        (only-in "objects.ss" make-org-event-helper))
(export table-event-initial table-close-form table-or-element-form table-content-helper)

(def table-rule (line-structure-table org-line-structure))
(def table-scope (make-source-event-scope 'org-table-row))
(def table-byte
  (char->integer (string-ref (table-line-delimiter table-rule) 0)))
(def table-content-end '(line-content-end))
(def table-indent '(line-skip-horizontal start))
(def table-line-predicate `(line-byte-equal? ,table-indent ,table-byte))
(def table-el-border?
  `(and (line-byte-equal? ,table-indent 43)
        (line-bytes-all-in? ,table-indent ,table-content-end
                            (43 45 9 32))
        (line-bytes-any-in? ,table-indent ,table-content-end (45))))

(def table-event-initial
  (append '((table-open #f) (table-el-open #f))
          (source-table-row-initial table-rule table-scope)))

(def table-close-form
  '(if (state table-open)
       ((finish-node) (set-bool table-open (bool #f)))
       ((if (state table-el-open)
            ((finish-node) (set-bool table-el-open (bool #f))) ()))))

(def table-content-helper
  (make-org-event-helper
   'table-cell-content (append source-content-initial '((inline-script-policy 2)))
   (source-content-forms
    'TableCellContent 'TableTrivia
    (lambda (from until)
      `((call-source-helper inline-span ,from ,until
                             ((state inline-script-policy))))))
   '(inline-script-policy)))

(def (table-or-element-form close-paragraph fixed-width-close otherwise)
  `(if (or ,table-el-border?
           (and (state table-el-open) ,table-line-predicate))
       (,fixed-width-close ,close-paragraph
        (if (not (state table-el-open))
            ((if (state table-open)
                 ((finish-node) (set-bool table-open (bool #f))) ())
             (start-node OrgTableEl)
             (set-bool table-el-open (bool #t))) ())
        (token TableElLine start end)
        (set-bool after-heading (bool #f)))
       ((if ,table-line-predicate
            (,fixed-width-close ,close-paragraph
             (if (not (state table-open))
                 ((start-node ,(table-line-table-node table-rule))
                  (set-bool table-open (bool #t))) ())
             ,@(source-table-row-forms table-rule table-scope 'table-cell-content
                                       '(inline-script-policy)
                                       rule-bytes: '(43 45 58 9 32)
                                       rule-marker: 45 escape-byte: 92)
             (set-bool after-heading (bool #f)))
            ((if (and (state table-open)
                      (line-starts-with-ascii-ci ,table-formula-marker))
                 () (,table-close-form))
             ,otherwise)))))
