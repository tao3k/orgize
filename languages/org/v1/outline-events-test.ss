;;; -*- Gerbil -*-
;;; Parser-specific AST checks preserve nesting and source byte spans.

(import (only-in :std/test check test-case test-suite)
        (only-in :std/encoding/json JSONReadOptions string->json)
        (only-in :std/misc/ports read-all-as-string)
        (only-in "modules/org-parser/test-syntax.ss"
                 check-org-ast org-events-cover-source?)
        (only-in "outline-event-fixture.ss" outline-event-fixture-json))
(export org-v1-outline-events-test)

(def org-v1-outline-events-test
  (test-suite "Org Scheme structural AST"
    (test-case "nested and sibling sections"
      (check-org-ast "* Parent\n** Child\ntext\n* Peer\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 8) (HeadlineTrivia 8 9))
          (OrgSection
           (OrgHeadline (HeadlineLine 9 11) (HeadlineTrivia 11 12)
                        (HeadlineTitle 12 17) (HeadlineTrivia 17 18))
           (OrgParagraph (OrgTextLine (TextLine 18 23)))))
         (OrgSection
          (OrgHeadline (HeadlineLine 23 24) (HeadlineTrivia 24 25)
                       (HeadlineTitle 25 29) (HeadlineTrivia 29 30))))))
    (test-case "non-headline stars and UTF-8 byte spans"
      (check-org-ast "*not a headline\n* α\n"
        (OrgFile
         (OrgParagraph (OrgTextLine (TextLine 0 16)))
         (OrgSection
          (OrgHeadline (HeadlineLine 16 17) (HeadlineTrivia 17 18)
                       (HeadlineTitle 18 20) (HeadlineTrivia 20 21))))))
    (test-case "empty document"
      (check-org-ast "" (OrgFile)))
    (test-case "AST assertion rejects missing and overlapping source spans"
      (check (org-events-cover-source?
              "abc" '((start OrgFile) (token TextLine 0 2) (finish)))
             => #f)
      (check (org-events-cover-source?
              "abc" '((start OrgFile) (token TextLine 0 2)
                      (token TextLine 1 3) (finish)))
             => #f))
    (test-case "blank lines separate paragraphs"
      (check-org-ast "alpha\nbeta\n\nnext\n"
        (OrgFile
         (OrgParagraph (OrgTextLine (TextLine 0 6))
                       (OrgTextLine (TextLine 6 11)))
         (OrgTextLine (TextLine 11 12))
         (OrgParagraph (OrgTextLine (TextLine 12 17))))))
    (test-case "Babel CALL retains its own Element kind"
      (check-org-ast "#+CALL: build(input=42)\n"
        (OrgFile
         (OrgBabelCall
          (KeywordTrivia 0 2) (KeywordKey 2 6)
          (KeywordTrivia 6 8) (KeywordValue 8 23)
          (KeywordTrivia 23 24)))))
    (test-case "source block remains inside its section"
      (check-org-ast "* Code\n#+BEGIN_SRC rust\nα\n#+END_SRC\n** Next\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 6) (HeadlineTrivia 6 7))
          (OrgSourceBlock
           (BlockBeginLine 7 18) (BlockHeaderTrivia 18 19)
           (SourceLanguage 19 23) (BlockHeaderTrivia 23 24)
           (TextLine 24 27) (BlockEndLine 27 37))
          (OrgSection
           (OrgHeadline (HeadlineLine 37 39) (HeadlineTrivia 39 40)
                        (HeadlineTitle 40 44) (HeadlineTrivia 44 45)))))))
    (test-case "unclosed source block recovers before a headline"
      (check-org-ast "* First\n#+begin_src rust\nbody\n** Next\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 7) (HeadlineTrivia 7 8))
          (OrgParagraph
           (OrgTextLine (TextLine 8 25))
           (OrgTextLine (TextLine 25 30)))
          (OrgSection
           (OrgHeadline (HeadlineLine 30 32) (HeadlineTrivia 32 33)
                        (HeadlineTitle 33 37) (HeadlineTrivia 37 38)))))))
    (test-case "Rowan fixture matches executable Scheme events"
      (let* ((options (JSONReadOptions object-as-hash: #t))
             (generated (string->json (outline-event-fixture-json) options))
             (saved (call-with-input-file
                     "languages/org/v1/generated/outline-event-fixture.json"
                     (lambda (port)
                       (string->json (read-all-as-string port) options)))))
        (check (hash-get saved "source") => (hash-get generated "source"))
        (check (hash-get saved "events") => (hash-get generated "events"))))))
