;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later

(import (only-in :std/test check check-exception test-case test-suite)
        (only-in :clan/poo/object .ref)
        (only-in "../languages/org/v1/modules/org-elements/source-headlines.ss"
                 org-source-headline-elements
                 org-source-headline-elements?))

(export org-source-headlines-test)

(def org-source-headlines-test
  (test-suite "source-derived Org headline Elements"
    (test-case "file-local TODO and Rowan structure determine finite Elements"
      (let* ((source
              "#+SEQ_TODO: WAIT | DONE\n* WAIT α\n#+BEGIN_SRC text\n* fake\n#+END_SRC\n* DONE β\n")
             (view (org-source-headline-elements source))
             (records (.ref view 'elements))
             (first (car records))
             (second (cadr records)))
        (check (org-source-headline-elements? view) => #t)
        (check (length records) => 2)
        (check (.ref first 'title) => "WAIT α")
        (check (.ref first 'todo-type) => "todo")
        (check (.ref first 'byte-start) => 24)
        (check (.ref second 'title) => "DONE β")
        (check (.ref second 'todo-type) => "done")
        (check (.ref first 'identity)
               => (string-append (.ref view 'source-sha256) ":24:34"))
        (check (.ref view 'worktree-bound?) => #f)
        (check (.ref view 'action-authority?) => #f)))
    (test-case "different bytes produce a different source identity"
      (let ((old (org-source-headline-elements "* TODO Open\n"))
            (new (org-source-headline-elements "* DONE Open\n")))
        (check (equal? (.ref old 'source-sha256)
                       (.ref new 'source-sha256)) => #f)
        (check (.ref (car (.ref new 'elements)) 'todo-type) => "done")))
    (test-case "an inlinetask header is not an ordinary headline Element"
      (let (view
            (org-source-headline-elements
             "*************** TODO Inline\nBody.\n*************** END\n* TODO Next\n"))
        (check (length (.ref view 'elements)) => 1)
        (check (.ref (car (.ref view 'elements)) 'title) => "TODO Next")))
    (test-case "the source bound refuses oversized input"
      (check-exception
       (org-source-headline-elements (make-string 1048577 #\x))
       true))))
