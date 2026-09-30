;;; -*- Gerbil -*-
;;; POO-declared Org headlines, planning, and keyed lines lower to event IR.

(import (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure-heading line-structure-key-lines
                 key-line-node key-line-prefix key-line-keys key-line-separator
                 key-line-key-token key-line-value-token key-line-trivia-token
                 heading-line-marker heading-line-separator)
        (only-in "../../parser.ss" org-v1-line-structure)
        (only-in "event-paragraph.ss"
                 paragraph-close-form paragraph-line-form)
        (only-in "event-source-header.ss" event-source-header-forms)
        (only-in "event-table-formula.ss" table-formula-marker)
        (only-in "event-headline-tags.ss" event-headline-title-forms))
(export headline-form headline-line-forms planning-start-condition
        heading-marker heading-separator
        ascii-ci-pattern-at offset-after)

(def (key-line-by-node node)
  (or (ormap (lambda (rule) (and (eq? (key-line-node rule) node) rule))
             (line-structure-key-lines org-v1-line-structure))
      (error "missing Org key-line declaration" node)))

(def heading-rule (line-structure-heading org-v1-line-structure))
(def heading-marker (heading-line-marker heading-rule))
(def heading-separator (heading-line-separator heading-rule))
(def keyword-rule (key-line-by-node 'OrgKeyword))
(def babel-call-rule (key-line-by-node 'OrgBabelCall))
(def planning-rule (key-line-by-node 'OrgPlanning))
(def clock-rule (key-line-by-node 'OrgClock))
(def keyword-prefix (key-line-prefix keyword-rule))
(def babel-call-marker
  (let (keys (key-line-keys babel-call-rule))
    (unless (and (pair? keys) (null? (cdr keys)))
      (error "Org Babel CALL requires one declared key" keys))
    (string-append (key-line-prefix babel-call-rule)
                   (car keys) (key-line-separator babel-call-rule))))
(def close-paragraph paragraph-close-form)
(def paragraph-form paragraph-line-form)
(def rich-keyword-markers
  (map (lambda (key)
         (string-append keyword-prefix key
                        (key-line-separator keyword-rule)))
       '("TITLE" "SUBTITLE" "AUTHOR" "DATE" "CAPTION" "DESCRIPTION")))
(def rich-keyword-condition
  `(or ,@(map (lambda (marker)
               `(line-starts-with-ascii-ci ,marker))
             rich-keyword-markers)))

(def (keyword-node-forms key-end value-prefix-end optional-start optional-end)
  (let* ((value-start `(line-skip-horizontal ,value-prefix-end))
         (value-end `(line-trim-end-from ,value-start)))
    `(,close-paragraph
      (if (line-starts-with-ascii-ci ,babel-call-marker)
          ((start-node OrgBabelCall)) ((start-node OrgKeyword)))
      (token KeywordTrivia start (line-prefix-end ,keyword-prefix))
      (token KeywordKey (line-prefix-end ,keyword-prefix) ,key-end)
      ,@(if optional-start
          `((token KeywordTrivia ,key-end ,optional-start)
            (token KeywordOptional ,optional-start ,optional-end)
            (token KeywordTrivia ,optional-end ,value-prefix-end))
          `((token KeywordTrivia ,key-end ,value-prefix-end)))
      (start-node OrgKeywordRawValue)
      (token KeywordTrivia ,value-prefix-end ,value-start)
      (if (and (state table-open)
               (line-starts-with-ascii-ci ,table-formula-marker))
          ((start-node OrgTableFormulaValue)
           (call-source-helper table-formula-assignments
                               ,value-start ,value-end)
           (finish-node))
          ((if ,rich-keyword-condition
          ((start-node OrgKeywordValue)
           (call-source-helper inline-span ,value-start ,value-end
                               ((state inline-script-policy)))
           (finish-node))
          ((if (line-starts-with-ascii-ci "#+TAGS:")
               ((call-source-helper tag-vocabulary ,value-start ,value-end))
               ((if (line-starts-with-ascii-ci "#+INCLUDE:")
               ((call-source-helper include-value ,value-start ,value-end))
               ((if (line-starts-with-ascii-ci "#+ATTR_")
                    ((start-node OrgKeywordAttributes)
                     ,@(event-source-header-forms value-start value-end value-end)
                     (finish-node))
                    ((if (or (line-starts-with-ascii-ci "#+HEADER")
                             (line-starts-with-ascii-ci "#+PROPERTY:"))
                         ((start-node OrgSourceHeaderArgs)
                          ,@(event-source-header-forms value-start value-end value-end)
                          (finish-node))
                         ((token KeywordValue ,value-start ,value-end)))))))))))))
      (finish-node)
      (token KeywordTrivia ,value-end end)
      (finish-node))))

(def (bare-readonly-forms key-start key-end)
  `(,close-paragraph
    (start-node OrgKeyword)
    (token KeywordTrivia start ,key-start)
    (token KeywordKey ,key-start ,key-end)
    (start-node OrgKeywordRawValue)
    (finish-node)
    (token KeywordTrivia ,key-end end)
    (finish-node)))

(def (keyword-form)
  (let* ((key-start `(line-prefix-end ,keyword-prefix))
         (key-end `(line-scan-key ,key-start))
         (optional-start `(line-step ,key-end))
         (optional-end `(line-scan-nonspace-until ,optional-start "]"))
         (optional-separator `(line-step ,optional-end))
         (optional-value-start `(line-step ,optional-separator))
         (readonly-end '(line-prefix-end "#+READONLY")))
    `(if (and (line-starts-with-ascii-ci "#+READONLY")
              (line-bytes-all-in? ,readonly-end (line-content-end) (9 32)))
         ,(bare-readonly-forms key-start readonly-end)
         ((if (line-has-key-after-prefix? ,keyword-prefix)
              ,(keyword-node-forms key-end `(line-step ,key-end) #f #f)
              ((if (and (line-starts-with-ascii-ci ,keyword-prefix)
                        (offset-less? ,key-start ,key-end)
                        (line-byte-equal? ,key-end 91)
                        (offset-less? ,optional-start ,optional-end)
                        (line-byte-equal? ,optional-end 93)
                        (line-byte-equal? ,optional-separator 58))
                   ,(keyword-node-forms key-end optional-value-start
                                        optional-start optional-end)
                   (,(paragraph-form)))))))))

(def (declared-key-form rule key otherwise)
  (let* ((marker (string-append (key-line-prefix rule) key
                                (key-line-separator rule)))
         (key-end `(line-prefix-end ,key))
         (value-start `(line-skip-horizontal (line-prefix-end ,marker))))
    `(if (line-starts-with-ascii-ci ,marker)
         (,close-paragraph
          (start-node ,(key-line-node rule))
          (token ,(key-line-key-token rule) start ,key-end)
          (token ,(key-line-trivia-token rule) ,key-end ,value-start)
          (token ,(key-line-value-token rule) ,value-start
                 (line-trim-end-from ,value-start))
          (token ,(key-line-trivia-token rule)
                 (line-trim-end-from ,value-start) end)
          (finish-node))
         (,otherwise))))

(def (declared-key-chain rule otherwise)
  (foldr (lambda (key next) (declared-key-form rule key next))
         otherwise (key-line-keys rule)))

(def (offset-after from count)
  (let loop ((offset from) (remaining count))
    (if (= remaining 0) offset
      (loop `(line-step ,offset) (- remaining 1)))))

(def (ascii-ci-pattern-at from pattern)
  (let loop ((rest (string->list pattern)) (offset from) (predicates []))
    (if (null? rest)
      (cons 'and (reverse predicates))
      (let* ((upper (char->integer (char-upcase (car rest))))
             (lower (char->integer (char-downcase (car rest))))
             (check (if (= upper lower)
                      `(line-byte-equal? ,offset ,upper)
                      `(line-bytes-any-in? ,offset (line-step ,offset)
                                           (,upper ,lower)))))
        (loop (cdr rest) `(line-step ,offset) (cons check predicates))))))

(def planning-index '(line-index planning-byte-index))
(def planning-next `(line-step ,planning-index))
(def planning-value-start '(state-offset planning-value-start))
(def planning-value-end '(state-offset planning-value-end))

(def (planning-following-key-form key otherwise)
  (let* ((marker (string-append key (key-line-separator planning-rule)))
         (key-start planning-next)
         (key-end (offset-after key-start (string-length key)))
         (marker-end (offset-after key-start (string-length marker)))
         (value-start `(line-skip-horizontal ,marker-end)))
    `(if (and ,(ascii-ci-pattern-at key-start marker)
              (or (line-bytes-all-in? ,marker-end (line-content-end) ())
                  (line-bytes-any-in? ,marker-end
                                      (line-step ,marker-end) (9 32))))
         ((if (offset-less? ,planning-value-start ,planning-value-end)
              ((start-node OrgPlanningValue)
               (call-source-helper timestamp-candidate
                                   ,planning-value-start ,planning-value-end)
               (finish-node)) ())
          (token ,(key-line-trivia-token planning-rule)
                 ,planning-value-end ,key-start)
          (token ,(key-line-key-token planning-rule) ,key-start ,key-end)
          (token ,(key-line-trivia-token planning-rule)
                 ,key-end ,value-start)
          (set-uint planning-value-start (offset ,value-start))
          (set-uint planning-value-end (offset ,value-start)))
         ,(if (null? otherwise) '() (list otherwise)))))

(def planning-following-key-chain
  (foldr (lambda (key next) (planning-following-key-form key next))
         '() (key-line-keys planning-rule)))

(def planning-start-condition
  (cons 'or
        (map (lambda (key)
               `(line-starts-with-ascii-ci
                 ,(string-append (key-line-prefix planning-rule) key
                                 (key-line-separator planning-rule))))
             (key-line-keys planning-rule))))

(def (planning-first-key-form key otherwise)
  (let* ((marker (string-append key (key-line-separator planning-rule)))
         (key-end `(line-prefix-end ,key))
         (value-start `(line-skip-horizontal (line-prefix-end ,marker))))
    `(if (line-starts-with-ascii-ci ,marker)
         (,close-paragraph
          (start-node ,(key-line-node planning-rule))
          (token ,(key-line-key-token planning-rule) start ,key-end)
          (token ,(key-line-trivia-token planning-rule)
                 ,key-end ,value-start)
          (if (offset-less? ,planning-value-start ,value-start)
              ((set-uint planning-value-start (offset ,value-start))) ())
          (if (offset-less? ,planning-value-end ,value-start)
              ((set-uint planning-value-end (offset ,value-start))) ())
          (for-line-bytes planning-byte-index ,value-start (line-content-end)
            ((if (line-bytes-any-in? ,planning-index ,planning-next (9 32))
                 (,planning-following-key-chain) ())
             (if (and (offset-less? ,planning-value-start ,planning-next)
                      (not (line-bytes-any-in?
                            ,planning-index ,planning-next (9 32))))
                 ((set-uint planning-value-end (offset ,planning-next))) ())))
          (if (offset-less? ,planning-value-start ,planning-value-end)
              ((start-node OrgPlanningValue)
               (call-source-helper timestamp-candidate
                                   ,planning-value-start ,planning-value-end)
               (finish-node)) ())
          (token ,(key-line-trivia-token planning-rule)
                 ,planning-value-end end)
          (finish-node))
         (,otherwise))))

(def (planning-first-key-chain otherwise)
  (foldr planning-first-key-form otherwise (key-line-keys planning-rule)))

(def (clock-value-forms value-start timestamp-end value-end)
  (let* ((after-timestamp `(line-skip-horizontal ,timestamp-end))
         (duration-start
          `(line-skip-horizontal ,(offset-after after-timestamp 2))))
    `((start-node OrgClockValue)
      (call-source-helper timestamp-candidate ,value-start ,timestamp-end)
      (if ,(ascii-ci-pattern-at after-timestamp "=>")
          ((token ,(key-line-trivia-token clock-rule)
                  ,timestamp-end ,duration-start)
           (token ClockDuration ,duration-start ,value-end))
          ((token ,(key-line-trivia-token clock-rule)
                  ,timestamp-end ,value-end)))
      (finish-node))))

(def (clock-form otherwise)
  (let* ((marker (string-append "CLOCK" (key-line-separator clock-rule)))
         (key-end '(line-prefix-end "CLOCK"))
         (value-start `(line-skip-horizontal (line-prefix-end ,marker)))
         (value-end `(line-trim-end-from ,value-start))
         (first-close `(line-scan-until ,value-start "]"))
         (after-first `(line-step ,first-close))
         (second-open (offset-after after-first 2))
         (second-close `(line-scan-until ,second-open "]")))
    `(if (line-starts-with-ascii-ci ,marker)
         (,close-paragraph
          (start-node OrgClock)
          (token ,(key-line-key-token clock-rule) start ,key-end)
          (token ,(key-line-trivia-token clock-rule) ,key-end ,value-start)
          (if (and (line-byte-equal? ,value-start 91)
                   (line-byte-equal? ,first-close 93))
              ((if (and ,(ascii-ci-pattern-at after-first "--[")
                        (line-byte-equal? ,second-close 93))
                   ,(clock-value-forms value-start
                                       `(line-step ,second-close) value-end)
                   ,(clock-value-forms value-start after-first value-end)))
              ((start-node OrgClockValue)
               (token TextLine ,value-start ,value-end)
               (finish-node)))
          (token ,(key-line-trivia-token clock-rule) ,value-end end)
          (finish-node))
         (,otherwise))))

(def (context-key-form)
  (clock-form
   `(if (state after-heading)
        (,(planning-first-key-chain (keyword-form)))
        (,(keyword-form)))))

(def (headline-line-forms)
  `((start-node OrgHeadline)
    (token HeadlineLine start
           (line-marker-end ,heading-marker ,heading-separator))
    (token HeadlineTrivia
           (line-marker-end ,heading-marker ,heading-separator)
           (line-skip-horizontal
            (line-marker-end ,heading-marker ,heading-separator)))
    ,@(event-headline-title-forms
       `(line-skip-horizontal
         (line-marker-end ,heading-marker ,heading-separator))
       `(line-trim-end-from
         (line-skip-horizontal
          (line-marker-end ,heading-marker ,heading-separator))))
    (token HeadlineTrivia
           (line-trim-end-from
            (line-skip-horizontal
             (line-marker-end ,heading-marker ,heading-separator))) end)
    (finish-node)))

(def (headline-form (levels 'open-levels))
  `(if (and (uint-equal? (stack-top container-frames) (uint 0))
            (uint-positive? (line-marker-level ,heading-marker ,heading-separator)))
       (,close-paragraph
        (close-through ,levels
                       (line-marker-level ,heading-marker ,heading-separator))
        (open-level ,levels
                    (line-marker-level ,heading-marker ,heading-separator)
                    OrgSection)
        ,@(headline-line-forms)
        (set-bool after-heading (bool #t)))
       (,(context-key-form)
        (set-bool after-heading (bool #f)))))
