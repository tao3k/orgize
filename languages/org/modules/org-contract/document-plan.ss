;;; -*- Gerbil -*-
;;; Pure, document-local Contract facts over parser-owned field projections.
(import (only-in "../org-parser/contract-reference-funs.ss"
                 org-contract-policy org-contract-aliases org-severity org-contract-block-plans)
        (only-in "../org-parser/property-token-funs.ss" org-ascii-lower))
(export org-contract-document-plan)
(defstruct contract-source-input (sections blocks))

;; Decode the complete counted shape before invoking any source compiler.
(def (decode-input fields)
  (def limit (length fields))
  (def (enough? rest n)
    (or (= n 0) (and (pair? rest) (enough? (cdr rest) (- n 1)))))
  (def (count! rest)
    (unless (pair? rest) (error "missing Contract source count"))
    (let (n (string->number (car rest)))
      (unless (and (exact-integer? n) (<= 0 n limit)
                   (string=? (number->string n) (car rest)))
        (error "invalid Contract source count"))
      n))
  (let sections ((n (count! fields)) (rest (cdr fields)) (out '()))
    (if (= n 0)
      (let blocks ((n (count! rest)) (rest (cdr rest)) (out-blocks '()))
        (if (= n 0)
          (begin
            (unless (null? rest) (error "trailing Contract source fields"))
            (make-contract-source-input (reverse out) (reverse out-blocks)))
          (begin
            (unless (enough? rest 4) (error "truncated Contract source block"))
            (blocks (- n 1) (list-tail rest 4)
                    (cons (list (car rest) (cadr rest) (caddr rest) (cadddr rest)) out-blocks)))))
      (let props ((p (count! rest)) (rest (cdr rest)) (properties '()))
        (if (= p 0)
          (sections (- n 1) rest (cons (reverse properties) out))
          (begin
            (unless (enough? rest 2) (error "truncated Contract properties"))
            (props (- p 1) (cddr rest)
                   (cons (cons (car rest) (cadr rest)) properties))))))))

(def property-keys
  '("contract_id" "contract_kind" "contract_scope" "contract_alias" "assert_id" "severity"))
(def (section-rows properties index)
  (let (values (make-hash-table))
    (for-each (lambda (property)
                (hash-put! values (org-ascii-lower (car property))
                           (org-contract-policy "name" (cdr property))))
              properties)
    (let* ((selected (map (lambda (key) (hash-get values key)) property-keys))
           (id (number->string index))
           (kind (list-ref selected 1)) (scope (list-ref selected 2))
           (aliases (list-ref selected 3)) (severity (list-ref selected 5)))
      (cons
       (append (list "section" id)
               (foldr (lambda (value tail)
                        (cons (if value "true" "false") (cons (or value "") tail)))
                      '() selected)
               (list (org-contract-policy "kind" (or kind "org-elements"))
                     (org-contract-policy "scope" (or scope "subtree"))
                     (or (and severity (org-severity severity)) "")))
       (if aliases
         (map (lambda (alias) (list "alias" id alias)) (org-contract-aliases aliases))
         '())))))

;; A malformed block remains an explicit per-block result, not a failed batch
;; or an implicit default assertion. The supplied compilers are named native
;; grammar owners; no evaluator or host callback is accepted by the wire API.
(def (form-rows index mode source compile)
  (with-catch
   (lambda (exception) (list (list "forms" index mode "invalid")))
   (lambda ()
     (cons (list "forms" index mode "ok")
           (map (lambda (row)
                  (list "form" index mode (number->string (car row))
                        (if (cdr row) (utf8->string (cdr row)) "")))
                (compile source))))))

(def (block-rows fields index expression-rows expectation-rows contract-rows)
  (let* ((language (car fields)) (name (cadr fields)) (parameters (caddr fields))
         (source (cadddr fields))
         (plan (car (org-contract-block-plans (list language name parameters))))
         (role (car plan)) (id (number->string index))
         (severity (and (string=? (list-ref plan 5) "true")
                        (org-severity (list-ref plan 6)))))
    (cons (append (list "block" id) plan (list (or severity "")))
          (cond
           ((string=? role "expect")
            (form-rows id "expect" source expectation-rows))
           ((string=? role "contract")
            (append (form-rows id "query" source expression-rows)
                    (form-rows id "contract" source contract-rows)))
           ((member role '("query" "selector"))
            (form-rows id "query" source expression-rows))
           (else '())))))

(def (org-contract-document-plan fields expression-rows expectation-rows contract-rows)
  (let (input (decode-input fields))
    (let sections ((rest (contract-source-input-sections input)) (index 0) (out '()))
      (if (null? rest)
        (let blocks ((rest (contract-source-input-blocks input)) (index 0) (out out))
          (if (null? rest) (reverse out)
            (blocks (cdr rest) (+ index 1)
                    (foldl cons out
                           (block-rows (car rest) index expression-rows expectation-rows contract-rows)))))
        (sections (cdr rest) (+ index 1)
                  (foldl cons out (section-rows (car rest) index)))))))
