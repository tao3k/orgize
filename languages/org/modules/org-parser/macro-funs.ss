;;; -*- Gerbil -*-
;;; Pure macro expansion; syntax and escaped arguments remain Scheme-owned.
(import (only-in "../../rowan-event-runtime.ss" parse-org-inline-events))
(export expand-org-macro-template expand-org-property-macros)

(def (string-concatenate chunks) (string-join chunks ""))

(def (expand-org-macro-template template arguments)
  (let ((end (string-length template)) (indexed (list->vector arguments)))
    (let loop ((index 0) (start 0) (chunks '()) (all #f))
      (if (= index end)
        (string-concatenate (reverse (cons (substring template start end) chunks)))
        (if (and (char=? (string-ref template index) #\$) (< (+ index 1) end))
          (let* ((next (string-ref template (+ index 1)))
                 (digit (- (char->integer next) (char->integer #\0))))
            (if (or (char=? next #\$) (<= 0 digit 9))
              (let* ((joined (if (= digit 0) (or all (string-join arguments ", ")) all))
                     (value (cond ((char=? next #\$) "$")
                                  ((= digit 0) joined)
                                  ((< digit (+ 1 (vector-length indexed))) (vector-ref indexed (- digit 1)))
                                  (else ""))))
                (loop (+ index 2) (+ index 2)
                      (cons value (cons (substring template start index) chunks)) joined))
              (loop (+ index 1) start chunks all)))
          (loop (+ index 1) start chunks all))))))

(def (source-text bytes start end) (utf8->string bytes start end))

;; Consume exactly the native OrgMacro subtree. Argument chunks are already
;; cooked by macro-arguments; neither commas nor escapes are reparsed here.
(def (macro-call bytes events)
  (let loop ((rest events) (depth 1) (start #f) (end 0)
             (name "") (arguments '()) (argument #f) (argument-depth #f))
    (let* ((event (car rest)) (tag (car event)))
      (case tag
        ((start)
         (if (eq? (cadr event) 'OrgMacroArgument)
           (loop (cdr rest) (+ depth 1) start end name arguments '() (+ depth 1))
           (loop (cdr rest) (+ depth 1) start end name arguments argument argument-depth)))
        ((token)
         (let* ((kind (cadr event)) (from (caddr event)) (until (cadddr event))
                (text (source-text bytes from until)))
           (loop (cdr rest) depth (or start from) until
                 (if (eq? kind 'MacroName) text name) arguments
                 (if (eq? kind 'MacroArgumentText) (cons text argument) argument)
                 argument-depth)))
        ((finish)
         (cond ((= depth 1) (values name (reverse arguments) start end (cdr rest)))
               ((and argument-depth (= depth argument-depth))
                (loop (cdr rest) (- depth 1) start end name
                      (cons (string-concatenate (reverse argument)) arguments) #f #f))
               (else (loop (cdr rest) (- depth 1) start end name arguments argument argument-depth))))
        (else (error "invalid native macro event" event))))))

(def (expand-org-property-macros source definitions)
  (let ((bytes (string->utf8 source)) (templates (make-hash-table)))
    ;; Source order gives later definitions the same explicit override priority.
    (for-each (lambda (definition) (hash-put! templates (car definition) (cdr definition)))
              definitions)
    (let loop ((events (parse-org-inline-events source)) (cursor 0) (chunks '()))
      (if (null? events)
        (string-concatenate
         (reverse (cons (source-text bytes cursor (u8vector-length bytes)) chunks)))
        (let (event (car events))
          (if (and (eq? (car event) 'start) (eq? (cadr event) 'OrgMacro))
            (let-values (((name arguments start end rest) (macro-call bytes (cdr events))))
              (let* ((template (hash-get templates name))
                     (value (if template (expand-org-macro-template template arguments)
                              (source-text bytes start end))))
                (loop rest end (cons value (cons (source-text bytes cursor start) chunks)))))
            (loop (cdr events) cursor chunks)))))))
