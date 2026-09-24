;;; -*- Gerbil -*-
;;; Source-backed inline Org links inside text lines.

(import (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure-text text-line-inline-link
                 inline-link-opening inline-link-separator inline-link-closing
                 inline-link-node inline-link-target-token
                 inline-link-description-token inline-link-trivia-token)
        (only-in "../../parser.ss" org-v1-line-structure)
        (only-in "funs.ss" token-if-nonempty))
(export emit-org-text-line)

(def link-rule
  (text-line-inline-link (line-structure-text org-v1-line-structure)))
(def link-open (string->utf8 (inline-link-opening link-rule)))
(def link-separator (string->utf8 (inline-link-separator link-rule)))
(def link-close (string->utf8 (inline-link-closing link-rule)))

(def (pattern-at? bytes offset end pattern)
  (let (size (u8vector-length pattern))
    (and (<= (+ offset size) end)
         (let loop ((index 0))
           (or (= index size)
               (and (= (u8vector-ref bytes (+ offset index))
                       (u8vector-ref pattern index))
                    (loop (+ index 1))))))))

(def (find-pattern bytes start end pattern)
  (let loop ((offset start))
    (cond
     ((> (+ offset (u8vector-length pattern)) end) #f)
     ((pattern-at? bytes offset end pattern) offset)
     (else (loop (+ offset 1))))))

(def (emit-events reversed events)
  (foldl cons reversed events))

(def (emit-link reversed cursor open target-start target-end
                separator close next)
  (emit-events
   reversed
   (append
    (token-if-nonempty 'TextLine cursor open)
    (list (list 'start (inline-link-node link-rule))
          (list 'token (inline-link-trivia-token link-rule)
                open target-start)
          (list 'token (inline-link-target-token link-rule)
                target-start target-end))
    (if separator
      (append
       (list (list 'token (inline-link-trivia-token link-rule)
                   separator (+ separator (u8vector-length link-separator))))
       (token-if-nonempty (inline-link-description-token link-rule)
                          (+ separator (u8vector-length link-separator))
                          close))
      '())
    (list (list 'token (inline-link-trivia-token link-rule)
                close next)
          '(finish)))))

(def (emit-org-text-line reversed bytes start end)
  (let loop ((cursor start) (events (cons '(start OrgTextLine) reversed)))
    (let (open (find-pattern bytes cursor end link-open))
      (if (not open)
        (cons '(finish)
              (emit-events events (token-if-nonempty 'TextLine cursor end)))
        (let* ((target-start (+ open (u8vector-length link-open)))
               (close (find-pattern bytes target-start end link-close))
               (separator (and close
                               (find-pattern bytes target-start close
                                             link-separator)))
               (target-end (or separator close)))
          (if (or (not close) (= target-start target-end))
            (cons '(finish)
                  (emit-events events
                               (token-if-nonempty 'TextLine cursor end)))
            (let (next (+ close (u8vector-length link-close)))
              (loop next (emit-link events cursor open
                                    target-start target-end separator
                                    close next)))))))))
