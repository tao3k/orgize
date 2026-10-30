;;; -*- Gerbil -*-
;;; Native source-backed semantic content, with lossless outer trivia.

(export source-content-initial source-content-forms source-space-at?)

(def source-content-initial
  '((content-first 0) (content-last 0) (content-skip 0) (content-seen #f)))
(def index '(line-index content-byte-index))
(def next `(line-step ,index))
(def first '(state-offset content-first))
(def last '(state-offset content-last))

;; Unicode White_Space, including the complete UTF-8 codepoint. Never classify
;; continuation bytes individually or probe outside the admitted helper slice.
(def unicode-space-bytes
  '((194 133) (194 160) (225 154 128)
    (226 128 128) (226 128 129) (226 128 130) (226 128 131)
    (226 128 132) (226 128 133) (226 128 134) (226 128 135)
    (226 128 136) (226 128 137) (226 128 138)
    (226 128 168) (226 128 169) (226 128 175)
    (226 129 159) (227 128 128)))

(def (bytes-at? from until bytes)
  (let loop ((rest bytes) (cursor from))
    (if (null? rest) '(bool #t)
        `(and (offset-less? ,cursor ,until)
              (line-byte-equal? ,cursor ,(car rest))
              ,(loop (cdr rest) `(line-step ,cursor))))))

(def (source-space-at? from until)
  `(or (line-bytes-any-in? ,from (line-step ,from) (9 10 11 12 13 32))
       ,@(map (lambda (bytes) (bytes-at? from until bytes)) unicode-space-bytes)))

(def (space-chain choices otherwise)
  (if (null? choices) otherwise
      (let* ((bytes (car choices))
             (offsets (let loop ((rest bytes) (cursor index))
                        (if (null? rest) '()
                            (cons cursor (loop (cdr rest) `(line-step ,cursor))))))
             (after (foldl (lambda (_ cursor) `(line-step ,cursor)) index bytes)))
        `((if (and ,@(map (lambda (offset byte)
                           `(and (offset-less? ,offset end)
                                 (line-byte-equal? ,offset ,byte))) offsets bytes))
              ((set-uint content-skip (offset ,after)))
              ,(space-chain (cdr choices) otherwise))))))

;; `body` is a pure declaration factory receiving exact semantic bounds.
(def (source-content-forms node trivia body (trim-right? #t))
  `((set-uint content-first (offset end))
    (set-uint content-last (offset end))
    (set-uint content-skip (offset start))
    (for-line-bytes content-byte-index start end
      ((if (offset-less? ,index (state-offset content-skip)) ()
           ((if (line-bytes-any-in? ,index ,next (9 10 11 12 13 32)) ()
                ,(space-chain unicode-space-bytes
                   `((if (not (state content-seen))
                         ((set-uint content-first (offset ,index))
                          (set-bool content-seen (bool #t))) ())
                     (set-uint content-last (offset ,next)))))))))
    (token ,trivia start ,first)
    (start-node ,node)
    ,@(body first (if trim-right? last 'end))
    (finish-node)
    (token ,trivia ,(if trim-right? last 'end) end)))
