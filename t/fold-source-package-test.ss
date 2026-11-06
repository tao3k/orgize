;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; Generated compilation units do not own the SDK installation prefix.
(import (only-in :std/test test-suite test-case check-equal?)
        (only-in :std/io/tempfile call-with-temporary-directory)
        (only-in :std/misc/ports read-all-as-string)
        (only-in "../languages/org/modules/org-parser/fold-package.ss"
                 write-fold-package fold-package-source-directory))
(export fold-source-package-test)

(def fold-source-package-test
  (test-suite "Native fold source ownership"
    (test-case "source generation preserves prefix authority and isolates native inputs"
      (call-with-temporary-directory "org-fold-source"
        (lambda (prefix)
          (def authority (path-expand "gerbil.pkg" prefix))
          (call-with-output-file authority
            (lambda (port) (display "(package: installation-authority)\n" port)))
          (create-directory (path-expand "pkg" prefix))
          (create-directory (path-expand "lib" prefix))
          (def source (fold-package-source-directory prefix))
          (write-fold-package source '(("probe" . "(export answer) (def answer 42)\n")))
          (check-equal? (call-with-input-file authority read)
                        '(package: installation-authority))
          (check-equal? (call-with-input-file (path-expand "gerbil.pkg" source) read)
                        '(package: orgize-fold))
          (check-equal? (list-sort string<? (directory-files source))
                        '("build.ss" "gerbil.pkg" "init.ss" "probe.ss"))
          (check-equal? (call-with-input-file (path-expand "probe.ss" source) read-all-as-string)
                        "(export answer) (def answer 42)\n")
          ;; Only the source-selection entry asks for Yuanxu authoring syntax.
          ;; Pure emitted imports/definitions use Gerbil's native default prelude.
          (check-equal? (call-with-input-file (path-expand "init.ss" source) read)
                        '(import (only-in :yuanxu/module-system/selection/init-syntax yuanxu!)))
          (check-equal? (file-exists? (path-expand "pkg" prefix)) #t)
          (check-equal? (file-exists? (path-expand "lib" prefix)) #t))))))
