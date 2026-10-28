;;; Org fields use native Records; only catalog and predicate bounds are Org policy.
(import (only-in :clan/poo/object .ref)
        (only-in :clan/poo/mop define-type Type. element? Any Bool Fun)
        (only-in :clan/poo/type Record String List Exactly Enum Or OrFalse)
        (only-in :clan/poo/number UInt)
        (only-in :std/list/list every)
        (only-in :gerbil-parser/graph-query-support GraphQueryViewContract graph-query-context?)
        (only-in :gerbil-parser/src/graph/graph-projection-objects
                 graph-node-label graph-node-fields graph-field-name)
        (only-in "graph-shape.ss" org-graph-shape org-headline-extra-fields))
(export OrgElementRow OrgElementId OrgElementKind
        +org-element-schema+ +org-element-query-kind+
        +org-element-clause-kind+ +org-element-graph-kind+
        +org-element-context-kind+ +org-element-profile-kind+
        +org-element-named-query-kind+
        +org-element-predicate-kind+
        +org-headline-properties-kind+
        +org-source-match-strategy-kind+
        OrgElementQuery OrgElementQueryClause
        OrgElementPredicate org-element-predicate?
        OrgNamedElementQuery org-named-element-query?
        OrgElementGraphView OrgElementQueryContext OrgElementsProfile
        OrgHeadlineProperties org-headline-properties?
        OrgSourceMatchStrategy org-source-match-strategy?
        org-element-query? org-element-query-clause?
        org-element-graph-view?
        org-element-query-context? org-elements-profile?)


(def OrgElementRow (Record))
(def OrgElementId UInt)
(def OrgElementKind String)
(def +org-element-schema+ "orgize.org-elements.v1")
(def +org-element-query-kind+ 'org-element-query)
(def +org-element-clause-kind+ 'org-element-query-clause)
(def +org-element-graph-kind+ 'org-element-graph-view)
(def +org-element-context-kind+ 'org-element-query-context)
(def +org-element-profile-kind+ 'org-elements-profile)
(def +org-element-named-query-kind+ 'org-named-element-query)
(def +org-element-predicate-kind+ 'org-element-predicate)
(def +org-headline-properties-kind+ 'org-headline-properties)
(def +org-source-match-strategy-kind+ 'org-source-match-strategy)


(define-type (OrgName @ Type.)
  .element?: (lambda (v) (and (string? v) (> (string-length v) 0))))
(define-type OrgTarget (Or (Exactly 'scope) OrgName))
(define-type PropertyClause
  (Record kind: [(Exactly +org-element-clause-kind+)]
          schema: [(Exactly +org-element-schema+)]
          clause-kind: [(Exactly 'property)] name: [OrgName] value: [String]
          match: [(Enum exact contains local-path)]))
(define-type RelationClause
  (Record kind: [(Exactly +org-element-clause-kind+)]
          schema: [(Exactly +org-element-schema+)]
          clause-kind: [(Exactly 'relation)]
          name: [(Enum at child-of descendant-of)] value: [OrgTarget]
          match: [(Exactly #f)]))
(define-type OrgElementQueryClause (Or PropertyClause RelationClause))
(def PropertyGroups (List (List PropertyClause)))
(define-type (OrgPredicateGroups @ PropertyGroups)
  .element?: (lambda (v)
               (and (element? PropertyGroups v) (pair? v) (<= (length v) 32)
                    (every (lambda (group) (<= (length group) 16)) v))))
(def (org-element-kind-rule label)
  (let loop ((rules org-graph-shape))
    (cond
     ((null? rules) #f)
     ((equal? label (graph-node-label (car rules))) (car rules))
     (else (loop (cdr rules))))))

(def (org-element-field? rule name)
  (and rule
       (or (and (member (graph-node-label rule)
                        '("headline" "inlinetask"))
                (member name org-headline-extra-fields))
           (let loop ((fields (graph-node-fields rule)))
             (cond
              ((null? fields) #f)
              ((equal? name (graph-field-name (car fields))) #t)
              (else (loop (cdr fields))))))))


(def QueryFields
  (Record kind: [(Exactly +org-element-query-kind+)]
          schema: [(Exactly +org-element-schema+)]
          node-kind: [String] groups: [OrgPredicateGroups]
          relation: [(Enum any at child-of descendant-of)] target: [(OrFalse OrgTarget)]))
(define-type (OrgElementQuery @ QueryFields)
  .element?:
  (lambda (v)
    (and (element? QueryFields v)
         (let (rule (org-element-kind-rule (.ref v 'node-kind)))
           (and rule
                (every (lambda (group)
                         (every (lambda (clause) (org-element-field? rule (.ref clause 'name))) group))
                       (.ref v 'groups))))
         (if (eq? (.ref v 'relation) 'any)
           (not (.ref v 'target)) (and (.ref v 'target) #t)))))
(define-type OrgElementPredicate
  (Record kind: [(Exactly +org-element-predicate-kind+)]
          schema: [(Exactly +org-element-schema+)] groups: [OrgPredicateGroups]))
(define-type OrgElementGraphView
  (Record [(.ref (GraphQueryViewContract OrgElementRow OrgElementId OrgElementKind) 'value-contract)]
          kind: [(Exactly +org-element-graph-kind+)] schema: [(Exactly +org-element-schema+)]
          field-of: [(Fun Any <- OrgElementRow String)]))
(define-type (OrgGraphIndex @ Type.) .element?: graph-query-context?)
(define-type OrgElementQueryContext
  (Record kind: [(Exactly +org-element-context-kind+)]
          schema: [(Exactly +org-element-schema+)]
          graph: [OrgElementGraphView] index: [OrgGraphIndex]))
(define-type OrgElementsProfile
  (Record kind: [(Exactly +org-element-profile-kind+)]
          schema: [(Exactly +org-element-schema+)] inventory: [(List Any)] projection: [Any]))
(define-type OrgNamedElementQuery
  (Record kind: [(Exactly +org-element-named-query-kind+)]
          schema: [(Exactly +org-element-schema+)] id: [OrgName] query: [OrgElementQuery]))
(define-type OrgSourceMatchStrategy
  (Record kind: [(Exactly +org-source-match-strategy-kind+)]
          schema: [(Exactly +org-element-schema+)] scan: [(Exactly 'utf8-character-boundaries)]
          candidate: [(Exactly 'exact-target-prefix)] boundary-unicode-alphanumeric: [Bool]
          boundary-extra: [String] winner: [(Exactly 'longest-then-first)]))
(define-type OrgHeadlineProperties
  (Record kind: [(Exactly +org-headline-properties-kind+)]
          schema: [(Exactly +org-element-schema+)] source-title: [String] title: [String]
          todo-keyword: [(OrFalse String)] todo-type: [(OrFalse String)]
          priority: [(OrFalse String)] tags: [(List String)]))

(def (org-element-query? v) (element? OrgElementQuery v))
(def (org-element-query-clause? v) (element? OrgElementQueryClause v))
(def (org-element-predicate? v) (element? OrgElementPredicate v))
(def (org-named-element-query? v) (element? OrgNamedElementQuery v))
(def (org-element-graph-view? v) (element? OrgElementGraphView v))
(def (org-element-query-context? v) (element? OrgElementQueryContext v))
(def (org-elements-profile? v) (element? OrgElementsProfile v))
(def (org-source-match-strategy? v) (element? OrgSourceMatchStrategy v))
(def (org-headline-properties? v) (element? OrgHeadlineProperties v))
