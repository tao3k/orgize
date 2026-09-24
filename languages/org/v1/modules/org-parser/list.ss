;;; -*- Gerbil -*-
;;; Org-owned list marker recognition and nested Element event projection.

(import (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure-list list-line-unordered-markers
                 list-line-ordered list-line-tab-width
                 list-line-list-node list-line-item-node
                 list-line-bullet-token list-line-trivia-token)
        (only-in "../../parser.ss" org-v1-line-structure)
        (only-in "funs.ss" line-content-end token-if-nonempty)
        (only-in "link.ss" emit-org-text-line))
(export org-list-marker consume-org-list)

(def list-rule (line-structure-list org-v1-line-structure))
(def unordered-bytes (string->utf8 (list-line-unordered-markers list-rule)))

(def (byte-member? byte bytes)
  (let loop ((index 0))
    (and (< index (u8vector-length bytes))
         (or (= byte (u8vector-ref bytes index))
             (loop (+ index 1))))))

(def (horizontal? byte)
  (memv byte '(9 32)))

(def (digit? byte)
  (<= 48 byte 57))

(def (alpha? byte)
  (or (<= 65 byte 90) (<= 97 byte 122)))

(def (indent-column bytes start end)
  (let loop ((offset start) (column 0))
    (if (and (< offset end) (horizontal? (u8vector-ref bytes offset)))
      (loop (+ offset 1)
            (if (= (u8vector-ref bytes offset) 9)
              (* (+ (quotient column (list-line-tab-width list-rule)) 1)
                 (list-line-tab-width list-rule))
              (+ column 1)))
      (values offset column))))

(def (ordered-end bytes first end)
  (let (byte (u8vector-ref bytes first))
    (and (list-line-ordered list-rule)
         (or (digit? byte) (alpha? byte))
         (let ((after (if (digit? byte)
                        (let loop ((offset (+ first 1)))
                          (if (and (< offset end)
                                   (digit? (u8vector-ref bytes offset)))
                            (loop (+ offset 1)) offset))
                        (+ first 1))))
           (and (< after end)
                (memv (u8vector-ref bytes after) '(41 46))
                (+ after 1))))))

(def (org-list-marker bytes span)
  (let* ((start (car span))
         (end (line-content-end bytes span)))
    (let-values (((bullet-start column) (indent-column bytes start end)))
      (and (< bullet-start end)
           (let* ((first (u8vector-ref bytes bullet-start))
                  (unordered? (byte-member? first unordered-bytes))
                  (bullet-end
                   (if unordered? (+ bullet-start 1)
                       (ordered-end bytes bullet-start end))))
             (and bullet-end
                  (or (= bullet-end end)
                      (horizontal? (u8vector-ref bytes bullet-end)))
                  (let loop ((offset bullet-end))
                    (if (and (< offset end)
                             (horizontal? (u8vector-ref bytes offset)))
                      (loop (+ offset 1))
                      (vector column (not unordered?) bullet-start
                              bullet-end offset)))))))))

(def (close-list frames reversed)
  (values (cdr frames)
          (cons '(finish) (cons '(finish) reversed))))

(def (close-deeper-lists frames reversed column ordered?)
  (if (and (pair? frames)
           (or (> (caar frames) column)
               (and (= (caar frames) column)
                    (not (eq? (cdar frames) ordered?)))))
    (let-values (((remaining events) (close-list frames reversed)))
      (close-deeper-lists remaining events column ordered?))
    (values frames reversed)))

(def (emit-list-item reversed bytes span marker)
  (let* ((start (car span))
         (end (cdr span))
         (bullet-start (vector-ref marker 2))
         (bullet-end (vector-ref marker 3))
         (content-start (vector-ref marker 4))
         (events
          (append
           (token-if-nonempty (list-line-trivia-token list-rule)
                              start bullet-start)
           (list (list 'token (list-line-bullet-token list-rule)
                       bullet-start bullet-end))
           (token-if-nonempty (list-line-trivia-token list-rule)
                              bullet-end content-start))))
    (let (item (foldl cons (cons (list 'start (list-line-item-node list-rule))
                                 reversed)
                      events))
      (if (< content-start (line-content-end bytes span))
        (values (emit-org-text-line (cons '(start OrgParagraph) item)
                                    bytes content-start end)
                #t)
        (values (foldl cons item
                       (token-if-nonempty (list-line-trivia-token list-rule)
                                          content-start end))
                #f)))))

(def (blank-line? bytes span)
  (let loop ((offset (car span)))
    (or (= offset (cdr span))
        (and (memv (u8vector-ref bytes offset) '(9 10 13 32))
             (loop (+ offset 1))))))

(def (close-list-run frames reversed paragraph?)
  (let close ((open frames)
              (events (if paragraph? (cons '(finish) reversed) reversed)))
    (if (null? open) events
        (let-values (((remaining closed) (close-list open events)))
          (close remaining closed)))))

(def (consume-org-list bytes spans reversed)
  (let loop ((rest spans) (frames []) (events reversed)
             (paragraph? #f) (blank-count 0))
    (let (marker (and (pair? rest) (org-list-marker bytes (car rest))))
      (cond
       ((null? rest)
        (values rest (close-list-run frames events paragraph?)))
       (marker
        (let* ((column (vector-ref marker 0))
               (ordered? (vector-ref marker 1))
               (finished (if paragraph? (cons '(finish) events) events)))
          (let-values (((open closed)
                        (close-deeper-lists frames finished column ordered?)))
            (let* ((same? (and (pair? open) (= (caar open) column)))
                   (parent (if same? (cons '(finish) closed) closed))
                   (next-open (if same? open
                                (cons (cons column ordered?) open)))
                   (list-events
                    (if same? parent
                        (cons (list 'start (list-line-list-node list-rule))
                              parent))))
              (let-values (((item has-paragraph?)
                            (emit-list-item list-events bytes
                                            (car rest) marker)))
                (loop (cdr rest) next-open item has-paragraph? 0))))))
       ((blank-line? bytes (car rest))
        (if (= blank-count 0)
          (loop (cdr rest) frames
                (cons (list 'token (list-line-trivia-token list-rule)
                            (caar rest) (cdar rest))
                      (if paragraph? (cons '(finish) events) events))
                #f 1)
          (values rest (close-list-run frames events paragraph?))))
       (else
        (let-values (((indent-end column)
                      (indent-column bytes (caar rest)
                                     (line-content-end bytes (car rest)))))
          (if (> column (caar frames))
            (loop (cdr rest) frames
                  (emit-org-text-line
                   (if paragraph? events
                       (cons '(start OrgParagraph) events))
                   bytes (caar rest) (cdar rest))
                  #t 0)
            (values rest (close-list-run frames events paragraph?)))))))))
