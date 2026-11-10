#!/usr/bin/env gxi
;;; Opt-in benchmark compilation. The package build never reads this spec.
(import (only-in :std/build-script defbuild-script)
        (only-in :std/source this-source-file))
(export main)

(def benchmark-root
  (path-expand "target/scheme-benchmarks" (path-directory (this-source-file))))

(defbuild-script
  '("t/org-parser-test-support"
    "t/benchmarks/org-helper-state/runtime-counters"
    "t/benchmarks/org-prefix-probe"
    "t/benchmarks/org-name-set"
    "t/benchmarks/org-helper-state"
    "t/benchmarks/org-event-fold"
    "t/benchmarks/org-prefix-probe/compare"
    "t/benchmarks/org-name-set/compare"
    "t/benchmarks/org-helper-state/compare"
    "t/benchmarks/org-event-fold/compare")
  libdir: (path-expand "lib" benchmark-root)
  bindir: (path-expand "bin" benchmark-root))
