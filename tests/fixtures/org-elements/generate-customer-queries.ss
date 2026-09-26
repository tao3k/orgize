#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Consumer-pack fixture: the same public Org Elements AOT API.

(import (only-in "../../../languages/org/v1/modules/org-elements/aot.ss"
                 generate-org-element-query-rust-module)
        (only-in "generated/customer-query-source.ss" org-element-queries))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-customer-queries.ss OUTPUT.rs"))

(generate-org-element-query-rust-module
 (car (reverse arguments)) org-element-queries)
