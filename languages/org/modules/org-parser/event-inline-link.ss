;;; Org declares description semantics; gerbil-parser owns link recognition.
(import (only-in :gerbil-parser/src/modules/parser/interface
                 line-structure-text text-line-inline-link inline-link-opening
                 make-source-event-scope source-inline-link-initial
                 source-inline-link-scan-forms source-inline-link-open-condition)
        (only-in "../../parser.ss" org-line-structure))
(export link-open link-scan-forms inline-link-event-initial link-open-condition)
(def text-rule (line-structure-text org-line-structure))
(def link-open (inline-link-opening (text-line-inline-link text-rule)))
(def link-scope (make-source-event-scope 'org-inline-link))
(def link-open-condition (source-inline-link-open-condition link-scope))
(def inline-link-event-initial (source-inline-link-initial text-rule link-scope))
(def (link-scan-forms (nested-description? #f))
  (source-inline-link-scan-forms text-rule link-scope
    index: 'inline-byte-index cursor: 'inline-cursor
    description-helper: 'link-description-span parameters: '(inline-script-policy)
    description-node: 'OrgLinkDescription literal-description?: nested-description?
    invalid-target: 'recover-region))
