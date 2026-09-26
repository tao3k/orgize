#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Developer AOT entry; Cargo consumers compile the committed Rust artifact.

(import (only-in :gerbil-parser/src/compiler/event-strategy-aot
                 generate-line-event-module)
        (only-in "line-event-parser.ss" parse_org_line_events))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-line-events.ss OUTPUT.rs"))

(generate-line-event-module (car (reverse arguments)) parse_org_line_events)
