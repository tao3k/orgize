#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Generate a native Scheme event golden, never a JSON semantic transport.

(import (only-in "rowan-event-fixture.ss" rowan-event-fixture))

(export main)

(def (main output)
  (call-with-output-file output
    (lambda (port)
      (display ";;; -*- Gerbil -*-\n;;; Native Scheme event golden.\n" port)
      (write '(export rowan-event-fixture-events) port)
      (newline port)
      (write (list 'def 'rowan-event-fixture-events
                   (list 'quote (rowan-event-fixture))) port)
      (newline port))))
