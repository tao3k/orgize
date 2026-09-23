;;; -*- Gerbil -*-
;;; Org parser declaration remains a checked POO value before AOT projection.

(import (only-in :std/test check test-case test-suite)
        (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure? line-structure-heading line-structure-blocks
                 line-structure-table table-line-delimiter
                 line-structure-list list-line-unordered-markers list-line-ordered
                 list-line-list-node list-line-item-node
                 table-line-table-node table-line-row-node table-line-cell-node
                 heading-line-section-node heading-line-heading-node
                 heading-line-fields heading-fields-title-token
                 block-line-opening block-line-closing block-line-block-node
                 block-line-unclosed block-line-heading-bound block-line-body-line
                 block-line-contents
                 block-line-header block-header-argument-token
                 line-structure-text text-line-inline-link
                 inline-link-node inline-link-target-token
                 key-value-line-marker key-value-line-node
                 key-value-line-key-token key-value-line-value-token)
        (only-in "parser.ss" org-v1-line-structure))
(export org-v1-parser-test)

(def org-v1-parser-test
  (test-suite "Org POO parser declaration"
    (test-case "Org owns sections, drawers, and greater blocks"
      (let* ((structure org-v1-line-structure)
             (heading (line-structure-heading structure))
             (blocks (line-structure-blocks structure))
             (source-block (car blocks))
             (drawer (cadr blocks))
             (table (line-structure-table structure)))
        (check (line-structure? structure) => #t)
        (check (heading-line-section-node heading) => 'OrgSection)
        (check (heading-line-heading-node heading) => 'OrgHeadline)
        (check (heading-fields-title-token (heading-line-fields heading))
               => 'HeadlineTitle)
        (check (length blocks) => 8)
        (check (map block-line-block-node blocks)
               => '(OrgSourceBlock OrgPropertyDrawer OrgQuoteBlock
                                    OrgExampleBlock OrgVerseBlock OrgCenterBlock
                                    OrgCommentBlock OrgExportBlock))
        (check (map block-line-opening (cddr blocks))
               => '("#+begin_quote" "#+begin_example" "#+begin_verse"
                                     "#+begin_center" "#+begin_comment"
                                     "#+begin_export"))
        (check (map block-line-contents (cddr blocks))
               => '(elements opaque elements elements opaque opaque))
        (check (block-header-argument-token
                (block-line-header (car (reverse blocks))))
               => 'ExportBackend)
        (check (block-line-opening source-block) => "#+begin_src")
        (check (block-line-closing source-block) => "#+end_src")
        (check (block-line-block-node source-block) => 'OrgSourceBlock)
        (check (block-line-unclosed source-block) => 'recover-as-text)
        (check (block-line-heading-bound source-block) => #t)
        (check (block-header-argument-token (block-line-header source-block))
               => 'SourceLanguage)
        (check (inline-link-node (text-line-inline-link
                                  (line-structure-text structure)))
               => 'OrgLink)
        (check (inline-link-target-token (text-line-inline-link
                                          (line-structure-text structure)))
               => 'LinkTarget)
        (check (block-line-opening drawer) => ":PROPERTIES:")
        (check (block-line-closing drawer) => ":END:")
        (check (block-line-block-node drawer) => 'OrgPropertyDrawer)
        (check (key-value-line-marker (block-line-body-line drawer)) => ":")
        (check (key-value-line-node (block-line-body-line drawer))
               => 'OrgNodeProperty)
        (check (key-value-line-key-token (block-line-body-line drawer))
               => 'PropertyKey)
        (check (key-value-line-value-token (block-line-body-line drawer))
               => 'PropertyValue)
        (check (table-line-delimiter table) => "|")
        (check (table-line-table-node table) => 'OrgTable)
        (check (table-line-row-node table) => 'OrgTableRow)
        (check (table-line-cell-node table) => 'OrgTableCell)
        (check (list-line-unordered-markers (line-structure-list structure))
               => "-+*")
        (check (list-line-ordered (line-structure-list structure)) => #t)
        (check (list-line-list-node (line-structure-list structure))
               => 'OrgPlainList)
        (check (list-line-item-node (line-structure-list structure))
               => 'OrgListItem)))))
