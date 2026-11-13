;;; Emit a Gerbil package for the Parser-produced compilation units.
(import (only-in :std/misc/ports read-all-as-string)
        (only-in :std/misc/process run-process))
(export write-fold-package fold-package-source-directory)

;; Generated sources are a native source package, never the installation prefix.
(def (fold-package-source-directory prefix)
  (path-expand "src/orgize-fold" prefix))
(def (write-source path text)
  (create-directory* (path-directory path))
  (unless (and (file-exists? path)
               (equal? text (call-with-input-file path read-all-as-string)))
    (call-with-output-file (list path: path truncate: #t) (cut display text <>))))
(def (write-fold-package output units)
  (create-directory* output)
  (for-each
    (lambda (unit)
      (write-source (path-expand (string-append (car unit) ".ss") output) (cdr unit))) units)
  ;; Compilation units are native build roots, not an invented business Module.
  (write-source (path-expand "gerbil.pkg" output) "(package: orgize-fold)\n")
  (def build (path-expand "build.ss" output))
  (write-source build
    (call-with-output-string
      (lambda (port)
        (display "#!/usr/bin/env gxi\n(import :std/build-script)\n" port)
        (write `(defbuild-script ',(map car units)) port)
        (display "\n(export main)\n" port))))
  ;; The native package manager executes build.ss directly.
  (run-process (list "chmod" "u+x" build) stdout-redirection: #f))
