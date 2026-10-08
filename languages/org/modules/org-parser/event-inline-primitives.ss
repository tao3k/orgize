;;; -*- Gerbil -*-
;;; Org-owned inline cursor names; generic pattern projections belong to the engine.

(export link-index inline-next)

(def link-index '(line-index inline-byte-index))
(def inline-next `(line-step ,link-index))
