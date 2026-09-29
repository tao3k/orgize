;;; -*- Gerbil -*-
;;; Pure Org document keyword algorithms, shared by Scheme tests and Rust AOT.

(import (only-in :std/string/misc string-trim)
        (only-in :gerbil-parser/src/compiler/rust-pure-aot
                 define-rust-pure string-words string-replace
                 string-first-word string-rest-after-first-word
                 string-before string-after string-prefix? ascii-ci=?))
(export keyword-word keyword-word-rust
        keyword-words keyword-words-rust
        keyword-tag-words keyword-tag-words-rust
        keyword-first-word keyword-first-word-rust
        keyword-rest keyword-rest-rust
        keyword-option-value keyword-option-value-rust
        keyword-option-present? keyword-option-present-rust
        keyword-boolean-value keyword-boolean-value-rust)

(define-rust-pure keyword-word keyword-word-rust
  ((word "&str")) "String"
  (string-trim word))

(define-rust-pure keyword-words keyword-words-rust
  ((value "&str")) "Vec<String>"
  (using ((keyword-word "&str"))
    (map keyword-word (string-words value))))

(define-rust-pure keyword-tag-words keyword-tag-words-rust
  ((value "&str")) "Vec<String>"
  (using ((keyword-word "&str"))
    (map keyword-word
         (string-words (string-replace value ":" " ")))))

(define-rust-pure keyword-first-word keyword-first-word-rust
  ((value "&str")) "String"
  (string-first-word value))

(define-rust-pure keyword-rest keyword-rest-rust
  ((value "&str")) "String"
  (string-rest-after-first-word (string-trim value)))

(define-rust-pure keyword-option-value keyword-option-value-rust
  ((value "&str") (key "&str")) "String"
  (string-trim
   (foldl (lambda (word found)
            (if (and (equal? (string-before word ":") key)
                     (string-prefix? (string-after word key) ":"))
              (string-after word ":")
              found))
          "" (string-words value))))

(define-rust-pure keyword-option-present? keyword-option-present-rust
  ((value "&str") (key "&str")) "bool"
  (ormap (lambda (word)
           (and (equal? (string-before word ":") key)
                (string-prefix? (string-after word key) ":")))
         (string-words value)))

(define-rust-pure keyword-boolean-value keyword-boolean-value-rust
  ((value "&str")) "&str"
  (if (or (ascii-ci=? value "t")
          (ascii-ci=? value "true")
          (ascii-ci=? value "yes"))
    "true"
    (if (or (ascii-ci=? value "nil")
            (ascii-ci=? value "false")
            (ascii-ci=? value "no"))
      "false" "")))
