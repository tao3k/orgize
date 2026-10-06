;;; -*- Gerbil -*-
;;; Org timestamp candidate recognition and source-backed AOT projection.

(import (only-in "event-inline-primitives.ss"
                 link-index inline-next pattern-end pattern-at?)
        (only-in "objects.ss" make-org-event-helper))
(export timestamp-event-initial timestamp-open-forms timestamp-scan-forms
        timestamp-candidate-helper timestamp-date-at?)

(def timestamp-digits
  (map char->integer (string->list "0123456789")))
(def timestamp-dayname-bytes
  (append (map char->integer
               (string->list "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"))
          (iota 128 128)))
(def timestamp-time-bytes
  (append timestamp-digits '(45 58)))
(def timestamp-cookie-units '(104 100 119 109 121))

;; The native event tree owns clock components and inline range boundaries.
;; Rust only converts these classified decimal fields to its ABI value types.
(def (timestamp-clock-events from until hour minute)
  (let* ((colon `(line-scan-until ,from ":"))
         (minute-start `(line-step ,colon)))
    `((if (and (offset-less? ,from ,colon)
               (offset-less? ,minute-start ,until)
               (line-byte-equal? ,colon 58)
               (line-bytes-all-in? ,from ,colon ,timestamp-digits)
               (line-bytes-all-in? ,minute-start ,until ,timestamp-digits))
          ((token ,hour ,from ,colon)
           (token TimestampTimeSeparator ,colon ,minute-start)
           (token ,minute ,minute-start ,until))
          ((if (offset-less? ,from ,until)
               ((token TimestampTimeText ,from ,until)) ()))))))

(def (timestamp-time-events from until (second? #f))
  (let (dash `(line-scan-until ,from "-"))
    `((start-node ,(if second? 'TimestampSecondTime 'TimestampTime))
      (if (offset-less? ,dash ,until)
          (,@(timestamp-clock-events from dash
                                     (if second? 'TimestampSecondHour 'TimestampHour)
                                     (if second? 'TimestampSecondMinute 'TimestampMinute))
           (token TimestampTimeRangeSeparator ,dash (line-step ,dash))
           (start-node TimestampTimeEnd)
           ,@(timestamp-clock-events `(line-step ,dash) until
                                     'TimestampEndHour 'TimestampEndMinute)
           (finish-node))
          ,(timestamp-clock-events from until
                                  (if second? 'TimestampSecondHour 'TimestampHour)
                                  (if second? 'TimestampSecondMinute 'TimestampMinute)))
      (finish-node))))

