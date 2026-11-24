;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; Comprehensive tests for parser-span-driven Org structural editing and
;;; `property:*` declarative `org-elements` queries.

(import :std/test
        (only-in :std/string/misc string-contains)
        (only-in "../languages/org/modules/org-elements/source-interface.ss"
                 org-source-element-graph org-headline-property org-headline-level
                 org-headline-tags org-headline-has-tag? org-headline-todo-state
                 org-find-headline org-headline-checklist-stats org-graph-checklist-stats)
        (only-in "../languages/org/modules/org-elements/query.ss"
                 make-org-element-query-context org-element-select)
        (only-in "../languages/org/modules/org-elements/syntax.ss"
                 org-elements property)
        (only-in "../languages/org/modules/org-edit/func.ss"
                 make-org-section-spec
                 org-edit set-todo add-tag remove-tag
                 set-planning remove-planning
                 upsert-properties delete-properties
                 set-checkbox upsert-babel-results
                 upsert-section append-section
                 org-apply-edit org-apply-edits org-extract-subtree))
(export org-edit-test)

(def sample-task-org
  (string-append
   "* IN_PROGRESS Build Parser Edit Algebra [1/2] :core:\n"
   ":PROPERTIES:\n"
   ":ID: task-root\n"
   ":ROLE: mainline\n"
   ":END:\n"
   "- [X] Define SpanEdit\n"
   "- [ ] Wire org-edit\n"
   "\n"
   "** TODO Implement Detour Guard\n"
   ":PROPERTIES:\n"
   ":ID: det-1\n"
   ":ROLE: detour\n"
   ":END:\n"
   "\n"
   "#+NAME: eval-1\n"
   "#+BEGIN_SRC llm :tier builder\n"
   "Verify edit algebra.\n"
   "#+END_SRC\n"))

