;;; -*- Gerbil -*-
;;; @generated from :org-elements-query blocks; do not edit.
(import (only-in "../../../../languages/org/modules/org-elements/interface.ss" org-element-query org-elements
property property-contains all-of any-of at child-of descendant-of))
(export org-element-queries)
(def org-element-queries (list
  (org-element-query "customer.active-review"
(org-elements headline
              (all-of (property todo-type "todo")
                      (property-contains source-title "Review"))
              (descendant-of scope))
  )
  (org-element-query "customer.cited-evidence"
(org-elements citation-reference (property key "doe2020"))
  )
  (org-element-query "customer.normalized-title"
(org-elements headline (property title "Review patch"))
  )
  (org-element-query "customer.priority"
(org-elements headline (property priority "A"))
  )
  (org-element-query "customer.tagged"
(org-elements headline (property tags "work"))
  )
  (org-element-query "customer.raw-value"
(org-elements headline (property raw-value "WAIT [#A] Review patch :work:"))
  )
  (org-element-query "customer.todo-contains"
(org-elements headline
              (all-of (property-contains todo-keyword "WAI")
                      (property-contains title "Review")))
  )
))