;; Classify complete cookies here, not by a permissive byte alphabet followed
;; by a Rust projection rejecting malformed signs, digits or units.
(def (timestamp-cookie-forms from until prefixes node mark value unit otherwise)
  (if (null? prefixes)
    otherwise
    (let* ((prefix (car prefixes))
           (body (pattern-end from prefix))
           (index '(line-index timestamp-cookie-byte-index))
           (next `(line-step ,index))
           (unit-at '(state-offset timestamp-cookie-unit-at)))
      `((if ,(pattern-at? from prefix)
            ((set-bool timestamp-cookie-valid (bool #t))
             (for-line-bytes timestamp-cookie-byte-index ,body ,until
               ((if (uint-equal? (offset ,next) (offset ,until))
                    ((set-uint timestamp-cookie-unit-at (offset ,index))
                     (if (line-bytes-any-in? ,index ,next ,timestamp-cookie-units)
                         () ((set-bool timestamp-cookie-valid (bool #f)))))
                    ((if (line-bytes-all-in? ,index ,next ,timestamp-digits)
                         () ((set-bool timestamp-cookie-valid (bool #f))))))))
             (if (and (state timestamp-cookie-valid)
                      (offset-less? (line-step ,body) ,until))
                 ((start-node ,node)
                  (token ,mark ,from ,body)
                  (token ,value ,body ,unit-at)
                  (token ,unit ,unit-at ,until)
                  (finish-node))
                 ,otherwise))
            ,(timestamp-cookie-forms from until (cdr prefixes)
                                     node mark value unit otherwise))))))

(def (timestamp-date-at? opening)
  (let ((year (pattern-end opening "<"))
        (year-end (pattern-end opening "<0000"))
        (month (pattern-end opening "<0000-"))
        (month-end (pattern-end opening "<0000-00"))
        (day (pattern-end opening "<0000-00-"))
        (day-end (pattern-end opening "<0000-00-00")))
    `(and (line-bytes-all-in? ,year ,year-end ,timestamp-digits)
          (line-byte-equal? ,year-end 45)
          (line-bytes-all-in? ,month ,month-end ,timestamp-digits)
          (line-byte-equal? ,month-end 45)
          (line-bytes-all-in? ,day ,day-end ,timestamp-digits)
          (offset-less? ,day-end (line-content-end)))))

(def timestamp-event-initial
  '((inline-timestamp-mode 0) (inline-timestamp-open-at 0)
    (inline-timestamp-diary-opens 0) (inline-timestamp-diary-closes 0)))

(def (timestamp-open-forms)
  `((if (and (or (line-byte-equal? ,link-index 60)
                 (line-byte-equal? ,link-index 91))
             ,(timestamp-date-at? link-index))
        ((if (line-byte-equal? ,link-index 60)
             ((set-uint inline-timestamp-mode (uint 1)))
             ((set-uint inline-timestamp-mode (uint 2))))
         (set-uint inline-timestamp-open-at (offset ,link-index)))
        ((if ,(pattern-at? link-index "<%%(")
             ((set-uint inline-timestamp-mode (uint 5))
              (set-uint inline-timestamp-open-at (offset ,link-index))
              (set-uint inline-timestamp-diary-opens (uint 0))
              (set-uint inline-timestamp-diary-closes (uint 0))) ())))))

(def (timestamp-following-range? mode opening)
  `(and (uint-equal? (state inline-timestamp-mode) (uint ,mode))
        (or (and ,(pattern-at? inline-next (string-append "--" opening))
                 ,(timestamp-date-at? (pattern-end inline-next "--")))
            (and ,(pattern-at? inline-next (string-append "-" opening))
                 ,(timestamp-date-at? (pattern-end inline-next "-"))))))

(def (timestamp-close-forms)
  `((if ,(timestamp-following-range? 1 "<")
        ((set-uint inline-timestamp-mode (uint 3)))
        ((if ,(timestamp-following-range? 2 "[")
             ((set-uint inline-timestamp-mode (uint 4)))
             ((token TextLine (state-offset inline-cursor)
                     (state-offset inline-timestamp-open-at))
              (call-source-helper timestamp-candidate
                                  (state-offset inline-timestamp-open-at)
                                  ,inline-next)
              (set-uint inline-cursor (offset ,inline-next))
              (set-uint inline-timestamp-mode (uint 0))))))))

(def (timestamp-scan-forms)
  `((if (uint-equal? (state inline-timestamp-mode) (uint 5))
        ((if (line-byte-equal? ,link-index 40)
             ((set-uint inline-timestamp-diary-opens
                        (uint-add (state inline-timestamp-diary-opens)
                                  (uint 1)))) ())
         (if (line-byte-equal? ,link-index 41)
             ((set-uint inline-timestamp-diary-closes
                        (uint-add (state inline-timestamp-diary-closes)
                                  (uint 1)))) ())) ())
    (if (line-bytes-any-in? ,link-index ,inline-next (10 13))
        ((set-uint inline-timestamp-mode (uint 0)))
        ((if (or (and (or (uint-equal? (state inline-timestamp-mode) (uint 1))
                           (uint-equal? (state inline-timestamp-mode) (uint 3)))
                      (line-byte-equal? ,link-index 62))
                 (and (or (uint-equal? (state inline-timestamp-mode) (uint 2))
                           (uint-equal? (state inline-timestamp-mode) (uint 4)))
                      (line-byte-equal? ,link-index 93))
                 (and (uint-equal? (state inline-timestamp-mode) (uint 5))
                      (line-byte-equal? ,link-index 62)
                      (uint-positive? (state inline-timestamp-diary-opens))
                      (uint-equal? (state inline-timestamp-diary-opens)
                                   (state inline-timestamp-diary-closes))))
             ,(timestamp-close-forms)
             ())))))

(def (timestamp-date-start opening) `(line-step ,opening))
(def (timestamp-date-end opening)
  (pattern-end opening "<0000-00-00"))
(def (timestamp-point-valid? opening close)
  (let (date-end (timestamp-date-end opening))
    `(and ,(timestamp-date-at? opening)
          (or (uint-equal? (offset ,date-end) (offset ,close))
              (and (line-byte-equal? ,date-end 32)
                   (offset-less? (line-step ,date-end) ,close))))))

(def timestamp-field-index '(line-index timestamp-field-byte-index))
(def timestamp-field-next `(line-step ,timestamp-field-index))
(def (timestamp-field-events until (second? #f))
  (let (from '(state-offset timestamp-field-start))
    `((if (offset-less? ,from ,until)
          ((if (line-bytes-all-in? ,from ,until
                                   ,timestamp-dayname-bytes)
               ((token ,(if second? 'TimestampSecondDayName 'TimestampDayName)
                       ,from ,until))
               ((if (and (line-bytes-all-in? ,from ,until
                                             ,timestamp-time-bytes)
                         (line-bytes-any-in? ,from ,until (58)))
                    ,(timestamp-time-events from until second?)
                    ,(timestamp-cookie-forms
                      from until '("--" "-") 'TimestampDelay
                      'TimestampDelayMark 'TimestampDelayValue 'TimestampDelayUnit
                      (timestamp-cookie-forms
                       from until '("++" ".+" "+") 'TimestampRepeater
                       'TimestampRepeaterMark 'TimestampRepeaterValue 'TimestampRepeaterUnit
                       `((token TimestampTrivia ,from ,until))))))))
          ()))))

(def (timestamp-tail-events date-end close (second? #f))
  `((if (offset-less? ,date-end ,close)
        ((if (uint-not-equal? (state timestamp-field-start)
                              (offset (line-step ,date-end)))
             ((set-uint timestamp-field-start
                        (offset (line-step ,date-end)))) ())
         (token TimestampTrivia ,date-end (line-step ,date-end))
         (for-line-bytes timestamp-field-byte-index (line-step ,date-end)
                         ,close
           ((if (line-bytes-any-in? ,timestamp-field-index
                                    ,timestamp-field-next (9 32))
                (,@(timestamp-field-events timestamp-field-index second?)
                 (token TimestampTrivia ,timestamp-field-index
                        ,timestamp-field-next)
                 (set-uint timestamp-field-start
                           (offset ,timestamp-field-next))) ())))
         ,@(timestamp-field-events close second?)) ())))
(def (timestamp-point-events opening close (second? #f))
  (let ((date-start (timestamp-date-start opening))
        (date-end (timestamp-date-end opening)))
    `((start-node OrgTimestampPoint)
      (token TimestampDelimiter ,opening (line-step ,opening))
      (start-node TimestampDate)
      (token ,(if second? 'TimestampSecondYear 'TimestampYear)
             ,date-start ,(pattern-end opening "<0000"))
      (token TimestampDateSeparator ,(pattern-end opening "<0000")
             ,(pattern-end opening "<0000-"))
      (token ,(if second? 'TimestampSecondMonth 'TimestampMonth)
             ,(pattern-end opening "<0000-")
             ,(pattern-end opening "<0000-00"))
      (token TimestampDateSeparator ,(pattern-end opening "<0000-00")
             ,(pattern-end opening "<0000-00-"))
      (token ,(if second? 'TimestampSecondDay 'TimestampDay)
             ,(pattern-end opening "<0000-00-") ,date-end)
      (finish-node)
      ,@(timestamp-tail-events date-end close second?)
      (token TimestampDelimiter ,close (line-step ,close))
      (finish-node))))

(def (timestamp-range-form closer node close after-close separator otherwise)
  (let* ((second-open (pattern-end after-close separator))
         (second-close `(line-scan-until ,second-open ,closer))
         (delimiter (char->integer (string-ref closer 0)))
         (range-marker (string-append separator
                                      (if (string=? closer ">") "<" "["))))
    `(if (and ,(pattern-at? after-close range-marker)
              ,(timestamp-point-valid? second-open second-close)
              (line-byte-equal? ,second-close ,delimiter)
              (uint-equal? (offset (line-step ,second-close)) (offset end)))
         ((start-node ,node)
          ,@(timestamp-point-events 'start close)
          (token TimestampRangeSeparator ,after-close ,second-open)
          ,@(timestamp-point-events second-open second-close #t)
          (finish-node))
         ,otherwise)))

(def (timestamp-candidate-forms closer node)
  (let* ((close `(line-scan-until start ,closer))
         (after-close `(line-step ,close))
         (delimiter (char->integer (string-ref closer 0)))
         (single-range
          (timestamp-range-form closer node close after-close "-"
                                '((token TextLine start end))))
         (range-form
          (timestamp-range-form closer node close after-close "--"
                                (list single-range))))
    `((if (and ,(timestamp-point-valid? 'start close)
               (line-byte-equal? ,close ,delimiter))
          ((if (uint-equal? (offset ,after-close) (offset end))
               ((start-node ,node)
                ,@(timestamp-point-events 'start close)
                (finish-node))
               (,range-form)))
          ((token TextLine start end))))))

(def timestamp-diary-index '(line-index timestamp-diary-byte-index))
(def timestamp-diary-next `(line-step ,timestamp-diary-index))
(def (timestamp-diary-forms)
  (let* ((close '(line-scan-until start ">"))
         (sexp-end '(state-offset timestamp-diary-sexp-end))
         (time-start `(line-skip-horizontal ,sexp-end)))
    `((for-line-bytes timestamp-diary-byte-index
                      ,(pattern-end 'start "<%%(") ,close
        ((if (line-byte-equal? ,timestamp-diary-index 40)
             ((set-uint timestamp-diary-opens
                        (uint-add (state timestamp-diary-opens) (uint 1)))) ())
         (if (line-byte-equal? ,timestamp-diary-index 41)
             ((set-uint timestamp-diary-closes
                        (uint-add (state timestamp-diary-closes) (uint 1)))
              (if (and (uint-equal? (state timestamp-diary-opens)
                                    (state timestamp-diary-closes))
                       (uint-not-equal? (state timestamp-diary-sexp-end)
                                        (offset ,timestamp-diary-next)))
                  ((set-uint timestamp-diary-sexp-end
                             (offset ,timestamp-diary-next))) ())) ())))
      (if (and (uint-positive? (state timestamp-diary-sexp-end))
               (or (uint-equal? (offset ,sexp-end) (offset ,close))
                   (and (offset-less? ,sexp-end ,time-start)
                        (offset-less? ,time-start ,close)
                        (line-bytes-all-in? ,time-start ,close
                                            ,timestamp-time-bytes)
                        (line-bytes-any-in? ,time-start ,close (58)))))
          ((start-node OrgTimestampDiary)
           (token TimestampDelimiter start (line-step start))
           (token TimestampDiaryExpression (line-step start) ,sexp-end)
           (if (offset-less? ,sexp-end ,close)
               ((token TimestampTrivia ,sexp-end ,time-start)
                ,@(timestamp-time-events time-start close)) ())
           (token TimestampDelimiter ,close end)
           (finish-node))
          ((token TextLine start end))))))

(def timestamp-candidate-helper
  (make-org-event-helper
   'timestamp-candidate '((timestamp-field-start 0)
                          (timestamp-cookie-valid #t)
                          (timestamp-cookie-unit-at 0)
                          (timestamp-diary-opens 1)
                          (timestamp-diary-closes 0)
                          (timestamp-diary-sexp-end 0))
   `((if ,(pattern-at? 'start "<%%(")
         ,(timestamp-diary-forms)
         ((if (line-byte-equal? start 60)
              ,(timestamp-candidate-forms ">" 'OrgTimestampActive)
              ,(timestamp-candidate-forms "]" 'OrgTimestampInactive)))))))
