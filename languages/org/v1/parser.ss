;;; -*- Gerbil -*-
;;; Org-owned POO contextual structure; gerbil-parser AOT resolves kind IDs.

(import (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 make-line-structure make-heading-line
                 make-block-line make-text-line))
(export org-v1-line-structure)

(def org-v1-line-structure
  (make-line-structure
   (make-heading-line "*" " " 'OrgSection 'OrgHeadline 'HeadlineLine)
   (list (make-block-line
          "#+begin_src" "#+end_src" #t #t
          'OrgSourceBlock 'BlockBeginLine 'TextLine 'BlockEndLine))
   (make-text-line 'OrgTextLine 'TextLine)))
