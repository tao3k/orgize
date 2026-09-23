;;; -*- Gerbil -*-
;;; Org parser declaration remains a checked POO value before AOT projection.

(import (only-in :std/test check test-case test-suite)
        (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure? line-structure-heading line-structure-blocks
                 heading-line-section-node heading-line-heading-node
                 block-line-opening block-line-closing block-line-block-node)
        (only-in "parser.ss" org-v1-line-structure))
(export org-v1-parser-test)

(def org-v1-parser-test
  (test-suite "Org POO parser declaration"
    (test-case "Org owns the section and source-block rules"
      (let* ((structure org-v1-line-structure)
             (heading (line-structure-heading structure))
             (blocks (line-structure-blocks structure))
             (source-block (car blocks)))
        (check (line-structure? structure) => #t)
        (check (heading-line-section-node heading) => 'OrgSection)
        (check (heading-line-heading-node heading) => 'OrgHeadline)
        (check (length blocks) => 1)
        (check (block-line-opening source-block) => "#+begin_src")
        (check (block-line-closing source-block) => "#+end_src")
        (check (block-line-block-node source-block) => 'OrgSourceBlock)))))
