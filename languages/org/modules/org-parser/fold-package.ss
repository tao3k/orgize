;;; Emit a native Yuanxu package for the Parser-produced compilation units.
;;; This file owns text emission only; selection, products and cleanup are Core's.
(import (only-in :std/misc/ports read-all-as-string))
(export write-fold-package)
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
  (write-source (path-expand "gerbil.pkg" output) "(package: orgize-fold prelude: :tao3k/yuanxu/prelude)\n")
  (write-source (path-expand "init.ss" output)
    "(yuanxu! :load-modules \".\" mode: explicit rename: Generated)\n")
  (write-source (path-expand "build.ss" output)
    (call-with-output-string
      (lambda (port)
        (display "(import (for-syntax (only-in :std/source this-source-file) (only-in :yuanxu/module-system/building/source-package register-module-build-source!)))\n(begin-syntax (register-module-build-source! (path-directory (this-source-file)) 'orgize-fold))\n(import (only-in :yuanxu/module-system/building/script defbuild-script) (only-in :clan/poo/object .get) (only-in \"./init.ss\" orgize-fold.init.exports))\n" port)
        (write `(defbuild-script ((.get orgize-fold.init.exports std-make-spec) ',(map car units))) port)
        (display "\n(export main)\n" port)))))
