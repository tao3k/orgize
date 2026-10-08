;;; -*- Gerbil -*-
;;; Private native task boundary. The POO parser remains the semantic owner.
(import (only-in "../../languages/org/native-event-runtime.ss"
                 parse-org-native-events-with-parameters))
(export org-parse-batch)

;; Each actor receives one batch partition in its own native mailbox. It owns
;; its parser calls and returns only ordinary Scheme values, never C pointers
;; or bridge root tokens. Exception containment lets all actors finish before
;; the batch caller observes a failed request.
(def (parse-partition)
  (let* ((partition (thread-receive))
         (requests (vector-ref partition 0))
         (start (vector-ref partition 1))
         (stride (vector-ref partition 2))
         (level (vector-ref partition 3))
         (policy (vector-ref partition 4)))
    (let loop ((index start) (results '()))
      (if (< index (vector-length requests))
        (let (outcome
              (with-catch
               (lambda (exception) (vector index #f exception))
               (lambda ()
                 (vector index #t
                         (parse-org-native-events-with-parameters
                          (vector-ref requests index) level policy)))))
          (loop (+ index stride) (cons outcome results)))
        (reverse results)))))

;; Functional batch surface; scheduling stays inside Gerbil. This is NOT a
;; foreign-thread attachment API or a claim that this VM supports SMP.
(def (org-parse-batch requests workers: (workers 1)
                      inlinetask-level: (level 15) script-policy: (policy 2))
  (unless (and (list? requests) (exact-integer? workers) (<= 1 workers 64)
               (exact-integer? level) (> level 0) (memv policy '(0 1 2)))
    (error "invalid Org native batch" workers))
  (let* ((inputs (list->vector requests))
         (count (vector-length inputs))
         (width (min workers count))
         (actors '())
         (output (make-vector count #f))
         (failures (make-vector count #f)))
    (dynamic-wind
     void
     (lambda ()
       (let start ((index 0))
         (when (< index width)
           (let (actor (spawn/name 'org-native-parser parse-partition))
             (set! actors (cons actor actors))
             (thread-send actor (vector inputs index width level policy)))
           (start (+ index 1))))
       ;; Join every actor before raising any request failure. Results are
       ;; placed by input index, independent of completion order or actor count.
       (let join ()
         (unless (null? actors)
           (let (results (thread-join! (car actors)))
             (set! actors (cdr actors))
             (for-each
              (lambda (outcome)
                (let ((index (vector-ref outcome 0))
                      (value (vector-ref outcome 2)))
                  (if (vector-ref outcome 1)
                    (vector-set! output index value)
                    (vector-set! failures index (cons #t value)))))
              results))
           (join)))
       (let check ((index 0))
         (when (< index count)
           (let (failure (vector-ref failures index))
             (when failure (raise (cdr failure))))
           (check (+ index 1))))
       (vector->list output))
     ;; Partial startup, unexpected actor failure and caller unwinding must not
     ;; leave private workers waiting forever in their native mailboxes.
     (lambda ()
       (for-each (lambda (actor)
                   (with-catch void (lambda () (thread-terminate! actor))))
                 actors)))))
