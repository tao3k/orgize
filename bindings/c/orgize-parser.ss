;;; -*- Gerbil -*-
;;; One export in the existing linked Gerbil program, not another runtime.
(import (only-in "orgize-native.ss" orgize-c-round-trip)
        (only-in "orgize-scheme-runtime.ss" orgize-scheme-runtime-link-anchor)
        (only-in :gerbil-scheme-rust/scheme/native gerbil-rs-root-bytevector)
        (only-in "../../languages/org/native-event-tape.ss" org-request->tape))
(export main contract-link-anchor runtime-link-anchor)
(def contract-link-anchor orgize-c-round-trip)
(def runtime-link-anchor orgize-scheme-runtime-link-anchor)
(def (main) (void))

(begin-foreign
  (namespace ("orgize/bindings/c/orgize-parser#" copy-request orgize-parse-events))
  (c-declare "#include <string.h>")
  (define copy-request
    (c-lambda ((pointer unsigned-int8) unsigned-int64 scheme-object) int
      "if (___arg2 > ___HD_BYTES(___HEADER(___arg3)) || (___arg2 && !___arg1)) { ___return(0); } if (___arg2) memcpy(___CAST(void*, ___BODY_AS(___arg3, ___tSUBTYPED)), ___arg1, ___arg2); ___return(1);"))
  (c-define (orgize-parse-events input length)
    ((pointer unsigned-int8) unsigned-int64) int64
    "orgize_parse_events" "extern"
    (with-exception-catcher
     (lambda (exception) 0)
     (lambda ()
       (let ((bytes (make-u8vector length)))
         (if (= (orgize/bindings/c/orgize-parser#copy-request input length bytes) 1)
           (gerbil-scheme-rust/scheme/native#gerbil-rs-root-bytevector
            (orgize/languages/org/native-event-tape#org-request->tape bytes))
           0))))))
