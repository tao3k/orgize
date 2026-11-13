;;; -*- Gerbil -*-
;;; The existing native-build owner stages the compiler's program closure.
(import (only-in :gerbil-scheme-rust/scheme/program-build gerbil-rs-stage-program)
        (only-in :std/make pkg-config))
(export main)

(def (crypto-static-link-options)
  ;; The Bridge declares the ordinary OpenSSL link. Exporting libcrypto.a
  ;; also needs the private libraries of this exact package provider.
  (def directory (pkg-config "libcrypto" "--variable=libdir"))
  (def options (pkg-config "libcrypto" "--static" "--libs"))
  (def (manifest-option flag)
    (cond
     ((equal? flag "-pthread") "-lpthread")
     ((and (string-prefix? "-l:lib" flag) (string-suffix? ".a" flag))
      (string-append "-l" (substring flag 6 (- (string-length flag) 2))))
     (else flag)))
  (string-join
    (cons (string-append "-L" directory)
          (map manifest-option (filter (lambda (flag) (not (string-empty? flag)))
                                      (string-split options #\space))))
    " "))

(def (main output-dir)
  (displayln "ORG-PARSER-STAGE begin")
  (force-output)
  ;; Headers are explicit native build contract inputs, not an extension to
  ;; the upstream compiler manifest schema.
  (gerbil-rs-stage-program "bindings/c/orgize-parser.ss" output-dir
                          gsc-options: ["-ld-options" (crypto-static-link-options)])
  (displayln "ORG-PARSER-STAGE OK"))
