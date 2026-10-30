;;; Native Records admit Org strategy fields; Parser owns grammar predicates.
(import (only-in :clan/poo/object .slot?)
        (only-in :clan/poo/mop define-type Type. element?)
        (only-in :clan/poo/type Record Exactly Symbol String List Enum)
        (only-in :gerbil-parser/language-support/grammar block-line?))
(export +org-event-block-kind+ +org-named-block-kind+ +org-inline-markup-kind+
        +org-inline-script-kind+
        +org-event-helper-kind+
        +org-event-strategy-kind+
        OrgEventBlock OrgNamedBlock OrgInlineMarkup OrgInlineScript
        OrgEventHelper OrgEventStrategy
        org-event-block? org-named-block? org-inline-markup?
        org-inline-script? org-event-helper? org-event-strategy?)

(def +org-event-block-kind+ 'org-event-block)
(def +org-named-block-kind+ 'org-named-block)
(def +org-inline-markup-kind+ 'org-inline-markup)
(def +org-inline-script-kind+ 'org-inline-script)
(def +org-event-helper-kind+ 'org-event-helper)
(def +org-event-strategy-kind+ 'org-event-strategy)


(define-type (PositiveFixnum @ Type.)
  .element?: (lambda (v) (and (fixnum? v) (> v 0))))
(define-type (AsciiByte @ Type.)
  .element?: (lambda (v) (and (fixnum? v) (<= 0 v 127))))
(define-type (AsciiDelimiter @ String)
  .element?: (lambda (v) (and (string? v) (> (string-length v) 0)
                              (every (lambda (c) (< (char->integer c) 128)) (string->list v)))))
(define-type (BlockLine @ Type.) .element?: block-line?)
(define-type (DeclarationList @ Type.) .element?: list?)
(define-type (DeclarationPairs @ Type.) .element?: pair?)
(define-type OrgEventBlock
  (Record kind: [(Exactly +org-event-block-kind+)] id: [PositiveFixnum] rule: [BlockLine]))
(define-type OrgNamedBlock
  (Record kind: [(Exactly +org-named-block-kind+)] opening: [AsciiDelimiter]
          closing: [AsciiDelimiter] node: [Symbol] name-token: [Symbol]))
(define-type OrgInlineMarkup
  (Record kind: [(Exactly +org-inline-markup-kind+)] byte: [AsciiByte]
          id: [PositiveFixnum] node: [Symbol]))
(define-type OrgInlineScript
  (Record kind: [(Exactly +org-inline-script-kind+)] byte: [(Enum 94 95)]
          id: [PositiveFixnum] node: [Symbol]))
(define-type OrgEventHelper
  (Record kind: [(Exactly +org-event-helper-kind+)] name: [Symbol]
          initial: [DeclarationList] forms: [DeclarationList] parameters: [(List Symbol)]))
(define-type StrategyFields
  (Record kind: [(Exactly +org-event-strategy-kind+)] root: [Symbol]
          initial: [DeclarationList] line-forms: [DeclarationPairs] finish-forms: [DeclarationList]
          helpers: [(List OrgEventHelper)] parameters: [DeclarationList]))
;; These two products are lazy owned computations. Membership checks presence
;; without evaluating an IR compiler or forcing the derived descriptor cache.
(define-type (OrgEventStrategy @ StrategyFields)
  .element?: (lambda (v) (and (element? StrategyFields v)
                              (.slot? v 'helper-descriptors) (.slot? v 'program))))

(def (org-event-block? v) (element? OrgEventBlock v))
(def (org-named-block? v) (element? OrgNamedBlock v))
(def (org-inline-markup? v) (element? OrgInlineMarkup v))
(def (org-inline-script? v) (element? OrgInlineScript v))
(def (org-event-helper? v) (element? OrgEventHelper v))
(def (org-event-strategy? v) (element? OrgEventStrategy v))
