;;; -*- Gerbil -*-
;;; Domain-specific hygienic checks for Org Element catalog and graph queries.

(import (only-in :std/test check)
        (only-in :std/list/list every)
        (only-in "config.ss"
                 +org-element-kinds+ +org-greater-element-kinds+
                 +org-object-kinds+ +org-recursive-object-kinds+
                 +org-affiliated-keywords+ +org-object-restrictions+
                 +org-secondary-values+ org-object-allowed?
                 org-secondary-value?)
        (only-in "funs.ss" org-element-select))
(export check-org-element-catalog check-org-element-selection)

(def (catalog-unique? values)
  (let loop ((remaining values) (seen '()))
    (if (null? remaining) #t
        (and (not (member (car remaining) seen))
             (loop (cdr remaining) (cons (car remaining) seen))))))

(def (known-kind? kind)
  (if (or (equal? kind "org-data")
          (member kind +org-element-kinds+)
          (member kind +org-object-kinds+))
    #t #f))

(defsyntax (check-org-element-catalog stx)
  (syntax-case stx ()
    ((_)
     (syntax
      (begin
        (check (every catalog-unique?
                      (list +org-element-kinds+
                            +org-greater-element-kinds+
                            +org-object-kinds+
                            +org-recursive-object-kinds+
                            +org-affiliated-keywords+
                            (map car +org-object-restrictions+))) => #t)
        (check (every known-kind? +org-greater-element-kinds+) => #t)
        (check (every (lambda (kind)
                        (if (member kind +org-object-kinds+) #t #f))
                      +org-recursive-object-kinds+) => #t)
        (check (every (lambda (entry)
                        (and (known-kind? (car entry))
                             (catalog-unique? (cdr entry))
                             (every (lambda (kind)
                                      (if (member kind +org-object-kinds+)
                                        #t #f))
                                    (cdr entry))))
                      +org-object-restrictions+) => #t)
        (check (every (lambda (entry)
                        (if (assoc (car entry) +org-object-restrictions+)
                          #t #f))
                      +org-secondary-values+) => #t)
        (check (org-object-allowed? "paragraph" "link") => #t)
        (check (org-object-allowed? "table-row" "table-cell") => #t)
        (check (equal? (cdr (assoc "table-row" +org-object-restrictions+))
                       '("table-cell")) => #t)
        (check (org-object-allowed? "table-row" "link") => #f)
        (check (org-object-allowed? "headline" "line-break") => #f)
        (check (org-object-allowed? "src-block" "bold") => #f)
        (check (org-secondary-value? "headline" "title") => #t)
        (check (org-secondary-value? "paragraph" "title") => #f))))))

(defsyntax (check-org-element-selection stx)
  (syntax-case stx ()
    ((_ query context scope targets id-of expected-ids)
     (syntax
      (check (map id-of (org-element-select query context scope targets))
             => expected-ids)))))
