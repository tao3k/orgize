#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Generate a native Scheme event golden, never a JSON semantic transport.

(import (only-in "native-event-fixture.ss" native-event-fixture))

(export main)

(def (main output)
  (call-with-output-file output
    (lambda (port)
      (display ";;; -*- Gerbil -*-\n;;; Native Scheme event golden.\n" port)
      (write '(export native-event-fixture-events) port)
      (newline port)
      (write (list 'def 'native-event-fixture-events
                   (list 'quote (native-event-fixture))) port)
      (newline port))))
