#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Development-only AOT projection of the Scheme Org element inventory.

(load "languages/org/v1/elements.ss")

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

(def (emit-elements port)
  (display "// Generated from languages/org/v1/elements.ss. Do not edit.\n\n" port)
  (emit-list port "ORG_ELEMENT_KINDS" +org-element-kinds+)
  (newline port)
  (emit-list port "ORG_GREATER_ELEMENT_KINDS" +org-greater-element-kinds+)
  (newline port)
  (emit-list port "ORG_OBJECT_KINDS" +org-object-kinds+)
  (newline port)
  (emit-list port "ORG_RECURSIVE_OBJECT_KINDS" +org-recursive-object-kinds+)
  (newline port)
  (emit-list port "ORG_AFFILIATED_KEYWORDS" +org-affiliated-keywords+))

(def arguments (command-line))
(unless (>= (length arguments) 3)
  (error "usage: gxi languages/org/v1/generate-elements.ss OUTPUT"))
(call-with-output-file (car (reverse arguments))
  (lambda (port) (emit-elements port)))
