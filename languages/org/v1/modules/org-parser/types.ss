;;; -*- Gerbil -*-
;;; Org-owned, admitted strategy declarations; event execution stays pure/AOT.

(import (only-in :clan/poo/object .ref .slot? object?)
        (only-in :clan/poo/mop define-type Type. element?)
        (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 block-line?))
(export +org-event-block-kind+ +org-inline-markup-kind+
        OrgEventBlock OrgInlineMarkup
        org-event-block? org-inline-markup?)

(def +org-event-block-kind+ 'org-event-block)
(def +org-inline-markup-kind+ 'org-inline-markup)

(def (event-block-shape? value)
  (and (object? value)
       (.slot? value 'kind) (.slot? value 'id) (.slot? value 'rule)
       (eq? (.ref value 'kind) +org-event-block-kind+)
       (let (id (.ref value 'id))
         (and (fixnum? id) (> id 0)))
       (block-line? (.ref value 'rule))))

(define-type (OrgEventBlock @ Type.)
  .element?: event-block-shape?)

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

(def (org-event-block? value) (element? OrgEventBlock value))
(def (org-inline-markup? value) (element? OrgInlineMarkup value))
