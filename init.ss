;;; Org capabilities and native leaves share Yuanxu's selected source graph.
(yuanxu!
  :load-modules "./languages/org/modules" mode: explicit rename: Org
  (org-parser) (org-elements) (org-contract)
  :load-modules "./bindings" mode: explicit rename: Native
  (c))
