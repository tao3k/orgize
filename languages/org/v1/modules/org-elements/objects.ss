;;; -*- Gerbil -*-
;;; Org Element query and graph values, shared by downstream features.

(import (only-in :clan/poo/object .o .ref)
        (only-in :clan/poo/mop element?)
        (only-in "types.ss"
                 +org-element-schema+ +org-element-query-kind+
                 +org-element-clause-kind+ +org-element-graph-kind+
                 OrgElementQuery OrgElementQueryClause OrgElementGraphView))
(export make-org-element-query make-org-element-property-clause
        make-org-element-relation-clause make-org-element-graph-view
        org-element-query-node-kind org-element-query-field-name
        org-element-query-field-value org-element-query-relation
        org-element-query-target org-element-clause-kind
        org-element-clause-name org-element-clause-value
        org-element-graph-records
        org-element-graph-id-of org-element-graph-parent-of
        org-element-graph-kind-of org-element-graph-field-of)

(def (admit! type value)
  (unless (element? type value)
    (error "invalid Org Element POO value" value))
  value)

(def (make-org-element-query node-kind-value
                             (field-name-value #f) (field-value-value #f)
                             (relation-value 'any) (target-value #f))
  (admit! OrgElementQuery
          (.o kind: +org-element-query-kind+
              schema: +org-element-schema+
              node-kind: node-kind-value
              field-name: field-name-value
              field-value: field-value-value
              relation: relation-value
              target: target-value)))

(def (make-org-element-property-clause name-value field-value)
  (admit! OrgElementQueryClause
          (.o kind: +org-element-clause-kind+
              schema: +org-element-schema+
              clause-kind: 'property
              name: name-value
              value: field-value)))

(def (make-org-element-relation-clause relation-value target-value)
  (admit! OrgElementQueryClause
          (.o kind: +org-element-clause-kind+
              schema: +org-element-schema+
              clause-kind: 'relation
              name: relation-value
              value: target-value)))

(def (make-org-element-graph-view records-value id-of-value parent-of-value
                                  kind-of-value field-of-value)
  (admit! OrgElementGraphView
          (.o kind: +org-element-graph-kind+
              schema: +org-element-schema+
              records: records-value
              id-of: id-of-value
              parent-of: parent-of-value
              kind-of: kind-of-value
              field-of: field-of-value)))

(def (org-element-query-node-kind value) (.ref value 'node-kind))
(def (org-element-query-field-name value) (.ref value 'field-name))
(def (org-element-query-field-value value) (.ref value 'field-value))
(def (org-element-query-relation value) (.ref value 'relation))
(def (org-element-query-target value) (.ref value 'target))
(def (org-element-clause-kind value) (.ref value 'clause-kind))
(def (org-element-clause-name value) (.ref value 'name))
(def (org-element-clause-value value) (.ref value 'value))
(def (org-element-graph-records value) (.ref value 'records))
(def (org-element-graph-id-of value) (.ref value 'id-of))
(def (org-element-graph-parent-of value) (.ref value 'parent-of))
(def (org-element-graph-kind-of value) (.ref value 'kind-of))
(def (org-element-graph-field-of value) (.ref value 'field-of))
