;;; -*- Gerbil -*-
;;; Org binds domain content; the engine compiles paragraph lifetime.

(import (only-in :gerbil-parser/src/modules/parser/interface
                 line-structure-text make-source-event-scope source-paragraph-initial
                 source-paragraph-close-form source-paragraph-line-form
                 source-paragraph-open-condition source-paragraph-span-forms
                 source-paragraph-blank-forms)
        (only-in "../../parser.ss" org-line-structure)
        (only-in "event-inline.ss"
                 event-inline-initial nested-description-event-initial
                 citation-affix-event-initial
                 event-text-line-forms)
        (only-in "event-inline-citation-reference.ss"
                 citation-reference-helper)
        (only-in "event-inline-timestamp.ss" timestamp-candidate-helper)
        (only-in "objects.ss" make-org-event-helper))
(export paragraph-event-initial paragraph-close-form paragraph-finish-form
        paragraph-line-form
        paragraph-event-helpers paragraph-open-condition
        paragraph-span-forms paragraph-blank-forms)

(def paragraph-rule (line-structure-text org-line-structure))
(def paragraph-scope (make-source-event-scope 'org-paragraph))
(def paragraph-event-initial (source-paragraph-initial paragraph-rule paragraph-scope))
(def paragraph-open-condition (source-paragraph-open-condition paragraph-scope))
(def (paragraph-span-forms from until)
  (source-paragraph-span-forms paragraph-rule paragraph-scope from until))
(def (paragraph-blank-forms until)
  (source-paragraph-blank-forms paragraph-scope until))
(def paragraph-close-form
  (source-paragraph-close-form paragraph-rule paragraph-scope 'inline-span '(inline-script-policy)))
(def paragraph-finish-form
  (source-paragraph-close-form paragraph-rule paragraph-scope 'inline-span '(inline-script-policy) #f))
(def (paragraph-line-form)
  (source-paragraph-line-form paragraph-rule paragraph-scope 'inline-span '(inline-script-policy)))

(def paragraph-event-helpers
  (list (make-org-event-helper
         'inline-span event-inline-initial (event-text-line-forms 'start)
         '(inline-script-policy))
        (make-org-event-helper
         'link-description-span nested-description-event-initial
         (event-text-line-forms 'start #t)
         '(inline-script-policy))
        (make-org-event-helper
         'citation-affix-span citation-affix-event-initial
         (event-text-line-forms 'start #t #f)
         '(inline-script-policy))
        citation-reference-helper
        timestamp-candidate-helper))
