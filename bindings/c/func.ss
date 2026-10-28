;;; The native boundary composes Parser, Element and Contract operations.
;;; Foreign definitions stay compiler-owned; func contains pure Scheme only.
(import "event-tape.ss" "orgize-actor.ss")
(export (import: "event-tape.ss") (import: "orgize-actor.ss"))
