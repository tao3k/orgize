;;; -*- Gerbil -*-
;;; One native keyword plan owns prescan routing, words, tags and options.
(import "text-funs.ss")
(export org-keyword-facts)

(def (ascii-case text up?)
  (list->string
   (map (lambda (ch)
          (let (n (char->integer ch))
            (cond ((and up? (<= 97 n 122)) (integer->char (- n 32)))
                  ((and (not up?) (<= 65 n 90)) (integer->char (+ n 32)))
                  (else ch)))) (string->list text))))

(def (words text (tags? #f))
  (let ((end (string-length text)))
    (def (separator? ch) (or (org-space? ch) (and tags? (char=? ch #\:))))
    (let loop ((index 0) (start #f) (out '()))
      (if (= index end)
        (reverse (if start (cons (substring text start end) out) out))
        (if (separator? (string-ref text index))
          (loop (+ index 1) #f (if start (cons (substring text start index) out) out))
          (loop (+ index 1) (or start index) out))))))

(def (boolean-word value)
  (let (word (ascii-case value #f))
    (cond ((member word '("t" "true" "yes")) "true")
          ((member word '("nil" "false" "no")) "false") (else ""))))

(def (org-keyword-facts key value)
  (let* ((route (ascii-case key #t))
         (known (member route '("TITLE" "AUTHOR" "DATE" "CAPTION" "PYTHON" "PYTHON_FILE"
                               "PYTHON-FILE" "READONLY" "ALLPRIORITIES" "CONTRACT_ORG"
                               "FILETAGS" "OPTIONS" "PROPERTY" "ARCHIVE" "SELECT_TAGS" "EXCLUDE_TAGS" "LINK"
                               "MACRO" "INCLUDE" "TAGS")))
         (trimmed (org-trim value))
         (tokens (if (member route '("PROPERTY" "LINK" "SELECT_TAGS" "EXCLUDE_TAGS" "OPTIONS"))
                   (words trimmed) '()))
         (first (if (null? tokens) "" (car tokens)))
         (rest (org-trim (substring trimmed (string-length first) (string-length trimmed))))
         (options (make-hash-table)))
    ;; One word walk; later occurrences override earlier options.
    (for-each
     (lambda (word)
       (let (colon (org-find word ":"))
         (when colon
           (let ((name (substring word 0 colon)) (setting (substring word (+ colon 1) (string-length word))))
             (when (member name '("H" "-" "e")) (hash-put! options name setting))))))
     tokens)
    (append (list (list "route" (if known route "")) (list "first" first) (list "rest" rest))
            (map (lambda (word) (list "word" word)) tokens)
            (map (lambda (word) (list "tag" word)) (if (string=? route "FILETAGS") (words trimmed #t) '()))
            (filter-map (lambda (name)
                          (let (setting (hash-get options name))
                            (and setting (list name (if (string=? name "H") setting (boolean-word setting))))))
                        '("H" "-" "e")))))
