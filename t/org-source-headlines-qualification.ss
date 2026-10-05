;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later

;;; A separate process keeps the source parser and Rust-pure headline module
;;; out of the legacy all-suites gxtest image. Each marker follows a real check.
(import (only-in :clan/poo/object .ref)
        :orgize/languages/org/v1/modules/org-elements/source-interface)

(def (require-equal name actual expected)
  (unless (equal? actual expected)
    (error "Org source headline qualification mismatch" name actual expected)))

(def (case-ok name)
  (display "SOURCE-HEADLINE-CASE-OK ")
  (display name)
  (newline)
  (force-output))

(let* ((source
        "#+SEQ_TODO: WAIT | DONE\n* WAIT α\n#+BEGIN_SRC text\n* fake\n#+END_SRC\n* DONE β\n")
       (view (org-source-headline-elements source))
       (records (.ref view 'elements))
       (first (car records))
       (second (cadr records)))
  (require-equal 'view (org-source-headline-elements? view) #t)
  (require-equal 'count (length records) 2)
  (require-equal 'first-title (.ref first 'title) "WAIT α")
  (require-equal 'first-state (.ref first 'todo-type) "todo")
  (require-equal 'first-start (.ref first 'byte-start) 24)
  (require-equal 'first-end (.ref first 'byte-end) 34)
  (require-equal
   'first-id (.ref first 'identity)
   (string-append (.ref view 'source-sha256) ":24:34"))
  (require-equal 'second-title (.ref second 'title) "DONE β")
  (require-equal 'second-state (.ref second 'todo-type) "done")
  (require-equal 'worktree-bound (.ref view 'worktree-bound?) #f)
  (require-equal 'action-authority (.ref view 'action-authority?) #f)
  (case-ok 'source-structure))

(let ((old (org-source-headline-elements "* TODO Open\n"))
      (new (org-source-headline-elements "* DONE Open\n")))
  (when (equal? (.ref old 'source-sha256) (.ref new 'source-sha256))
    (error "changed source retained its identity"))
  (require-equal 'changed-state
                 (.ref (car (.ref new 'elements)) 'todo-type) "done")
  (case-ok 'changed-bytes))

(let (view
      (org-source-headline-elements
       "*************** TODO Inline\nBody.\n*************** END\n* TODO Next\n"))
  (require-equal 'inlinetask-exclusion (length (.ref view 'elements)) 1)
  (require-equal 'remaining-title
                 (.ref (car (.ref view 'elements)) 'title) "TODO Next")
  (case-ok 'inlinetask-exclusion))

(require-equal
 'source-bound
 (with-catch
  (lambda (error-value) (error-message error-value))
  (lambda ()
    (org-source-headline-elements (make-string 1048577 #\x))
    'accepted))
 "Org source headline projection exceeds one MiB")
(case-ok 'source-bound)

(display "SOURCE-HEADLINE-OK")
(newline)
(force-output)
