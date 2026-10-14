;;; -*- Gerbil -*-
;;; Private positional compiler state, not a public authoring DSL.
(import (only-in :gerbil-parser/src/runtime/artifact
                 event-kind token-event-token-kind token-event-lexeme))
(export contract-events->rows)

(defstruct contract-form (kind value))

(def (atom-value form)
  (and (eq? (contract-form-kind form) 'atom) (contract-form-value form)))

(def (form-items form)
  (and (eq? (contract-form-kind form) 'list) (contract-form-value form)))

(def (form-head items)
  (and (pair? items) (atom-value (car items))))

(def (events->forms events)
  (let loop ((rest events) (stack (list '())))
    (if (null? rest) (reverse (car stack))
      (let* ((event (car rest))
             (kind (event-kind event)))
        (cond
         ((and (eq? kind 'start-node) (eq? (vector-ref event 2) 'ContractList))
          (loop (cdr rest) (cons '() stack)))
         ((and (eq? kind 'finish-node) (eq? (vector-ref event 2) 'ContractList))
          (let (form (make-contract-form 'list (reverse (car stack))))
            (loop (cdr rest) (cons (cons form (cadr stack)) (cddr stack)))))
         ((and (eq? kind 'token) (memq (token-event-token-kind event) '(atom string)))
          (let (form (make-contract-form (token-event-token-kind event) (token-event-lexeme event)))
            (loop (cdr rest) (cons (cons form (car stack)) (cdr stack)))))
         (else (loop (cdr rest) stack)))))))

(def (binding-name form)
  (let (text (atom-value form))
    (unless text (error "Contract binding name must be an atom"))
    (let loop ((index 0))
      (if (and (< index (string-length text)) (= (char->integer (string-ref text index)) 36))
        (loop (+ index 1))
        (begin
          (when (= index (string-length text)) (error "empty Contract binding name"))
          (substring text index (string-length text)))))))

;; Flatten composition once in source order. The local name set is parse-local;
;; no hidden registry, runtime effects or duplicate Rust policy.
(def (normalize-contract forms)
  (let (names (make-hash-table))
    (let loop ((rest forms) (bindings '()))
      (when (null? rest) (error "Contract requires one final assertion"))
      (let* ((form (car rest)) (items (form-items form)) (head (form-head items)))
        (cond
         ((equal? head "let")
          (unless (>= (length items) 2) (error "missing Contract bindings"))
          (let (declarations (form-items (cadr items)))
            (unless declarations (error "Contract bindings must be a list"))
            (let bind ((remaining declarations) (acc bindings))
              (if (null? remaining)
                (if (null? (cddr items))
                  (loop (cdr rest) acc)
                  (begin
                    (unless (null? (cdr rest)) (error "Contract let body must be final"))
                    (loop (cddr items) acc)))
                (let* ((pair (form-items (car remaining))))
                  (unless (and pair (= (length pair) 2)) (error "invalid Contract binding"))
                  (let (name (binding-name (car pair)))
                    (when (hash-get names name) (error "duplicate Contract binding" name))
                    (hash-put! names name #t)
                    (bind (cdr remaining)
                          (cons (make-contract-form 'list
                                  (list (make-contract-form 'atom name) (cadr pair))) acc))))))))
         ((equal? head "assert")
          (unless (and (null? (cdr rest)) (>= (length items) 2))
            (error "Contract assertion must be final"))
          (let* ((expectation (atom-value (cadr items)))
                 (arity (cond ((member expectation '("exists" "not-exists")) 3)
                              ((member expectation '("count" "value-set")) 5)
                              (else #f))))
            (unless (and arity (= (length items) arity))
              (error "invalid Contract assertion shape"))
            (if (null? bindings) (list form)
              (list (make-contract-form 'list
                      (list (make-contract-form 'atom "let")
                            (make-contract-form 'list (reverse bindings))))
                    form))))
         (else (error "unsupported Contract composition")))))))

(def (contract-events->rows events string-value)
  (let emit ((todo (normalize-contract (events->forms events))) (rows '()))
    (if (null? todo) (reverse rows)
      (let (form (car todo))
        (if (eq? form 'finish)
          (emit (cdr todo) (cons (cons 2 #f) rows))
          (case (contract-form-kind form)
            ((list)
             (emit (append (contract-form-value form) (cons 'finish (cdr todo)))
                   (cons (cons 1 #f) rows)))
            ((atom)
             (emit (cdr todo) (cons (cons 3 (string->utf8 (contract-form-value form))) rows)))
            ((string)
             (emit (cdr todo) (cons (cons 4 (string->utf8
                                            (string-value (contract-form-value form)))) rows)))
            (else (error "invalid admitted Contract value"))))))))
