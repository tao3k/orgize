(import (only-in :std/test check test-case test-suite)
        (only-in "../bindings/c/orgize-actor.ss" org-parse-batch)
        (only-in "../languages/org/rowan-event-runtime.ss"
                 parse-org-rowan-events-with-parameters))
(export org-native-actor-test)

(def (parse source)
  (parse-org-rowan-events-with-parameters source 15 2))

(def org-native-actor-test
  (test-suite "Org native actor ownership"
    (test-case "empty batch has no actors or results"
      (check (org-parse-batch '() workers: 4) => '()))
    (test-case "native actors preserve indexed parser results"
      (let* ((inputs '("" "* α\n" "x^{β}\n" "| a | b |\n"
                      "#+begin_src scheme\n(+ 1 2)\n#+end_src\n"))
             (expected (map parse inputs)))
        (for-each (lambda (workers)
                    (check (org-parse-batch inputs workers: workers) => expected))
                  '(1 2 4 8))))
    (test-case "request failure is contained and next batch still works"
      (check (with-catch (lambda (exception) 'rejected)
                        (lambda ()
                          (org-parse-batch (list "* valid\n" 42 "after invalid\n")
                                           workers: 3))) => 'rejected)
      (let (input "* recovered\n")
        (check (org-parse-batch (list input) workers: 4)
               => (list (parse input)))))
    (test-case "native partitions do not consume the caller mailbox"
      (thread-send (current-thread) 'host-owned-message)
      (org-parse-batch (list "* isolated\n") workers: 2)
      (check (thread-receive 1 'missing) => 'host-owned-message))
    (test-case "native tasks preserve caller parser configuration"
      (let (input "* α\nx^{β} x^2\n")
        (for-each
         (lambda (config)
           (check (org-parse-batch (list input input) workers: 2
                                   inlinetask-level: (car config)
                                   script-policy: (cdr config))
                  => (let (events (parse-org-rowan-events-with-parameters
                                    input (car config) (cdr config)))
                       (list events events))))
         '((1 . 0) (15 . 1) (15 . 2)))))
    (test-case "worker policy is checked before spawning"
      (for-each
       (lambda (workers)
         (check (with-catch (lambda (exception) 'rejected)
                           (lambda () (org-parse-batch '() workers: workers)))
                => 'rejected))
       '(0 -1 65 1.5)))))
