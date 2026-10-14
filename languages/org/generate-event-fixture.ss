#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Generate a native Scheme event golden, never a JSON semantic transport.

(import (only-in "event-fixture.ss" event-fixture))

(export main)

(def (main output)
  (call-with-output-file output
    (lambda (port)
      (display ";;; -*- Gerbil -*-\n;;; Native Scheme event golden.\n" port)
      (write '(export event-fixture-events) port)
      (newline port)
      (write (list 'def 'event-fixture-events
                   (list 'quote (event-fixture))) port)
      (newline port))))
