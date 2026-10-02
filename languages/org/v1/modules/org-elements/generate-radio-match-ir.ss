#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Generate the radio matcher from its Scheme POO strategy.

(import (only-in "radio-match.ss"
                 org-radio-match-strategy write-org-radio-match-ir))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-radio-match-ir.ss OUTPUT_DIR"))
(write-org-radio-match-ir
 (path-expand "org_radio_next_match.ir.json" (car (reverse arguments)))
 org-radio-match-strategy)
