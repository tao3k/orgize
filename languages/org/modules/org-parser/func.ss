;;; The Module's language declarations and strategy are native role dependencies.
(import "grammar.ss" "parser.ss" "event-strategy.ss")
(export (import: "grammar.ss") (import: "parser.ss")
        (import: "event-strategy.ss"))
