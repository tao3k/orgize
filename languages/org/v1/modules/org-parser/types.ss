;;; -*- Gerbil -*-
;;; Org-owned, admitted strategy declarations; event execution stays pure/AOT.

(import (only-in :clan/poo/object .ref .slot? object?)
        (only-in :clan/poo/mop define-type Type. element?)
        (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 block-line?))
(export +org-event-block-kind+ +org-named-block-kind+ +org-inline-markup-kind+
        +org-inline-script-kind+
        +org-event-strategy-kind+
        OrgEventBlock OrgNamedBlock OrgInlineMarkup OrgInlineScript OrgEventStrategy
        org-event-block? org-named-block? org-inline-markup?
        org-inline-script? org-event-strategy?)

(def +org-event-block-kind+ 'org-event-block)
(def +org-named-block-kind+ 'org-named-block)
(def +org-inline-markup-kind+ 'org-inline-markup)
(def +org-inline-script-kind+ 'org-inline-script)
(def +org-event-strategy-kind+ 'org-event-strategy)

(def (event-block-shape? value)
  (and (object? value)
       (.slot? value 'kind) (.slot? value 'id) (.slot? value 'rule)
       (eq? (.ref value 'kind) +org-event-block-kind+)
       (let (id (.ref value 'id))
         (and (fixnum? id) (> id 0)))
       (block-line? (.ref value 'rule))))

(define-type (OrgEventBlock @ Type.)
  .element?: event-block-shape?)

(def (named-block-shape? value)
  (and (object? value)
       (.slot? value 'kind) (.slot? value 'opening)
       (.slot? value 'closing) (.slot? value 'node)
       (.slot? value 'name-token)
       (eq? (.ref value 'kind) +org-named-block-kind+)
       (let ((opening (.ref value 'opening))
             (closing (.ref value 'closing)))
         (and (string? opening) (string? closing)
              (> (string-length opening) 0)
              (> (string-length closing) 0)
              (every (lambda (character) (< (char->integer character) 128))
                     (string->list opening))
              (every (lambda (character) (< (char->integer character) 128))
                     (string->list closing))))
       (symbol? (.ref value 'node))
       (symbol? (.ref value 'name-token))))

(define-type (OrgNamedBlock @ Type.)
  .element?: named-block-shape?)

(def (inline-markup-shape? value)
  (and (object? value)
       (.slot? value 'kind) (.slot? value 'byte)
       (.slot? value 'id) (.slot? value 'node)
       (eq? (.ref value 'kind) +org-inline-markup-kind+)
       (let ((byte (.ref value 'byte)) (id (.ref value 'id)))
         (and (fixnum? byte) (<= 0 byte 127)
              (fixnum? id) (> id 0)))
       (symbol? (.ref value 'node))))

(define-type (OrgInlineMarkup @ Type.)
  .element?: inline-markup-shape?)

(def (inline-script-shape? value)
  (and (object? value)
       (.slot? value 'kind) (.slot? value 'byte)
       (.slot? value 'id) (.slot? value 'node)
       (eq? (.ref value 'kind) +org-inline-script-kind+)
       (let ((byte (.ref value 'byte)) (id (.ref value 'id)))
         (and (fixnum? byte) (memv byte '(94 95))
              (fixnum? id) (> id 0)))
       (symbol? (.ref value 'node))))

(define-type (OrgInlineScript @ Type.)
  .element?: inline-script-shape?)

(def (event-strategy-shape? value)
  (and (object? value)
       (.slot? value 'kind) (.slot? value 'root)
       (.slot? value 'initial) (.slot? value 'line-forms)
       (.slot? value 'finish-forms) (.slot? value 'helpers)
       (eq? (.ref value 'kind) +org-event-strategy-kind+)
       (symbol? (.ref value 'root))
       (list? (.ref value 'initial))
       (pair? (.ref value 'line-forms))
       (list? (.ref value 'finish-forms))
       (list? (.ref value 'helpers))))

(define-type (OrgEventStrategy @ Type.)
  .element?: event-strategy-shape?)

(def (org-event-block? value) (element? OrgEventBlock value))
(def (org-named-block? value) (element? OrgNamedBlock value))
(def (org-inline-markup? value) (element? OrgInlineMarkup value))
(def (org-inline-script? value) (element? OrgInlineScript value))
(def (org-event-strategy? value) (element? OrgEventStrategy value))
