;;; -*- Gerbil -*-
;;; @generated from :org-elements blocks; do not edit.
(import (only-in "../../../../languages/org/v1/modules/org-elements/interface.ss" org-element-query org-elements
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
))
