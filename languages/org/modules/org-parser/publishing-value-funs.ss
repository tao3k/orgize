;;; -*- Gerbil -*-
;;; Pure publishing policy. Rows are the final ABI projection, never evaluated.
(import (only-in "text-funs.ss" org-trim org-space? org-find org-prefix-at?)
        (only-in "property-token-funs.ss" org-value-words org-ascii-lower))
(export org-publishing-keyword org-publishing-keywords)

;; One runtime admission for the document, not one crossing per keyword.
(def (org-publishing-keywords fields)
  (let loop ((fields fields) (index 0) (rows '()))
    (if (null? fields) (reverse rows)
      (let (projected (org-publishing-keyword (car fields) (cadr fields)))
        (loop (cddr fields) (+ index 1)
              (foldl (lambda (row rows) (cons (cons (number->string index) row) rows))
                     rows projected))))))

(def (option-kind key)
  (if (member key '("H" "num" "-" "e" "todo" "tags" "<" "author" "creator"
                   "date" "email" "title" "d" "p" "pri" "broken-links"))
    key "other"))

(def (org-publishing-keyword key raw)
  (let ((key (org-ascii-lower key)) (text (org-trim raw)))
    (cond
      ((string=? key "export_file_name") (list (list "export-file-name" text)))
      ((string=? key "setupfile") (list (list "setup-file" text)))
      ((string=? key "bind")
       (if (string=? text "") '()
         (let* ((end (string-length text))
                (split (let loop ((i 0))
                         (if (or (= i end) (org-space? (string-ref text i))) i
                           (loop (+ i 1))))))
           (list (list "bind" (substring text 0 split)
                       (org-trim (substring text split end)))))))
      ((string=? key "options")
       (let loop ((tokens (org-value-words raw)) (rows '()))
         (if (null? tokens) (reverse rows)
           (let* ((token (car tokens)) (split (org-find token ":"))
                  (name (and split (substring token 0 split))))
             (loop (cdr tokens)
                   (if split
                     (cons (list "option" name (substring token (+ split 1) (string-length token))
                                 token (option-kind name)) rows)
                     rows))))))
      ((org-prefix-at? key "attr_" 0)
       ;; Preserve repeated prefix stripping from the former projection.
       (let loop ((start 0))
         (if (org-prefix-at? key "attr_" start) (loop (+ start 5))
           (list (list "attribute" (substring key start (string-length key)))))))
      ((or (ormap (lambda (prefix) (org-prefix-at? key prefix 0))
                  '("html_" "latex_" "md_" "beamer_" "odt_"))
           (member key '("export_title" "export_author" "export_date")))
       (list (list "backend-keyword" text)))
      (else '()))))
