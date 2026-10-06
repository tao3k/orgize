#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Consumer-pack fixture: the same public Org Elements AOT API.

(import (only-in "../../../languages/org/v1/modules/org-elements/aot.ss"
                 generate-org-element-query-rust-module)
        (only-in "generated/customer-query-source.ss" org-element-queries))

(export main)

(def (main output-path)
  (generate-org-element-query-rust-module output-path org-element-queries))
