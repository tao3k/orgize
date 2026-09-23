;;; -*- Gerbil -*-
;;; Maintained Org Element catalog and projection are the module's authority.

(import (only-in :clan/poo/object .o)
        (only-in :gerbil-parser/src/modules/parser/graph-projection-objects
                 graph-projection?)
        (only-in "types.ss"
                 +org-element-schema+ +org-element-profile-kind+
                 org-elements-profile?)
        (only-in "catalog.ss"
                 +org-element-kinds+ +org-greater-element-kinds+
                 +org-object-kinds+ +org-recursive-object-kinds+
                 +org-affiliated-keywords+ +org-object-restrictions+
                 +org-secondary-values+ org-object-allowed?
                 org-secondary-value?)
        (only-in "../../graph.ss" org-v1-graph-projection))
(export OrgElementsProfile. org-elements-default-profile
        +org-element-kinds+ +org-greater-element-kinds+
        +org-object-kinds+ +org-recursive-object-kinds+
        +org-affiliated-keywords+ +org-object-restrictions+
        +org-secondary-values+ org-object-allowed?
        org-secondary-value?)

(def OrgElementsProfile.
  (.o kind: +org-element-profile-kind+
      schema: +org-element-schema+
      inventory: +org-element-kinds+
      projection: org-v1-graph-projection))

(def org-elements-default-profile
  (begin
    (unless (and (graph-projection? org-v1-graph-projection)
                 (org-elements-profile? OrgElementsProfile.))
      (error "invalid Org Elements profile"))
    OrgElementsProfile.))
