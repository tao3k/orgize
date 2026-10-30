;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later

;;; Native compiled source-projection tests; imports and lifecycle belong to gxtest.
(import (only-in :std/test check test-case test-suite)
        (only-in :std/error error-message)
        (only-in :clan/poo/object .ref)
        "../languages/org/modules/org-elements/source-interface.ss")
(export org-source-headlines-test)
(def org-source-headlines-test
  (test-suite "Source-backed headline projection"
    (test-case "source structure and UTF-8 identity"
      (let* ((source "#+SEQ_TODO: WAIT | DONE\n* WAIT α\n#+BEGIN_SRC text\n* fake\n#+END_SRC\n* DONE β\n")
             (view (org-source-headline-elements source))
             (records (.ref view 'elements))
             (first (car records))
             (second (cadr records)))
        (check (org-source-headline-elements? view) => #t)
        (check (length records) => 2)
        (check (.ref first 'title) => "WAIT α")
        (check (.ref first 'todo-type) => "todo")
        (check (.ref first 'byte-start) => 24)
        (check (.ref first 'byte-end) => 34)
        (check (.ref first 'identity) =>
               (string-append (.ref view 'source-sha256) ":24:34"))
        (check (.ref second 'title) => "DONE β")
        (check (.ref second 'todo-type) => "done")
        (check (.ref view 'worktree-bound?) => #f)
        (check (.ref view 'action-authority?) => #f)))
    (test-case "changed bytes invalidate source identity"
      (let ((old (org-source-headline-elements "* TODO Open\n"))
            (new (org-source-headline-elements "* DONE Open\n")))
        (check (equal? (.ref old 'source-sha256) (.ref new 'source-sha256)) => #f)
        (check (.ref (car (.ref new 'elements)) 'todo-type) => "done")))
    (test-case "inlinetasks are not ordinary headline elements"
      (let (view (org-source-headline-elements
                  "*************** TODO Inline\nBody.\n*************** END\n* TODO Next\n"))
        (check (length (.ref view 'elements)) => 1)
        (check (.ref (car (.ref view 'elements)) 'title) => "TODO Next")))
    (test-case "source resource bound"
      (check (with-catch error-message
               (lambda () (org-source-headline-elements (make-string 1048577 #\x))
                          'accepted))
             => "Org source headline projection exceeds one MiB"))))
