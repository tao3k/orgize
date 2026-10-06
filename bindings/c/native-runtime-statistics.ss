;;; Private, opt-in diagnostic projection of Gambit's existing statistics API.
;;; No raw C VM fields, kernel changes, forced collections, or runtime policy.
(export native-runtime-statistics-snapshot native-runtime-statistics-delta)

;; Layout follows ##process-statistics / ##exec-stats in the qualified native
;; Gambit owner: seconds at 0..5, collection count at 6.
;; Reject unexpected shape/numeric representation. The field semantics still
;; belong to this qualified owner version; upgrades require requalification.
(def (admit-statistics snapshot)
  (unless (and (f64vector? snapshot) (= (f64vector-length snapshot) 20))
    (error "unsupported native runtime statistics layout"))
  (let loop ((index 0))
    (when (< index 7)
      (let ((value (f64vector-ref snapshot index)))
        ;; Also rejects NaN/infinity; counts must remain exactly integral.
        (unless (and (<= 0 value) (< value 9007199254740992.)
                     (or (< index 6) (= value (floor value))))
          (error "invalid native runtime statistics counter" index)))
      (loop (+ index 1))))
  snapshot)

(def (native-runtime-statistics-snapshot)
  (admit-statistics (##process-statistics)))

(def (counter-delta begin end index)
  (let ((value (- (f64vector-ref end index) (f64vector-ref begin index))))
    (unless (>= value 0)
      (error "native runtime statistics counter regressed" index))
    value))

(def (seconds->ns seconds)
  (let ((ns (inexact->exact (floor (* seconds 1000000000.)))))
    (unless (<= 0 ns 18446744073709551615)
      (error "native runtime statistics duration overflow"))
    ns))

;; Internal transport-ready values: process CPU ns, VM GC CPU ns, VM GC wall
;; ns, VM collection count. GC is VM-wide;
;; process CPU includes every OS thread. Overlapping intervals are not additive.
;; Snapshots allocate: a collection caused by the ending snapshot can be
;; included. This is neither exclusive per-document accounting nor an
;; allocation profiler. Field 7 is deliberately not projected: the native
;; qualification observed it regress across GC, so it is not admitted here
;; as a monotonic cumulative allocation counter.
(def (native-runtime-statistics-delta begin end)
  (admit-statistics begin)
  (admit-statistics end)
  (vector
   (seconds->ns (+ (counter-delta begin end 0) (counter-delta begin end 1)))
   (seconds->ns (+ (counter-delta begin end 3) (counter-delta begin end 4)))
   (seconds->ns (counter-delta begin end 5))
   (inexact->exact (counter-delta begin end 6))))
