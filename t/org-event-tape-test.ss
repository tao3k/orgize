(import (only-in :std/test check test-case test-suite)
        (only-in "../languages/org/native-event-tape.ss"
                 org-events->tape org-request->tape org-event-tape-header)
        (only-in "../languages/org/native-event-runtime.ss"
                 parse-org-native-events-with-parameters))
(export org-event-tape-test)

(def (request source level policy (operation 0))
  (let* ((source-bytes (string->utf8 source))
         (bytes (make-u8vector (+ 14 (u8vector-length source-bytes)) 0)))
    (u8vector-copy! bytes 0 (u8vector 79 78 82 49))
    (u8vector-set! bytes 4 operation)
    (u8vector-set! bytes 5 level)
    (u8vector-set! bytes 13 policy)
    (u8vector-copy! bytes 14 source-bytes)
    bytes))

(def (batch-request inputs (operation 6))
  (let* ((payloads (map string->utf8 inputs))
         (bytes (make-u8vector (+ 16 (foldl (lambda (payload total)
                                             (+ total 4 (u8vector-length payload)))
                                           0 payloads)) 0)))
    (u8vector-copy! bytes 0 (request "" 15 2 operation))
    (u8vector-set! bytes 14 (length inputs))
    (let write ((rest payloads) (offset 16))
      (unless (null? rest)
        (let* ((payload (car rest)) (size (u8vector-length payload)))
          (for-each (lambda (index)
                      (u8vector-set! bytes (+ offset index)
                                     (modulo (quotient size (expt 256 index)) 256)))
                    '(0 1 2 3))
          (u8vector-copy! bytes (+ offset 4) payload)
          (write (cdr rest) (+ offset 4 size)))))
    bytes))

(def (batch-rows bytes)
  (let loop ((index 0) (offset 6) (rows '()))
    (if (= index (u8vector-ref bytes 4)) (reverse rows)
      (let* ((size (foldl (lambda (position total)
                           (+ total (* (u8vector-ref bytes (+ offset 1 position))
                                       (expt 256 position))))
                         0 '(0 1 2 3)))
             (start (+ offset 5)) (end (+ start size)))
        (loop (+ index 1) end
              (cons (cons (u8vector-ref bytes offset) (subu8vector bytes start end)) rows))))))

(def org-event-tape-test
  (test-suite "Org native event boundary"
    (test-case "bounded batch retains ordered individual grammar tapes"
      (let* ((inputs '("" "* α\r\nx^{β}\n" "* duplicate\n" "* duplicate\n"))
             (tape (org-request->tape (batch-request inputs)))
             (rows (batch-rows tape)))
        (check (subu8vector tape 0 4) => (u8vector 79 66 84 49))
        (check (map car rows) => '(0 0 0 0))
        (check (map cdr rows) => (map (lambda (source) (org-request->tape (request source 15 2))) inputs))
        (check (map (lambda (row) (subu8vector (cdr row) 28 (u8vector-length (cdr row))))
                    (let (profile (org-request->tape (batch-request inputs 7)))
                      (check (subu8vector profile 0 4) => (u8vector 79 66 80 49))
                      (check (subu8vector profile 52 56) => (u8vector 79 66 84 49))
                      (batch-rows (subu8vector profile 52 (u8vector-length profile)))))
               => (map cdr rows))))
    (test-case "batch framing admits exact bounds and rejects truncation or overflow"
      (check (length (batch-rows (org-request->tape (batch-request (make-list 64 ""))))) => 64)
      (let (valid (batch-request '("α" "")))
        (for-each
         (lambda (end)
           (check (with-catch (lambda (exception) 'rejected)
                             (lambda () (org-request->tape (subu8vector valid 0 end))))
                  => 'rejected))
         '(0 13 14 15 16 17 18 19 20 21 22 23 24 25))
        (check (with-catch (lambda (exception) 'rejected)
                          (lambda () (org-request->tape (u8vector-append valid (u8vector 0)))))
               => 'rejected))
      (for-each
       (lambda (inputs)
         (check (with-catch (lambda (exception) 'rejected)
                           (lambda () (org-request->tape (batch-request inputs))))
                => 'rejected))
       (list '() (make-list 65 "") (list (make-string 65537 #\x)))))
    (test-case "bad document is isolated and following source still parses"
      (for-each
       (lambda (operation)
         (let (input (batch-request '("valid\n" "x" "* after\n") operation))
           ;; Mutate exactly the one-byte second payload, not its framing.
           (u8vector-set! input 30 255)
           (let* ((result (org-request->tape input))
                  (rows (batch-rows
                         (if (= operation 7)
                           (subu8vector result 52 (u8vector-length result))
                           result))))
             (check (map car rows) => '(0 1 0))
             (let (payload (cdr (caddr rows)))
               (check (if (= operation 7)
                        (subu8vector payload 28 (u8vector-length payload)) payload)
                      => (org-request->tape (request "* after\n" 15 2)))))))
       '(6 7)))
    (test-case "empty document retains root and grammar identity"
      (let* ((header-size (u8vector-length org-event-tape-header))
             (tape (org-request->tape (request "" 15 2))))
        (check (subu8vector tape 0 header-size) => org-event-tape-header)
        (check (subu8vector tape header-size (u8vector-length tape))
               => (u8vector 1 0 0 0))))
    (test-case "combined configuration and multibyte input keep Scheme events"
      (for-each
       (lambda (config)
         (let ((source "* α\r\nx^{β} x^2\n"))
           (check (org-request->tape (request source (car config) (cdr config)))
                  => (org-events->tape
                      (parse-org-native-events-with-parameters
                       source (car config) (cdr config))))))
       '((1 . 0) (15 . 1) (15 . 2))))
    (test-case "native expression requests carry their own grammar identity"
      (let ((tape (org-request->tape (request "(query π \"one\"\"two\") ; note\n" 0 0 1))))
        (check (subu8vector tape 0 4) => (u8vector 79 69 86 49))
        (check (equal? (subu8vector tape 0 (u8vector-length org-event-tape-header))
                       org-event-tape-header) => #f)))
    (test-case "malformed expressions and unknown operations reject"
      (for-each
       (lambda (source)
         (check (with-catch (lambda (exception) 'rejected)
                           (lambda () (org-request->tape (request source 0 0 1))))
                => 'rejected))
       '(")" "(query" "\"unclosed"))
      (check (with-catch (lambda (exception) 'rejected)
                        (lambda () (org-request->tape (request "" 0 0 255)))) => 'rejected)
      (check (with-catch (lambda (exception) 'rejected)
                        (lambda () (org-request->tape (make-u8vector 9 0)))) => 'rejected))
    (test-case "invalid requests reject before parser execution"
      (check (with-catch (lambda (exception) 'rejected)
                        (lambda () (org-request->tape (u8vector 0)))) => 'rejected)
      (check (with-catch (lambda (exception) 'rejected)
                        (lambda () (org-request->tape (request "x" 0 2)))) => 'rejected)
      (check (with-catch (lambda (exception) 'rejected)
                        (lambda () (org-request->tape (request "x" 15 3)))) => 'rejected))
    (test-case "diagnostic envelope preserves exact ordinary tape"
      (for-each
       (lambda (source)
         (let (profile (org-request->tape (request source 15 2 5)))
           (check (subu8vector profile 0 4) => (u8vector 79 80 82 49))
           (check (subu8vector profile 28 (u8vector-length profile))
                  => (org-request->tape (request source 15 2)))))
       '("" "* α\r\nx^{β} x^2\n" "#+begin_src rust\nlet x = 1;\n#+end_src\n"))
      (check (with-catch (lambda (exception) 'rejected)
                        (lambda () (org-request->tape (request "x" 0 2 5)))) => 'rejected))
    (test-case "typed values carry cooked strings without CST reconstruction"
      (let* ((tape (org-request->tape (request "\"π\\n\\t\\q\"" 0 0 2)))
             (offset (u8vector-length org-event-tape-header)))
        (check (subu8vector tape 0 4) => (u8vector 79 88 86 49))
        (check (u8vector-ref tape offset) => 4)
        (check (utf8->string tape (+ offset 9) (u8vector-length tape)) => "π\n\tq"))
      (check (with-catch (lambda (exception) 'rejected)
                        (lambda () (org-request->tape (request ")" 0 0 2)))) => 'rejected)
      (check (with-catch (lambda (exception) 'rejected)
                        (lambda () (org-request->tape (request "x" 1 0 2)))) => 'rejected))
    (test-case "expectation grammar admits exactly one native form"
      (for-each
       (lambda (source)
         (check (subu8vector (org-request->tape (request source 0 0 3)) 0 4)
                => (u8vector 79 88 86 49)))
       '("exists" "not exists" "count <= 0" "count < 1" "count >= 1"
         "count > 0" "count == 1" "count != 2" "# note\r\ncount >= 1 # note"))
      (for-each
       (lambda (source)
         (check (with-catch (lambda (exception) 'rejected)
                           (lambda () (org-request->tape (request source 0 0 3))))
                => 'rejected))
       '("" "exists not exists" "notexists" "count >= 1x"
         "count >= -1" "count >= nope" "count >= 1 ignored"
         "count >= 18446744073709551616")))
    (test-case "Contract composition is normalized once by Scheme"
      (check (org-request->tape
              (request "(let (($a (paragraph))) (let (($b (headline))) (assert exists (paragraph))))" 0 0 4))
             => (org-request->tape
                 (request "(let ((a (paragraph)) (b (headline)))) (assert exists (paragraph))" 0 0 2)))
      (for-each
       (lambda (source)
         (check (with-catch (lambda (exception) 'rejected)
                           (lambda () (org-request->tape (request source 0 0 4))))
                => 'rejected))
       '("(let (($a (paragraph))) (let ((a (headline))) (assert exists (paragraph))))"
         "(let (($ (paragraph))) (assert exists (paragraph)))"
         "(let (($a (paragraph) ignored)) (assert exists (paragraph)))"
         "(assert exists (paragraph)) (let ())"
         "(let () (let ())) (assert exists (paragraph))"
         "(assert exists (paragraph) ignored)"
         "(imaginary)")))))
