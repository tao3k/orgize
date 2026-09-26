#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Generate typed Rust event IR from Org's executable Scheme algorithm.

(import (only-in "rowan-event-parser.ss" parse_org_rowan_events))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-rowan-event-ir.ss OUTPUT.json"))
(call-with-output-file (car (reverse arguments))
  (lambda (port) (write-string parse_org_rowan_events port)))
