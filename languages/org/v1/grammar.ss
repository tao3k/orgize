;;; -*- Gerbil -*-
;;; Org v1 grammar. Contextual structure is declared in parser.ss.

(import (only-in :gerbil-parser/language-support deflanguage-grammar))
(export org-v1-language-grammar)

(deflanguage-grammar org-v1
  (identity "org" "v1" "org-elements.v1")
  (syntax-kinds
   (OrgFile node (element))
   (OrgHeadline node (line))
   (OrgSourceBlock node (begin body end))
   (OrgTextLine node (line))
   (OrgSection node (heading element))
   (HeadlineLine token (text))
   (BlockBeginLine token (text))
   (BlockEndLine token (text))
   (TextLine token (text)))
  (terminals
   (headline HeadlineLine)
   (block-begin BlockBeginLine)
   (block-end BlockEndLine)
   (text TextLine))
  (lexical-rules
   (headline (literals "*"))
   (block-begin (literals "#+begin_src"))
   (block-end (literals "#+end_src"))
   (text (fallback)))
  (rules
   (org-file
    (alias OrgFile
      (repeat (field element (reference org-element)))))
   (org-element
    (choice (reference source-block)
            (reference headline)
            (reference text-line)))
   (source-block
    (alias OrgSourceBlock
      (seq (field begin (token block-begin))
           (repeat (field body (token text)))
           (field end (token block-end)))))
   (headline
    (alias OrgHeadline (field line (token headline))))
   (text-line
    (alias OrgTextLine (field line (token text)))))
  (extras)
  (keywords)
  (parser-entrypoints (org-file parse pure))
  (recoveries)
  (conflicts reject)
  (case-insensitive #f)
  (flow (source lexical) (lexical cst)))
