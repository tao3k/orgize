;;; The shared native lifecycle enters this service only for runtime-scheme.
;;; One native actor owns all Scheme ABI calls and GC root transfers. The
;;; blocking idle wait is intentional for this single-worker comparative lane;
;;; it is NOT a general green-thread I/O scheduler or an SMP implementation.
(import (only-in "../../languages/org/native-event-tape.ss" org-request->tape))
(export orgize-scheme-runtime-link-anchor)
(extern namespace: orgize/bindings/c/orgize-scheme-runtime
  take-job job-kind job-length copy-job publish-job process-contract-job
  fail-job runtime-ready)

(def (process-runtime-job job)
  (with-catch
   (lambda (exception) (fail-job job))
   (lambda ()
     (if (= (job-kind job) 0)
       (let (bytes (make-u8vector (job-length job)))
         (unless (= (copy-job job bytes) 1)
           (error "invalid native request storage"))
         (publish-job job (org-request->tape bytes)))
       (process-contract-job job)))))

(def (run-scheme-runtime)
  (let (actor
        (spawn/name 'org-scheme-runtime
          (lambda ()
            ;; Initialization must cover green-thread startup, not merely
            ;; runtime creation. Publish only when the serving actor runs.
            (runtime-ready)
            (let loop ()
              (let (job (take-job))
                (when job
                  (process-runtime-job job)
                  (loop)))))))
    (thread-join! actor)
    0))

(def orgize-scheme-runtime-link-anchor run-scheme-runtime)

(begin-foreign
  (namespace ("orgize/bindings/c/orgize-scheme-runtime#"
              take-job job-kind job-length copy-job publish-job
              process-contract-job fail-job runtime-ready))
  (c-declare "#include \"orgize_runtime.h\"")
  (define take-job (c-lambda () (pointer void) "orgize_runtime_take"))
  (define job-kind (c-lambda ((pointer void)) int
    "___return(((orgize_runtime_job*)___arg1)->kind);"))
  (define job-length (c-lambda ((pointer void)) unsigned-int64
    "___return(((orgize_runtime_job*)___arg1)->length);"))
  (define copy-job (c-lambda ((pointer void) scheme-object) int
    "orgize_runtime_job *job = (orgize_runtime_job*)___arg1; if (job->length != ___HD_BYTES(___HEADER(___arg2))) { ___return(0); } if (job->length) memcpy(___CAST(void*, ___BODY_AS(___arg2, ___tSUBTYPED)), job->input, job->length); ___return(1);"))
  (define publish-job (c-lambda ((pointer void) scheme-object) int32
    "___return(orgize_runtime_publish((orgize_runtime_job*)___arg1, ___CAST(const uint8_t*, ___BODY_AS(___arg2, ___tSUBTYPED)), ___HD_BYTES(___HEADER(___arg2))));"))
  (define process-contract-job (c-lambda ((pointer void)) void "orgize_runtime_process_contract"))
  (define fail-job (c-lambda ((pointer void)) void "orgize_runtime_fail_job"))
  (define runtime-ready (c-lambda () void "orgize_runtime_ready"))
  (c-define (orgize-scheme-runtime-run) () int32
    "orgize_scheme_runtime_run" "extern"
    (with-exception-catcher
     (lambda (exception) -1)
     (lambda () (orgize/bindings/c/orgize-scheme-runtime#run-scheme-runtime)))))
