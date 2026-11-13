;;; Yuanxu Module Scope publishes the existing structural validator.
(import (only-in :clan/poo/object .def .get)
        (only-in "types.ss" OrgParserScopeContract)
        (only-in "value-types.ss" org-event-strategy?))
(export OrgParser.)
(.def (OrgParser. @ (.get OrgParserScopeContract proto))
  event-strategy?: org-event-strategy?)
