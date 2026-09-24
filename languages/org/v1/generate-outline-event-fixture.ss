#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Serialize the executable Scheme outline events for Rowan parity tests.

(import (only-in :std/encoding/json write-json)
        (only-in "outline-event-fixture.ss" outline-event-fixture))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-outline-event-fixture.ss OUTPUT.json"))
(call-with-output-file (car (reverse arguments))
  (lambda (port) (write-json port (outline-event-fixture))))
