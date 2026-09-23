;;; -*- Gerbil -*-
;;; Org parser declaration remains a checked POO value before AOT projection.

(import (only-in :std/test check test-case test-suite)
        (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure? line-structure-heading line-structure-blocks
                 heading-line-section-node heading-line-heading-node
                 block-line-opening block-line-closing block-line-block-node
                 block-line-unclosed block-line-heading-bound block-line-body-line
                 block-line-header block-header-argument-token
                 line-structure-text text-line-inline-link
                 inline-link-node inline-link-target-token
                 key-value-line-marker key-value-line-node
                 key-value-line-key-token key-value-line-value-token)
        (only-in "parser.ss" org-v1-line-structure))
(export org-v1-parser-test)

(def org-v1-parser-test
  (test-suite "Org POO parser declaration"
    (test-case "Org owns sections, source blocks, and property drawers"
      (let* ((structure org-v1-line-structure)
             (heading (line-structure-heading structure))
             (blocks (line-structure-blocks structure))
             (source-block (car blocks))
             (drawer (cadr blocks)))
        (check (line-structure? structure) => #t)
        (check (heading-line-section-node heading) => 'OrgSection)
        (check (heading-line-heading-node heading) => 'OrgHeadline)
        (check (length blocks) => 2)
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
               => 'PropertyValue)))))
