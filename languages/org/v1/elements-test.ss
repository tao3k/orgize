;;; -*- Gerbil -*-
;;; Scheme-owned element inventory admission contracts.

(load "languages/org/v1/elements.ss")

(def (assert-unique label values)
  (let loop ((remaining values) (seen '()))
    (unless (null? remaining)
      (when (member (car remaining) seen)
        (error "duplicate Org declaration" label (car remaining)))
      (loop (cdr remaining) (cons (car remaining) seen)))))

(for-each
 (lambda (entry) (assert-unique (car entry) (cdr entry)))
 (list (cons 'elements +org-element-kinds+)
       (cons 'greater-elements +org-greater-element-kinds+)
       (cons 'objects +org-object-kinds+)
       (cons 'recursive-objects +org-recursive-object-kinds+)
       (cons 'affiliated-keywords +org-affiliated-keywords+)))

(for-each
 (lambda (kind)
   (unless (or (equal? kind "org-data")
               (member kind +org-element-kinds+))
     (error "greater element not declared as an element" kind)))
 +org-greater-element-kinds+)

(for-each
 (lambda (kind)
   (unless (member kind +org-object-kinds+)
     (error "recursive object not declared as an object" kind)))
 +org-recursive-object-kinds+)

(displayln "Org element inventory contracts OK")
