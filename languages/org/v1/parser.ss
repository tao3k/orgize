;;; -*- Gerbil -*-
;;; Org-owned POO contextual structure; gerbil-parser AOT resolves kind IDs.

(import (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 make-line-structure make-heading-line
                 make-block-line make-block-header make-key-value-line
                 make-inline-link make-text-line))
(export org-v1-line-structure)

(def org-v1-line-structure
  (make-line-structure
   (make-heading-line "*" " " 'OrgSection 'OrgHeadline 'HeadlineLine)
   (list (make-block-line
          "#+begin_src" "#+end_src" #t #t
          'OrgSourceBlock 'BlockBeginLine 'TextLine 'BlockEndLine
          'recover-as-text #t #f
          (make-block-header 'SourceLanguage 'BlockHeaderTrivia))
         (make-block-line
          ":PROPERTIES:" ":END:" #t #t
          'OrgPropertyDrawer 'DrawerBeginLine 'TextLine 'DrawerEndLine
          'recover-as-text #t
          (make-key-value-line ":" 'OrgNodeProperty
                               'PropertyKey 'PropertyValue 'PropertyTrivia)))
   (make-text-line 'OrgTextLine 'TextLine
                   (make-inline-link "[[" "][" "]]" 'OrgLink
                                     'LinkTarget 'LinkDescription 'LinkTrivia))))
