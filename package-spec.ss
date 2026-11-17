;;; Orgize's package build projections come from its Module Build owners.
(import (for-syntax
          (only-in :std/source this-source-file)
          (only-in :tao3k/yuanxu/module-system/building/source-package register-module-build-source!)))
(begin-syntax
  (register-module-build-source! (path-directory (this-source-file)) 'orgize))

(import (only-in :std/source this-source-file)
        (only-in :clan/poo/object .o .+)
        (only-in :tao3k/yuanxu/module-system/building/source-package register-module-build-source!)
        (rename-in (only-in "./languages/org/modules/org-parser/build.ss" Build) (Build FoldBuild))
        (rename-in (only-in "./bindings/c/build.ss" Build) (Build ProgramBuild))
        (only-in "./init.ss" orgize.init.exports))
(export orgize.package.exports)

(register-module-build-source! (path-directory (this-source-file)) 'orgize)

(def orgize.package.exports
  (.o (:: @ orgize.init.exports)
      build-actions: (.+ FoldBuild ProgramBuild)))
