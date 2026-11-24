;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; Single canonical Org syntax emitter owned by Orgize.
;;; Downstream consumers never hand-concatenate Org headings, drawers, or results.

(import (only-in :clan/poo/object .o .ref)
        (only-in :clan/poo/type validate)
        (only-in :std/string/misc string-trim)
        (only-in "types.ss" +org-edit-schema+ +org-section-spec-kind+
                 OrgSectionSpec org-section-spec?))
(export make-org-section-spec
        format-org-tags
        format-org-headline-line
        format-org-planning-line
        format-org-property-line
        format-org-property-drawer
        format-org-section
        format-org-babel-results)

(def (join-non-empty parts sep)
  (let ((kept (filter (lambda (p) (and (string? p) (> (string-length p) 0))) parts)))
    (if (null? kept)
      ""
      (let loop ((rest (cdr kept)) (acc (car kept)))
        (if (null? rest)
          acc
          (loop (cdr rest) (string-append acc sep (car rest))))))))

(def (make-org-section-spec level-val title-val
                            todo: (todo-val #f)
                            priority: (priority-val #f)
                            cookie: (cookie-val #f)
                            tags: (tags-val '())
                            planning: (planning-val '())
                            properties: (properties-val '())
                            body: (body-val #f))
  (unless (and (exact-integer? level-val) (> level-val 0))
    (error "Org section level must be a positive integer" level-val))
  (validate OrgSectionSpec
            (.o bind-these: #f
                kind: +org-section-spec-kind+
                schema: +org-edit-schema+
                level: level-val
                title: title-val
                todo: todo-val
                priority: priority-val
                cookie: cookie-val
                tags: tags-val
                planning: planning-val
                properties: properties-val
                body: body-val)))

(def (format-org-tags tags)
  (let ((clean (filter (lambda (t) (and (string? t) (> (string-length (string-trim t)) 0)))
                       tags)))
    (if (null? clean)
      ""
      (string-append ":" (join-non-empty (map string-trim clean) ":") ":"))))

(def (format-org-headline-line level title
                               todo: (todo #f)
                               priority: (priority #f)
                               cookie: (cookie #f)
                               tags: (tags '()))
  (let* ((stars (make-string (max 1 level) #\*))
         (todo-str (and (string? todo) (> (string-length (string-trim todo)) 0)
                        (string-trim todo)))
         (prio-str (and (string? priority) (> (string-length (string-trim priority)) 0)
                        (string-append "[#" (string-trim priority) "]")))
         (cookie-str (and (string? cookie) (> (string-length (string-trim cookie)) 0)
                          (string-trim cookie)))
         (tag-str (format-org-tags tags)))
    (join-non-empty (list stars todo-str prio-str (string-trim title) cookie-str tag-str) " ")))

(def (format-org-timestamp key ts)
  (let ((trimmed (string-trim ts)))
    (if (and (> (string-length trimmed) 0)
             (memv (string-ref trimmed 0) '(#\[ #\<)))
      trimmed
      (if (string-ci=? key "CLOSED")
        (string-append "[" trimmed "]")
        (string-append "<" trimmed ">")))))

(def (format-org-planning-line key timestamp)
  (string-append (string-upcase (string-trim key)) ": "
                 (format-org-timestamp key timestamp)))

(def (format-org-property-line key value)
  (string-append ":" (string-trim key) ": " (string-trim value)))

(def (format-org-property-drawer prop-pairs)
  (if (null? prop-pairs)
    ""
    (string-append
     ":PROPERTIES:\n"
     (join-non-empty
      (map (lambda (pair) (format-org-property-line (car pair) (cdr pair)))
           prop-pairs)
      "\n")
     "\n:END:\n")))

(def (format-org-section spec)
  (unless (org-section-spec? spec)
    (error "format-org-section requires a validated OrgSectionSpec" spec))
  (let* ((head (format-org-headline-line (.ref spec 'level)
                                         (.ref spec 'title)
                                         todo: (.ref spec 'todo)
                                         priority: (.ref spec 'priority)
                                         cookie: (.ref spec 'cookie)
                                         tags: (.ref spec 'tags)))
         (planning-lines
          (map (lambda (p) (format-org-planning-line (car p) (cdr p)))
               (.ref spec 'planning)))
         (drawer (format-org-property-drawer (.ref spec 'properties)))
         (body (.ref spec 'body))
         (body-str (if (and (string? body) (> (string-length body) 0))
                     (if (char=? (string-ref body (- (string-length body) 1)) #\newline)
                       body
                       (string-append body "\n"))
                     "")))
    (string-append
     head "\n"
     (if (null? planning-lines)
       ""
       (string-append (join-non-empty planning-lines "\n") "\n"))
     drawer
     body-str)))

(def (format-org-babel-results block-name body-text (thinking-text #f))
  (let ((clean-name (string-trim block-name))
        (clean-body (string-trim body-text))
        (clean-thinking (and (string? thinking-text) (string-trim thinking-text))))
    (string-append
     "#+RESULTS: " clean-name "\n"
     ":RESULTS:\n"
     (if (and clean-thinking (> (string-length clean-thinking) 0))
       (string-append "#+BEGIN_THINKING\n" clean-thinking "\n#+END_THINKING\n")
       "")
     clean-body
     (if (> (string-length clean-body) 0) "\n" "")
     ":END:\n")))
