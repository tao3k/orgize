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

(assert-unique 'object-restriction-owners
               (map car +org-object-restrictions+))
(for-each
 (lambda (entry)
   (let ((owner (car entry)) (objects (cdr entry)))
     (unless (or (member owner +org-element-kinds+)
                 (member owner +org-object-kinds+))
       (error "unknown object restriction owner" owner))
     (assert-unique owner objects)
     (for-each
      (lambda (object)
        (unless (member object +org-object-kinds+)
          (error "unknown restricted object" owner object)))
      objects)))
 +org-object-restrictions+)

(for-each
 (lambda (entry)
   (unless (assoc (car entry) +org-object-restrictions+)
     (error "secondary value lacks an object restriction" (car entry))))
 +org-secondary-values+)

(when (member "line-break" (cdr (assoc "headline" +org-object-restrictions+)))
  (error "headline title must exclude line breaks"))
(unless (equal? (cdr (assoc "table-row" +org-object-restrictions+))
                '("table-cell"))
  (error "table-row may only contain table-cell objects"))

(unless (and (org-object-allowed? "paragraph" "link")
             (org-object-allowed? "table-row" "table-cell")
             (not (org-object-allowed? "table-row" "link"))
             (not (org-object-allowed? "headline" "line-break"))
             (not (org-object-allowed? "src-block" "bold"))
             (org-secondary-value? "headline" "title")
             (not (org-secondary-value? "paragraph" "title")))
  (error "Org object containment or secondary-value contract mismatch"))

(displayln "Org element inventory contracts OK")
