;;; -*- Gerbil -*-
;;; Scheme-owned Babel CALL target span, shared across keyword contexts.

(import (only-in "objects.ss" make-org-event-helper))
(export event-babel-call-helper event-babel-call-forms)

(def babel-paren-end '(line-scan-nonspace-until start "("))
(def babel-bracket-end '(line-scan-nonspace-until start "["))

(def (event-babel-call-forms from until)
  `((call-source-helper babel-call-value ,from ,until)))

(def event-babel-call-helper
  (make-org-event-helper
   'babel-call-value
   '()
   `((if (offset-less? ,babel-bracket-end ,babel-paren-end)
         ((token BabelCallName start ,babel-bracket-end)
          (token KeywordValue ,babel-bracket-end end))
         ((token BabelCallName start ,babel-paren-end)
          (token KeywordValue ,babel-paren-end end))))))