(def org-edit-test
  (test-suite "Orgize parser-owned org-elements property:* and org-edit"
    (test-case "org-elements DSL queries property:* and computes headline level"
      (let* ((g (org-source-element-graph sample-task-org))
             (ctx (make-org-element-query-context g))
             (q (org-elements headline (property property:ROLE "detour")))
             (hits (org-element-select q ctx 0 '())))
        (check (length hits) => 1)
        (check (org-headline-property ctx (car hits) "ID") => "det-1")
        (check (org-headline-level ctx (car hits)) => 2)
        (let ((root (org-find-headline ctx g 'first)))
          (check (org-headline-level ctx root) => 1)
          (check (org-headline-has-tag? ctx root "core") => #t))))

    (test-case "declarative org-edit archive and reopen round-trip cleanly"
      (let* ((archived
              (org-apply-edits
               sample-task-org
               (org-edit
                (set-todo 'first "DONE")
                (add-tag 'first "ARCHIVE")
                (set-planning 'first "CLOSED" "2026-11-24 Tue 13:00"))))
             (ga (org-source-element-graph archived))
             (ctxa (make-org-element-query-context ga))
             (roota (org-find-headline ctxa ga 'first)))
        (check (org-headline-has-tag? ctxa roota "ARCHIVE") => #t)
        (check (org-headline-has-tag? ctxa roota "core") => #t)
        (check (org-headline-todo-state ctxa roota) => 'done)
        (check (and (string-contains archived "CLOSED: [2026-11-24 Tue 13:00]") #t) => #t)
        (let* ((reopened
                (org-apply-edits
                 archived
                 (org-edit
                  (remove-tag 'first "ARCHIVE")
                  (remove-planning 'first "CLOSED")
                  (set-todo 'first "TODO"))))
               (gr (org-source-element-graph reopened))
               (ctxr (make-org-element-query-context gr))
               (rootr (org-find-headline ctxr gr 'first)))
          (check (org-headline-has-tag? ctxr rootr "ARCHIVE") => #f)
          (check (org-headline-has-tag? ctxr rootr "core") => #t)
          (check (org-headline-todo-state ctxr rootr) => 'todo)
          (check (string-contains reopened "CLOSED:") => #f))))

    (test-case "upsert-properties and delete-properties operate strictly on OrgPropertyDrawer spans"
      (let* ((updated
              (org-apply-edit
               sample-task-org
               (upsert-properties "det-1"
                                  '(("OWNER_ACTOR" . "agent-alpha")
                                    ("CLAIM_STATE" . "claimed-active")
                                    ("ROLE" . "blocker-detour")))))
             (g (org-source-element-graph updated))
             (ctx (make-org-element-query-context g))
             (det (org-find-headline ctx g "det-1")))
        (check (org-headline-property ctx det "OWNER_ACTOR") => "agent-alpha")
        (check (org-headline-property ctx det "CLAIM_STATE") => "claimed-active")
        (check (org-headline-property ctx det "ROLE") => "blocker-detour")
        (let* ((deleted (org-apply-edit updated (delete-properties "det-1" '("CLAIM_STATE"))))
               (g2 (org-source-element-graph deleted))
               (ctx2 (make-org-element-query-context g2))
               (det2 (org-find-headline ctx2 g2 "det-1")))
          (check (org-headline-property ctx2 det2 "CLAIM_STATE") => #f)
          (check (org-headline-property ctx2 det2 "OWNER_ACTOR") => "agent-alpha"))))

    (test-case "set-checkbox syncs statistics-cookie atomically"
      (let* ((updated (org-apply-edit sample-task-org (set-checkbox "task-root" 1 #t)))
             (g (org-source-element-graph updated))
             (ctx (make-org-element-query-context g)))
        (check (and (string-contains updated "[2/2]") #t) => #t)
        (let-values (((done total all?) (org-graph-checklist-stats ctx g)))
          (check done => 2)
          (check total => 2)
          (check all? => #t))))

    (test-case "upsert-babel-results inserts and idempotently replaces RESULTS drawer"
      (let* ((r1 (org-apply-edit sample-task-org
                                 (upsert-babel-results "eval-1" "first-output" "thought-1")))
             (r2 (org-apply-edit r1
                                 (upsert-babel-results "eval-1" "second-output"))))
        (check (and (string-contains r1 "#+RESULTS: eval-1") #t) => #t)
        (check (and (string-contains r1 "thought-1") #t) => #t)
        (check (and (string-contains r2 "second-output") #t) => #t)
        (check (string-contains r2 "first-output") => #f)))

    (test-case "upsert-section, append-section, and extract-subtree preserve parser validity"
      (let* ((spec (make-org-section-spec
                    1 "Task Alpha"
                    todo: "DONE"
                    cookie: "[2/2]"
                    tags: '("ARCHIVE")
                    planning: '(("CLOSED" . "2026-11-24 Tue 13:00"))
                    properties: '(("ID" . "root-alpha") ("TASK_ID" . "t-alpha"))
                    body: "[[file:tasks/t-alpha.org][Task Alpha]]"))
             (idx1 (org-apply-edit "" (upsert-section "root-alpha" spec "#+TITLE: Task Index\n\n")))
             (spec2 (make-org-section-spec
                     1 "Task Alpha"
                     todo: "TODO"
                     cookie: "[1/2]"
                     properties: '(("ID" . "root-alpha") ("TASK_ID" . "t-alpha"))
                     body: "[[file:tasks/t-alpha.org][Task Alpha]]"))
             (idx2 (org-apply-edit idx1 (upsert-section "root-alpha" spec2))))
        (check (and (string-contains idx1 "#+TITLE: Task Index") #t) => #t)
        (check (and (string-contains idx2 "* TODO Task Alpha [1/2]") #t) => #t)
        (check (string-contains idx2 ":ARCHIVE:") => #f))
      (let-values (((remaining promoted)
                    (org-extract-subtree
                     sample-task-org "det-1"
                     extra-properties: '(("CONTEXT_ORIGIN_MODULE" . "test.fission")
                                         ("ROLE" . "should-not-overwrite"))
                     preserve-existing?: #t)))
        (let* ((gp (org-source-element-graph promoted))
               (ctxp (make-org-element-query-context gp))
               (hp (org-find-headline ctxp gp "det-1")))
          (check (org-headline-property ctxp hp "CONTEXT_ORIGIN_MODULE") => "test.fission")
          (check (org-headline-property ctxp hp "ROLE") => "detour")
          (check (string-contains remaining ":ID: det-1") => #f))))

    (test-case "fail-closed negative control rejects missing headline ID"
      (check-exception
       (org-apply-edit sample-task-org (set-todo "non-existent-id" "DONE"))
       true))))
