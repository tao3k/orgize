;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; POO contracts for Org-owned structural edits and canonical section specs.

(import (only-in :clan/poo/object .o .ref .cc)
        (only-in :clan/poo/type define-type Type. element? Any Bool Fun
                 Record String List Enum Exactly OrFalse)
        (only-in :clan/poo/number UInt)
        (only-in :tao3k/yuanxu/types/contracts YuanxuContract.))
(export +org-edit-schema+ +org-edit-op-kind+ +org-section-spec-kind+
        OrgEditOpKind OrgEditOp OrgEditOpContract
        OrgSectionSpec OrgSectionSpecContract
        OrgEditScopeContract
        org-edit-op? org-section-spec?)

(def +org-edit-schema+ "orgize.org-edit.v1")
(def +org-edit-op-kind+ 'org-edit-op)
(def +org-section-spec-kind+ 'org-section-spec)

(define-type OrgEditOpKind
  (Enum set-todo add-tag remove-tag
        set-planning remove-planning
        upsert-properties delete-properties
        set-checkbox upsert-babel-results
        upsert-section append-section))

(define-type OrgEditOp
  (Record kind: [(Exactly +org-edit-op-kind+)]
          schema: [(Exactly +org-edit-schema+)]
          op: [OrgEditOpKind]
          target: [Any]
          payload: [Any]))

(def OrgEditOpContract
  (.cc YuanxuContract.
       identity: 'org-edit/op
       proto: (.o)
       value-contract: OrgEditOp))

(define-type OrgSectionSpec
  (Record kind: [(Exactly +org-section-spec-kind+)]
          schema: [(Exactly +org-edit-schema+)]
          level: [UInt]
          title: [String]
          todo: [(OrFalse String)]
          priority: [(OrFalse String)]
          cookie: [(OrFalse String)]
          tags: [(List String)]
          planning: [(List Any)]
          properties: [(List Any)]
          body: [(OrFalse String)]))

(def OrgSectionSpecContract
  (.cc YuanxuContract.
       identity: 'org-edit/section-spec
       proto: (.o)
       value-contract: OrgSectionSpec))

(define-type OrgEditScope
  (Record apply-edit: [(Fun String <- String OrgEditOp)]
          apply-edits: [(Fun String <- String (List OrgEditOp))]))

(def OrgEditScopeContract
  (.cc OrgEditScope identity: 'org-edit/scope))

(def (org-edit-op? v) (element? OrgEditOp v))
(def (org-section-spec? v) (element? OrgSectionSpec v))
