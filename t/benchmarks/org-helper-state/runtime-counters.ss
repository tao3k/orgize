;;; Pure diagnostic projection: ns, ns, ns, count. No clocks or GC policy.
(export runtime-counter-vector? sum-runtime-counters helper-observation-valid?)

;; Absent observations are not zero observations. Timing receipts must not
;; be compared with diagnostic receipts or silently receive fabricated zeros.
(def (helper-observation-valid? mode scope workload-scope callbacks counters workload)
  (case mode
    ((timing)
     (and (eq? scope 'disabled) (eq? workload-scope 'disabled)
          (equal? callbacks 0) (eq? counters #f) (eq? workload #f)))
    ((diagnostic)
     (and (eq? scope 'asp-run-inclusive) (eq? workload-scope 'asp-callback-10-folds)
          (equal? callbacks 20)
          (runtime-counter-vector? counters) (runtime-counter-vector? workload)))
    (else #f)))

(def (runtime-counter-vector? counters)
  (and (vector? counters) (= (vector-length counters) 4)
       (let loop ((index 0))
         (or (= index 4)
             (let (value (vector-ref counters index))
               (and (exact-integer? value) (<= 0 value) (loop (+ index 1))))))))

(def (sum-runtime-counters samples)
  (for-each (lambda (sample)
              (unless (runtime-counter-vector? sample)
                (error "invalid runtime diagnostic counter vector" sample))) samples)
  (list->vector
   (map (lambda (index)
          (foldl (lambda (sample total) (+ total (vector-ref sample index))) 0 samples))
        '(0 1 2 3))))
