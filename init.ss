;;; Org capabilities and native leaves share Yuanxu's selected source graph.
(import (only-in :tao3k/yuanxu/module-system/selection/init-syntax yuanxu!))
(yuanxu!
  :load-modules "./languages/org/modules" mode: explicit rename: Org
  (org-parser) (org-elements) (org-contract)
  :load-modules "./bindings" mode: explicit rename: Native
  (c))
