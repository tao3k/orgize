;;; -*- Gerbil -*-
;;; @generated from :org-elements blocks; do not edit.
(import (only-in "../interface.ss" org-element-query org-elements
property property-contains at child-of descendant-of))
(export org-element-queries)
(def org-element-queries (list
  (org-element-query "tasks.open"
(org-elements headline (property todo-type "todo")
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
