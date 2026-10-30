;;; The Org inventory is already expressed in Parser's admitted native family.
(import (only-in :gerbil-parser/graph-projection-support make-graph-projection)
        (only-in "graph-shape.ss" org-graph-shape))
(export org-graph-projection)
(def org-graph-projection (make-graph-projection org-graph-shape))
