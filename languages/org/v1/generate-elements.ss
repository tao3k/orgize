#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Development-only AOT projection of the Scheme Org element inventory.

(load "languages/org/v1/modules/org-elements/catalog.ss")
(import (only-in :std/crypto/digest sha256)
        (only-in :std/misc/ports read-all-as-u8vector))

(def (elements-digest)
  (let ((digest (sha256
                 (call-with-input-file "languages/org/v1/modules/org-elements/catalog.ss"
                   read-all-as-u8vector)))
        (digits "0123456789abcdef"))
    (string-append
     "sha256:"
     (list->string
      (apply append
             (map (lambda (byte)
                    (list (string-ref digits (quotient byte 16))
                          (string-ref digits (modulo byte 16))))
                  (u8vector->list digest)))))))

(def (emit-list port name values)
  (display "#[rustfmt::skip]\n" port)
  (display "pub const " port)
  (display name port)
  (display ": &[&str] = &[\n" port)
  (for-each
   (lambda (value)
     (display "    " port)
     (write value port)
     (display ",\n" port))
   values)
  (display "];\n" port))

(def (emit-relations port name entries)
  (display "#[rustfmt::skip]\n" port)
  (display "pub const " port)
  (display name port)
  (display ": &[(&str, &[&str])] = &[\n" port)
  (for-each
   (lambda (entry)
     (display "    (" port)
     (write (car entry) port)
     (display ", &[" port)
     (for-each
      (lambda (value)
        (write value port)
        (display ", " port))
      (cdr entry))
     (display "]),\n" port))
   entries)
  (display "];\n" port))

(def (emit-elements port)
  (display "// Generated from languages/org/v1/modules/org-elements/catalog.ss. Do not edit.\n\n" port)
  (display "pub const ELEMENTS_DIGEST: &str =\n    " port)
  (write (elements-digest) port)
  (display ";\n\n" port)
  (emit-list port "ORG_ELEMENT_KINDS" +org-element-kinds+)
  (newline port)
  (emit-list port "ORG_GREATER_ELEMENT_KINDS" +org-greater-element-kinds+)
  (newline port)
  (emit-list port "ORG_OBJECT_KINDS" +org-object-kinds+)
  (newline port)
  (emit-list port "ORG_RECURSIVE_OBJECT_KINDS" +org-recursive-object-kinds+)
  (newline port)
  (emit-list port "ORG_AFFILIATED_KEYWORDS" +org-affiliated-keywords+)
  (newline port)
  (emit-relations port "ORG_OBJECT_RESTRICTIONS" +org-object-restrictions+)
  (newline port)
  (emit-relations port "ORG_SECONDARY_VALUES" +org-secondary-values+))

(def arguments (command-line))
(unless (>= (length arguments) 3)
  (error "usage: gxi languages/org/v1/generate-elements.ss OUTPUT"))
(call-with-output-file (car (reverse arguments))
  (lambda (port) (emit-elements port)))
