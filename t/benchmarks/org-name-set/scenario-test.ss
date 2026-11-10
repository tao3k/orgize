(import (only-in :std/test check test-case test-suite)
        (only-in "../org-name-set.ss" name-set-fold))
(export name-set-test)

(def name-set-test
  (test-suite "Org entity name-set benchmark"
    (test-case "Exact entity names match while near matches remain text"
      (let* ((events (name-set-fold))
             (starts (filter (lambda (event) (eq? (car event) 'start)) events))
             (one-pass '(Heading Heading Heading Heading Text Text Text Heading Text Text)))
        (check (map cadr starts) => (cons 'Document (append one-pass one-pass)))))))
