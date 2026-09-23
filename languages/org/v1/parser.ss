;;; -*- Gerbil -*-
;;; Org-owned POO contextual structure; gerbil-parser AOT resolves kind IDs.

(import (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 make-line-structure make-heading-line make-heading-fields
                 make-block-line make-block-header make-key-value-line
                 make-inline-link make-text-line make-table-line))
(export org-v1-line-structure)

(def org-v1-line-structure
  (make-line-structure
   (make-heading-line "*" " " 'OrgSection 'OrgHeadline 'HeadlineLine
                      (make-heading-fields 'HeadlineTitle 'HeadlineTrivia))
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
                               'PropertyKey 'PropertyValue 'PropertyTrivia))
         (make-block-line
          "#+begin_quote" "#+end_quote" #t #t
          'OrgQuoteBlock 'BlockBeginLine 'TextLine 'BlockEndLine
          'recover-as-text #t #f)
         (make-block-line
          "#+begin_example" "#+end_example" #t #t
          'OrgExampleBlock 'BlockBeginLine 'TextLine 'BlockEndLine
          'recover-as-text #t #f)
         (make-block-line
          "#+begin_verse" "#+end_verse" #t #t
          'OrgVerseBlock 'BlockBeginLine 'TextLine 'BlockEndLine
          'recover-as-text #t #f)
         (make-block-line
          "#+begin_center" "#+end_center" #t #t
          'OrgCenterBlock 'BlockBeginLine 'TextLine 'BlockEndLine
          'recover-as-text #t #f)
         (make-block-line
          "#+begin_comment" "#+end_comment" #t #t
          'OrgCommentBlock 'BlockBeginLine 'TextLine 'BlockEndLine
          'recover-as-text #t #f)
         (make-block-line
          "#+begin_export" "#+end_export" #t #t
          'OrgExportBlock 'BlockBeginLine 'TextLine 'BlockEndLine
          'recover-as-text #t #f
          (make-block-header 'ExportBackend 'BlockHeaderTrivia)))
   (make-text-line 'OrgTextLine 'TextLine
                   (make-inline-link "[[" "][" "]]" 'OrgLink
                                     'LinkTarget 'LinkDescription 'LinkTrivia)
                   'OrgParagraph)
   (make-table-line "|" 'OrgTable 'OrgTableRow 'OrgTableRuleRow
                    'OrgTableCell 'TableSeparator 'TableCellText
                    'TableTrivia 'TableRuleText)))
