;;; -*- Gerbil -*-
;;; Pure block line semantics; Rust only projects admitted rows and annotations.
(import "text-funs.ss")
(export org-block-line-facts org-dynamic-content-facts)

(def (label-parts pattern)
  (let (marker (org-find pattern "%s"))
    (if marker (cons (substring pattern 0 marker) (substring pattern (+ marker 2) (string-length pattern)))
      (cons "(ref:" ")"))))

(def (reference-name? name)
  (and (> (string-length name) 0)
       (andmap (lambda (ch)
                 (or (char<=? #\a ch #\z) (char<=? #\A ch #\Z)
                     (char<=? #\0 ch #\9) (memv ch '(#\- #\_ #\:))))
               (string->list name))))

;; Private match: start/end, whitespace-removal start, name and raw label.
(def (line-reference text label)
  (let ((prefix (car label)) (suffix (cdr label)) (end (string-length text)))
    (let search ((cursor 0))
      (let (start (org-find text prefix cursor))
        (and start
             (let* ((from (+ start (string-length prefix)))
                    (until (if (string=? suffix "")
                             (let scan ((index from))
                               (if (or (= index end) (org-space? (string-ref text index))) index
                                 (scan (+ index 1))))
                             (org-find text suffix from))))
               (if (and until (reference-name? (substring text from until)))
                 (let* ((stop (+ until (string-length suffix)))
                        (remove-start
                         (let trim ((index start))
                           (if (and (> index 0) (org-space? (string-ref text (- index 1))))
                             (trim (- index 1)) index))))
                   (list start stop remove-start (substring text from until) (substring text start stop)))
                 (let (next (+ start (max 1 (string-length prefix))))
                   (and (<= next end) (search next))))))))))

(def (without-reference text reference)
  (if reference
    (string-append (substring text 0 (list-ref reference 2))
                   (substring text (cadr reference) (string-length text))) text))

(def (expand-leading-tabs text width)
  (let (end (string-length text))
    (let loop ((index 0) (chunks '()))
      (if (and (< index end) (org-space? (string-ref text index)))
        (let (ch (string-ref text index))
          (loop (+ index 1) (cons (if (char=? ch #\tab) (make-string width #\space) (string ch)) chunks)))
        (string-join (reverse (cons (substring text index end) chunks)) "")))))

(def (leading-spaces text)
  (let (end (string-length text))
    (let loop ((index 0))
      (if (and (< index end) (char=? (string-ref text index) #\space)) (loop (+ index 1)) index))))

;; One request per block, not per line. Source offsets and derived values share
;; a single physical-line owner; normalization includes blank lines unchanged.
(def (org-block-line-facts value source pattern width preserve?)
  (let* ((label (label-parts pattern))
         (source-lines (list->vector (org-physical-lines source)))
         (value-lines (org-physical-lines value))
         (expanded (map (lambda (line) (expand-leading-tabs (car line) width)) value-lines))
         (indent (if (or preserve? (null? expanded)) 0
                   (foldl (lambda (text least) (min least (leading-spaces text)))
                          (leading-spaces (car expanded)) (cdr expanded)))))
    (let loop ((rest value-lines) (values expanded) (index 0) (rows '()))
      (if (null? rest) (reverse rows)
        (let* ((line (car rest)) (text (car line))
               (raw (if (< index (vector-length source-lines)) (vector-ref source-lines index) line))
               (normal (substring (car values) indent (string-length (car values))))
               (reference (line-reference text label)))
          (loop (cdr rest) (cdr values) (+ index 1)
                (cons
                 (list (number->string (+ index 1)) (car raw) text normal
                       (without-reference text reference) (without-reference normal (line-reference normal label))
                       (number->string indent) (cadr line)
                       (number->string (caddr raw)) (number->string (cadddr raw))
                       (if reference (number->string (+ 1 (car reference))) "0")
                       (if reference (number->string (+ 1 (cadr reference))) "0")
                       (if reference (list-ref reference 3) "") (if reference (list-ref reference 4) ""))
                 rows)))))))

(def (org-dynamic-content-facts text)
  (let* ((lines (org-physical-lines text)) (body (if (null? lines) '() (cdr lines))))
    (list (number->string (length body))
          (if (ormap (lambda (line) (not (string=? (org-trim (car line)) ""))) body) "true" "false"))))
