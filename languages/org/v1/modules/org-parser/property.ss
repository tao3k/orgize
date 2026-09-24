;;; -*- Gerbil -*-
;;; Source-backed node properties for Org's declared property drawer.

(import (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 block-line-block-node block-line-begin-token
                 block-line-body-line block-line-end-token
                 key-value-line-marker key-value-line-node
                 key-value-line-key-token key-value-line-value-token
                 key-value-line-trivia-token)
        (only-in "funs.ss" skip-horizontal line-content-end
                 token-if-nonempty))
(export parse-property-drawer emit-property-drawer)

(def (property-parts bytes span rule)
  (let* ((start (car span))
         (end (cdr span))
         (content-end (line-content-end bytes span))
         (marker (u8vector-ref (string->utf8
                                (key-value-line-marker rule)) 0))
         (prefix (skip-horizontal bytes start content-end))
         (key-start (+ prefix 1)))
    (and (< key-start content-end)
         (= (u8vector-ref bytes prefix) marker)
         (let scan ((offset key-start))
           (and (< offset content-end)
                (let (byte (u8vector-ref bytes offset))
                  (cond
                   ((= byte marker)
                    (let ((after (+ offset 1)))
                      (and (or (= after content-end)
                               (memv (u8vector-ref bytes after) '(9 32)))
                           (let* ((value-start
                                   (skip-horizontal bytes after content-end))
                                  (value-end
                                   (let trim ((tail content-end))
                                     (if (and (> tail value-start)
                                              (memv (u8vector-ref bytes (- tail 1))
                                                    '(9 32)))
                                       (trim (- tail 1))
                                       tail))))
                             (list key-start offset value-start value-end)))))
                   ((memv byte '(9 32)) #f)
                   (else (scan (+ offset 1))))))))))

(def (parse-property-drawer bytes pending block)
  (let* ((lines (reverse pending))
         (rule (block-line-body-line block)))
    (and rule
         (let loop ((rest (cdr lines)) (reversed '()))
           (if (null? rest)
             (cons (car lines) (reverse reversed))
             (let (parts (property-parts bytes (car rest) rule))
               (and parts
                    (loop (cdr rest)
                          (cons (cons (car rest) parts) reversed)))))))))

(def (emit-property-line reversed entry rule)
  (let* ((span (car entry))
         (parts (cdr entry))
         (key-start (car parts))
         (key-end (cadr parts))
         (value-start (caddr parts))
         (value-end (cadddr parts)))
    (foldl cons reversed
           (append
            (list (list 'start (key-value-line-node rule)))
            (token-if-nonempty (key-value-line-trivia-token rule)
                               (car span) key-start)
            (list (list 'token (key-value-line-key-token rule)
                        key-start key-end))
            (token-if-nonempty (key-value-line-trivia-token rule)
                               key-end value-start)
            (token-if-nonempty (key-value-line-value-token rule)
                               value-start value-end)
            (token-if-nonempty (key-value-line-trivia-token rule)
                               value-end (cdr span))
            (list '(finish))))))

(def (emit-property-drawer reversed block parsed closing)
  (let* ((opening (car parsed))
         (body (cdr parsed))
         (rule (block-line-body-line block))
         (opened (foldl cons reversed
                        (list (list 'start (block-line-block-node block))
                              (list 'token (block-line-begin-token block)
                                    (car opening) (cdr opening)))))
         (filled (foldl (lambda (entry events)
                          (emit-property-line events entry rule))
                        opened body)))
    (foldl cons filled
           (list (list 'token (block-line-end-token block)
                       (car closing) (cdr closing))
                 '(finish)))))
