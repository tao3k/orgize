;;; -*- Gerbil -*-
;;; Private pure Unicode text operations shared by native semantic owners.
(export org-space? org-trim org-prefix-at? org-find org-physical-lines)

(def (org-space? ch)
  (let (n (char->integer ch))
    (or (<= 9 n 13) (memv n '(32 133 160 5760 8232 8233 8239 8287 12288))
        (<= 8192 n 8202))))

(def (org-trim text)
  (let ((end (string-length text)))
    (let left ((start 0))
      (if (and (< start end) (org-space? (string-ref text start)))
        (left (+ start 1))
        (let right ((until end))
          (if (and (> until start) (org-space? (string-ref text (- until 1))))
            (right (- until 1)) (substring text start until)))))))

(def (org-prefix-at? text pattern start)
  (let ((size (string-length pattern)) (end (string-length text)))
    (and (<= (+ start size) end)
         (let loop ((index 0))
           (or (= index size)
               (and (char=? (string-ref text (+ start index)) (string-ref pattern index))
                    (loop (+ index 1))))))))

(def (org-find text pattern (start 0))
  (let ((end (- (string-length text) (string-length pattern))))
    (let loop ((index start))
      (and (<= index end)
           (if (org-prefix-at? text pattern index) index (loop (+ index 1)))))))

(def (utf8-width ch)
  (let (n (char->integer ch))
    (cond ((< n 128) 1) ((< n 2048) 2) ((< n 65536) 3) (else 4))))

;; No trailing phantom line. Offsets exclude terminators and count UTF-8 bytes.
(def (org-physical-lines text)
  (let (end (string-length text))
    (let loop ((index 0) (start 0) (byte-index 0) (byte-start 0) (rows '()))
      (if (= index end)
        (reverse (if (< start end)
                   (cons (list (substring text start end) "" byte-start byte-index) rows) rows))
        (let (ch (string-ref text index))
          (if (or (char=? ch #\newline) (char=? ch #\return))
            (let* ((width (if (and (char=? ch #\return) (< (+ index 1) end)
                                  (char=? (string-ref text (+ index 1)) #\newline)) 2 1))
                   (next (+ index width)) (next-byte (+ byte-index width)))
              (loop next next next-byte next-byte
                    (cons (list (substring text start index) (substring text index next)
                                byte-start byte-index) rows)))
            (loop (+ index 1) start (+ byte-index (utf8-width ch)) byte-start rows)))))))
