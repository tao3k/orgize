;;; -*- Gerbil -*-
;;; Native Scheme affiliated keyword membership.

(import (only-in :gerbil-parser/text-support
                 ascii-ci=? string-before))
(export org-affiliated-keyword?)

(def (org-affiliated-keyword? key names)
  (or (ascii-ci=? (string-before key "_") "ATTR")
      (ormap (lambda (name) (ascii-ci=? key name)) names)))
