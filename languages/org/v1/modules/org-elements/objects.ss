;;; -*- Gerbil -*-
;;; Org Element query and graph values, shared by downstream features.

(import (only-in :clan/poo/object .o .ref)
        (only-in :clan/poo/mop element?)
        (only-in "types.ss"
                 +org-element-schema+ +org-element-query-kind+
                 +org-element-clause-kind+ +org-element-graph-kind+
                 +org-element-predicate-kind+
                 +org-element-named-query-kind+
                 +org-headline-properties-kind+
                 OrgElementQuery OrgElementQueryClause OrgElementGraphView
                 OrgNamedElementQuery OrgElementPredicate
                 OrgHeadlineProperties))
(export make-org-element-query make-org-element-property-clause
        make-org-element-query-groups make-org-element-predicate
        make-org-named-element-query
        org-named-element-query-id org-named-element-query-query
        make-org-element-relation-clause make-org-element-graph-view
        org-element-query-node-kind org-element-query-field-name
        org-element-query-field-value org-element-query-field-match
        org-element-query-relation
        org-element-query-groups org-element-predicate-groups
        org-element-query-target org-element-clause-kind
        org-element-clause-name org-element-clause-value
        org-element-clause-match
        org-element-graph-records
        org-element-graph-id-of org-element-graph-parent-of
        org-element-graph-kind-of org-element-graph-field-of
        make-org-headline-properties org-headline-property-field)

(def (admit! type value)
  (unless (element? type value)
    (error "invalid Org Element POO value" value))
  value)

(def (make-org-headline-properties source-title-value title-value
                                   todo-keyword-value todo-type-value
                                   priority-value tags-value)
  (admit! OrgHeadlineProperties
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
               (memq field-match-value '(exact contains)))
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
  (admit! OrgElementQuery
          (.o kind: +org-element-query-kind+
              schema: +org-element-schema+
              node-kind: node-kind-value
              groups: groups-value
              relation: relation-value
              target: target-value)))

(def (make-org-element-predicate groups-value)
  (admit! OrgElementPredicate
          (.o kind: +org-element-predicate-kind+
              schema: +org-element-schema+
              groups: groups-value)))

(def (make-org-named-element-query id-value query-value)
  (admit! OrgNamedElementQuery
          (.o kind: +org-element-named-query-kind+
              schema: +org-element-schema+
              id: id-value query: query-value)))

(def (org-named-element-query-id value) (.ref value 'id))
(def (org-named-element-query-query value) (.ref value 'query))

(def (make-org-element-property-clause name-value field-value
                                       (field-match-value 'exact))
  (admit! OrgElementQueryClause
          (.o kind: +org-element-clause-kind+
              schema: +org-element-schema+
              clause-kind: 'property
              name: name-value
              value: field-value
              match: field-match-value)))

(def (make-org-element-relation-clause relation-value target-value)
  (admit! OrgElementQueryClause
          (.o kind: +org-element-clause-kind+
              schema: +org-element-schema+
              clause-kind: 'relation
              name: relation-value
              value: target-value
              match: #f)))

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
(def (org-element-query-groups value) (.ref value 'groups))
(def (org-element-predicate-groups value) (.ref value 'groups))
(def (org-element-query-first-property value)
  (let (groups (org-element-query-groups value))
    (and (= (length groups) 1) (= (length (car groups)) 1)
         (caar groups))))
(def (org-element-query-field-name value)
  (let (property (org-element-query-first-property value))
    (and property (org-element-clause-name property))))
(def (org-element-query-field-value value)
  (let (property (org-element-query-first-property value))
    (and property (org-element-clause-value property))))
(def (org-element-query-field-match value)
  (let (property (org-element-query-first-property value))
    (if property (org-element-clause-match property) 'exact)))
(def (org-element-query-relation value) (.ref value 'relation))
(def (org-element-query-target value) (.ref value 'target))
(def (org-element-clause-kind value) (.ref value 'clause-kind))
(def (org-element-clause-name value) (.ref value 'name))
(def (org-element-clause-value value) (.ref value 'value))
(def (org-element-clause-match value) (.ref value 'match))
(def (org-element-graph-records value) (.ref value 'records))
(def (org-element-graph-id-of value) (.ref value 'id-of))
(def (org-element-graph-parent-of value) (.ref value 'parent-of))
(def (org-element-graph-kind-of value) (.ref value 'kind-of))
(def (org-element-graph-field-of value) (.ref value 'field-of))
