;;; -*- Gerbil -*-
;;; Org Element query and graph values, shared by downstream features.

(import (only-in :clan/poo/object .o .ref)
        (only-in :clan/poo/mop validate)
        (only-in :yuanxu/object-family/interface defpoo-object-family)
        (only-in "value-types.ss"
                 +org-element-schema+ +org-element-query-kind+
                 +org-element-clause-kind+ +org-element-graph-kind+
                 +org-element-predicate-kind+
                 +org-element-named-query-kind+
                 +org-headline-properties-kind+
                 OrgElementQuery OrgElementQueryClause OrgElementGraphView
                 OrgNamedElementQuery OrgElementPredicate
                 OrgHeadlineProperties OrgSourceMatchStrategy
                 +org-source-match-strategy-kind+))
(export make-org-element-query make-org-element-property-clause
        make-org-element-query-groups make-org-element-predicate
        make-org-named-element-query
        org-named-element-query-id org-named-element-query-query
        make-org-element-relation-clause make-org-element-graph-view
        org-element-query-node-kind
        org-element-query-relation
        org-element-query-groups org-element-predicate-groups
        org-element-query-target org-element-clause-kind
        org-element-clause-name org-element-clause-value
        org-element-clause-match
        org-element-graph-records
        org-element-graph-id-of org-element-graph-parent-of
        org-element-graph-kind-of org-element-graph-field-of
        make-org-headline-properties org-headline-property-field
        make-org-source-match-strategy)

(def (make-org-source-match-strategy extra-word-characters)
  (validate OrgSourceMatchStrategy
          (.o kind: +org-source-match-strategy-kind+
              schema: +org-element-schema+
              scan: 'utf8-character-boundaries
              candidate: 'exact-target-prefix
              boundary-unicode-alphanumeric: #t
              boundary-extra: extra-word-characters
              winner: 'longest-then-first)))


(def (make-org-headline-properties source-title-value title-value
                                   todo-keyword-value todo-type-value
                                   priority-value tags-value)
  (validate OrgHeadlineProperties
          (.o kind: +org-headline-properties-kind+
              schema: +org-element-schema+
              source-title: source-title-value title: title-value
              todo-keyword: todo-keyword-value todo-type: todo-type-value
              priority: priority-value tags: tags-value)))

(def (org-headline-property-field value name)
  (let (slot (if (equal? name "raw-value") 'title
               (and (member name '("title" "source-title" "todo-keyword"
                                   "todo-type" "priority" "tags"))
                    (string->symbol name))))
    (if slot (values #t (.ref value slot)) (values #f #f))))

(def (make-org-element-query node-kind-value
                             (field-name-value #f) (field-value-value #f)
                             (relation-value 'any) (target-value #f)
                             (field-match-value 'exact))
  (unless (and (eq? (not field-name-value) (not field-value-value))
               (memq field-match-value '(exact contains local-path)))
    (error "invalid Org Element property arguments"))
  (make-org-element-query-groups
   node-kind-value
   (if field-name-value
     (list (list (make-org-element-property-clause
                  field-name-value field-value-value field-match-value)))
     (list '()))
   relation-value target-value))

(def (make-org-element-query-groups node-kind-value groups-value
                                    relation-value target-value)
  (validate OrgElementQuery
          (.o kind: +org-element-query-kind+
              schema: +org-element-schema+
              node-kind: node-kind-value
              groups: groups-value
              relation: relation-value
              target: target-value)))

(def (make-org-element-predicate groups-value)
  (validate OrgElementPredicate
          (.o kind: +org-element-predicate-kind+
              schema: +org-element-schema+
              groups: groups-value)))

(def (make-org-named-element-query id-value query-value)
  (validate OrgNamedElementQuery
          (.o kind: +org-element-named-query-kind+
              schema: +org-element-schema+
              id: id-value query: query-value)))


(def (make-org-element-property-clause name-value field-value
                                       (field-match-value 'exact))
  (validate OrgElementQueryClause
          (.o kind: +org-element-clause-kind+
              schema: +org-element-schema+
              clause-kind: 'property
              name: name-value
              value: field-value
              match: field-match-value)))

(def (make-org-element-relation-clause relation-value target-value)
  (validate OrgElementQueryClause
          (.o kind: +org-element-clause-kind+
              schema: +org-element-schema+
              clause-kind: 'relation
              name: relation-value
              value: target-value
              match: #f)))

(def (make-org-element-graph-view records-value id-of-value parent-of-value
                                  kind-of-value field-of-value)
  ;; This structurally admits a caller-supplied view. It does not attest that
  ;; the records came from any declared Org source bytes or WorkTree cut.
  (validate OrgElementGraphView
          (.o kind: +org-element-graph-kind+
              schema: +org-element-schema+
              records: records-value
              id-of: id-of-value
              parent-of: parent-of-value
              kind-of: kind-of-value
              field-of: field-of-value)))


(defpoo-object-family
  (accessors
   (org-named-element-query-id id)
   (org-named-element-query-query query)
   (org-element-query-node-kind node-kind)
   (org-element-query-groups groups)
   (org-element-predicate-groups groups)
   (org-element-query-relation relation)
   (org-element-query-target target)
   (org-element-clause-kind clause-kind)
   (org-element-clause-name name)
   (org-element-clause-value value)
   (org-element-clause-match match)
   (org-element-graph-records records)
   (org-element-graph-id-of id-of)
   (org-element-graph-parent-of parent-of)
   (org-element-graph-kind-of kind-of)
   (org-element-graph-field-of field-of)
)
  (projections))
