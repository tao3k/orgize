(import (only-in :std/test check test-case test-suite)
        (only-in :orgize/bindings/c/clock native-monotonic-ns native-thread-cpu-ns)
        (only-in :orgize/t/clock/reference reference-thread-cpu-ns reference-process-cpu-ns reference-spin reference-spin-snapshot))
(export clock-test)

;; The production Mach interval encloses the reference thread interval.
;; Process CPU includes all process threads, not just this Scheme owner.
(def (observe-clock label work)
  (let* ((wall-begin (native-monotonic-ns))
         (process-begin (reference-process-cpu-ns))
         (owner-begin (native-thread-cpu-ns))
         (thread-begin (reference-thread-cpu-ns))
         (result (work))
         (thread-end (reference-thread-cpu-ns))
         (owner-end (native-thread-cpu-ns))
         (process-end (reference-process-cpu-ns))
         (wall-end (native-monotonic-ns))
         (owner (- owner-end owner-begin))
         (thread (- thread-end thread-begin))
         (process (- process-end process-begin))
         (wall (- wall-end wall-begin)))
    (for-each (lambda (value) (check (> value 0) => #t))
              (list wall-begin process-begin owner-begin thread-begin))
    (check (<= thread-begin thread-end) => #t)
    (check (<= owner-begin owner-end) => #t)
    (check (<= process-begin process-end) => #t)
    (check (<= wall-begin wall-end) => #t)
    ;; Mach separately reports microsecond user/system counters: their sum
    ;; can lose up to two microseconds. This is clock precision, not a SLO.
    (check (<= thread (+ owner 2000)) => #t)
    (check (<= owner (+ process 2000)) => #t)
    (displayln "NATIVE-CLOCK " label " wall_ns=" wall
               " mach_owner_ns=" owner " posix_thread_ns=" thread
               " process_ns=" process)
    (force-output)
    result))

(def clock-test
  (test-suite "Native diagnostic clock calibration"
    (test-case "CPU work advances both same-thread clocks"
      (for-each
       (lambda (count)
         (check (observe-clock 'cpu (lambda () (reference-spin count)))
                => (quotient (* count (- count 1)) 2))
         ;; Clocks here bracket only the C loop, not its Scheme call/return.
         (let ((wall (reference-spin-snapshot 0))
               (thread (reference-spin-snapshot 1))
               (process (reference-spin-snapshot 2)))
           (check (> thread 0) => #t)
           (check (<= thread (+ process 2000)) => #t)
           (displayln "NATIVE-C-LOOP iterations=" count " wall_ns=" wall
                      " thread_ns=" thread " process_ns=" process)
           (force-output)))
       (append '(1000000 4000000) (make-list 10 16000000))))
    (test-case "native Scheme sleep reports wall and CPU separately"
      (observe-clock 'sleep (lambda () (thread-sleep! 0.02))))
    (test-case "clock-only observations remain monotonic"
      (observe-clock 'empty (lambda () 'ok)))))
