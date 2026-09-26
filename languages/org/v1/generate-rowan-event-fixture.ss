#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Serialize the executable Scheme outline events for Rowan parity tests.

(import (only-in :std/encoding/json write-json)
        (only-in "rowan-event-fixture.ss" rowan-event-fixture))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-rowan-event-fixture.ss OUTPUT.json"))
(call-with-output-file (car (reverse arguments))
  (lambda (port) (write-json port (rowan-event-fixture))))
