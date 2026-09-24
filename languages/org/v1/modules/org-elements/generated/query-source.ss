;;; -*- Gerbil -*-
;;; @generated from :org-elements blocks; do not edit.
(import (only-in "../interface.ss" org-element-query org-elements
property property-contains all-of any-of at child-of descendant-of))
(export org-element-queries)
(def org-element-queries (list
  (org-element-query "tasks.open"
(org-elements headline (property todo-type "todo")
              (descendant-of scope))
  )
  (org-element-query "tasks.review-or-audit"
(org-elements headline
              (all-of (property todo-type "todo")
                      (any-of (property-contains source-title "Review")
                              (property-contains source-title "Audit")))
              (descendant-of scope))
  )
  (org-element-query "tasks.done"
(org-elements headline (property todo-type "done")
              (descendant-of scope))
  )
  (org-element-query "headlines.child"
(org-elements headline (property source-title "DONE Child :work:"))
  )
))
