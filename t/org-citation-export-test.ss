;;; -*- Gerbil -*-
(import (only-in :std/test check test-case test-suite)
        (only-in "../languages/org/modules/org-parser/value-funs.ss" org-value-plan))
(export org-citation-export-test)
(def (plan key raw) (org-value-plan (list "citation-export-keywords" key raw)))
(def org-citation-export-test
  (test-suite "Native citation export policy"
    (test-case "Bibliography preserves whitespace-first quoted token semantics"
      (check (plan "BiBlIoGrApHy" "　\"refs.bib\" \"\" \"two words\"") =>
             '(("0" "bibliography" "refs.bib" "two" "words")))
      (check (plan "BIBLIOGRAPHY" "") => '(("0" "bibliography"))))
    (test-case "Processor distinguishes missing and present styles"
      (check (plan "cite_export" "") => '(("0" "processor" "" "none")))
      (check (plan "CITE_EXPORT" "csl") => '(("0" "processor" "csl" "none")))
      (check (plan "CITE_EXPORT" "csl　apa en") => '(("0" "processor" "csl" "some" "apa en"))))
    (test-case "Print options preserve duplicates, empty keys and optional values"
      (check (plan "print_bibliography" "ignored :::key one :flag :key two :") =>
             '(("0" "print") ("0" "option" "key" ":::key" "some" "one")
               ("0" "option" "flag" ":flag" "none")
               ("0" "option" "key" ":key" "some" "two")
               ("0" "option" "" ":" "none")))
      (check (plan "PRINT_BIBLIOGRAPHY" "") => '(("0" "print"))))
    (test-case "Batch keyword indexes retain source order and ignored entries"
      (check (org-value-plan '("citation-export-keywords" "TITLE" "ignored"
                "BIBLIOGRAPHY" "a a" "PRINT_BIBLIOGRAPHY" ":x y" "CITE_EXPORT" "nil")) =>
             '(("1" "bibliography" "a" "a") ("2" "print")
               ("2" "option" "x" ":x" "some" "y") ("3" "processor" "nil" "none")))
      (check (org-value-plan '("citation-export-keywords")) => '()))
    (test-case "Nocite policy is ASCII insensitive, never trims or evaluates"
      (check (org-value-plan '("citation-nocite" "NoCiTe" " nocite" "nocite " "nil" "")) =>
             '(("true") ("false") ("false") ("false") ("false")))
      (check (org-value-plan '("citation-nocite")) => '()))
    (test-case "Citation values remain inert and quote handling stays bounded"
      (check (plan "BIBLIOGRAPHY" "\"(error\" \"x)\" 'single'") =>
             '(("0" "bibliography" "(error" "x)" "'single'")))
      (check (plan "PRINT_BIBLIOGRAPHY" ":x \"\" :y \"value\"") =>
             '(("0" "print") ("0" "option" "x" ":x" "none")
               ("0" "option" "y" ":y" "some" "value"))))))
