;;; -*- Gerbil -*-
;;; Encode Parser's projected navigation and Element data for the Rust consumer.
(import (for-syntax
         (only-in :gerbil-parser/graph-projection-support
                  graph-projection-digest graph-projection-nodes graph-node-syntax-kind
                  graph-node-fields graph-field-name)
         (only-in "../org-parser/grammar.ss" org-mode-language-grammar)
         (only-in "graph.ss" org-graph-projection))
        (only-in "../org-parser/family-funs.ss" org-ledger-headline)
        (only-in "headline-properties.ss" todo-directive?))
(export org-fold-product)

(defsyntax (define-product-identities stx)
  (syntax-case stx ()
    ((_ digest keywords headings)
     (let (nodes (graph-projection-nodes org-graph-projection))
       (def (fields kind names)
         (let loop ((remaining nodes) (ordinal 0))
           (let (node (car remaining))
             (if (eq? (graph-node-syntax-kind node) kind)
               (let (fields (graph-node-fields node))
                 (cons ordinal
                       (map (lambda (name)
                              (filter-map (lambda (field id)
                                            (and (string=? name (graph-field-name field)) id))
                                          fields (iota (length fields))))
                            names)))
               (loop (cdr remaining) (+ ordinal 1))))))
       (with-syntax ((identity (datum->syntax #'digest
                                (string->utf8 (graph-projection-digest org-mode-language-grammar
                                                                       org-graph-projection))))
                     (keyword-fields (datum->syntax #'keywords
                                     (fields 'OrgKeyword '("key" "raw-value"))))
                     (heading-fields (datum->syntax #'headings
                                     (fields 'OrgSection '("markers" "title" "tag")))))
         #'(begin (def digest identity)
                  (def keywords 'keyword-fields)
                  (def headings 'heading-fields)))))))

(define-product-identities projection-identity ledger-keyword-fields ledger-heading-fields)

(def (record-field row ordinals)
  (find (lambda (field) (memv (car field) ordinals)) (vector-ref row 5)))

(def (field-text source field)
  (if (not field) ""
    (let* ((spans (cdr field))
           (first (car spans))
           (until
            (let contiguous ((rest (cdr spans)) (end (cdr first)))
              (cond ((null? rest) end)
                    ((= (caar rest) end) (contiguous (cdr rest) (cdar rest)))
                    (else #f)))))
      (if until
        (utf8->string source (car first) until)
        (apply string-append
               (map (lambda (span) (utf8->string source (car span) (cdr span))) spans))))))

(def (ledger-headline-elements source elements todo-profile)
  (let loop ((index 1) (directives '()) (title #f) (tag? #f))
    (if (= index (vector-length elements))
      (if title
        (map string->utf8 (org-ledger-headline todo-profile (reverse! directives) title tag?))
        '())
      (let* ((row (vector-ref elements index)) (kind (vector-ref row 0)))
        (cond
         ((= kind (car ledger-keyword-fields))
          (let (key (field-text source (record-field row (cadr ledger-keyword-fields))))
            (loop (+ index 1)
                  (if (todo-directive? key)
                    (cons (field-text source (record-field row (caddr ledger-keyword-fields)))
                          directives)
                    directives)
                  title tag?)))
         ((and (= kind (car ledger-heading-fields)) (eqv? (vector-ref row 1) 0)
               (= (vector-ref row 2) 0)
               (string=? "*" (field-text source (record-field row (cadr ledger-heading-fields)))))
          (let (field (record-field row (caddr ledger-heading-fields)))
            (loop (+ index 1) directives (and field (field-text source field))
                  (record-field row (cadddr ledger-heading-fields)))))
         (else (loop (+ index 1) directives title tag?)))))))

(def (put-word! bytes offset value)
  (##u8vector-set! bytes offset (fxand value 255))
  (##u8vector-set! bytes (fx+ offset 1) (fxand (fxarithmetic-shift-right value 8) 255))
  (##u8vector-set! bytes (fx+ offset 2) (fxand (fxarithmetic-shift-right value 16) 255))
  (##u8vector-set! bytes (fx+ offset 3) (fxarithmetic-shift-right value 24)))

(def (put-pair! bytes offset first second)
  (put-word! bytes offset first)
  (put-word! bytes (fx+ offset 4) second))

(def (put-syntax-row! bytes offset kind parent start end subtree-end)
  (put-pair! bytes offset kind parent)
  (put-pair! bytes (fx+ offset 8) start end)
  (put-word! bytes (fx+ offset 16) subtree-end))

(def absent #xffffffff)
(def syntax-row-words 5)
(def projection-offset 75)
(def document-header-width (+ projection-offset 71 8))

(def (org-fold-product product (source #f) (todo-profile #f))
  (def navigation (vector-ref product 1))
  (def elements (vector-ref product 2))
  (def syntax-words (u32vector-length navigation))
  (def element-count (vector-length elements))
  (def elements-offset (+ document-header-width (* 4 syntax-words)))
  (def columns-end
    (let loop ((index 0) (size elements-offset))
      (if (= index element-count) size
        (loop (+ index 1)
              (foldl (lambda (field bytes) (+ bytes 8 (* 8 (length (cdr field)))))
                     (+ size 24) (vector-ref (vector-ref elements index) 5))))))
  (def metadata (and todo-profile (ledger-headline-elements source elements todo-profile)))
  (def size
    (if metadata
      (foldl (lambda (value size) (+ size 4 (u8vector-length value))) (+ columns-end 4) metadata)
      columns-end))
  (def bytes (make-u8vector size 0))
  (u8vector-copy! bytes 0 #u8(79 68 86 49)) ; ODV1
  (u8vector-copy! bytes 4 (string->utf8 (vector-ref product 0)))
  (u8vector-copy! bytes projection-offset projection-identity)
  (put-pair! bytes (- document-header-width 8)
             (quotient syntax-words syntax-row-words) element-count)
  (do ((index 0 (fx+ index 1)) (offset document-header-width (fx+ offset 4)))
      ((fx= index syntax-words))
    (put-word! bytes offset (##u32vector-ref navigation index)))
  (let write ((index 0) (offset elements-offset))
    (unless (= index element-count)
      (let* ((element (vector-ref elements index))
             (fields (vector-ref element 5))
             (next
              (let fields-loop ((remaining fields) (offset (+ offset 24)))
                (if (null? remaining) offset
                  (let* ((field (car remaining)) (spans (cdr field)))
                    (put-pair! bytes offset (car field) (length spans))
                    (fields-loop
                     (cdr remaining)
                     (let span-loop ((spans spans) (offset (+ offset 8)))
                       (if (null? spans) offset
                         (begin
                           (put-pair! bytes offset (caar spans) (cdar spans))
                           (span-loop (cdr spans) (+ offset 8)))))))))))
        (put-syntax-row! bytes offset (vector-ref element 0) (or (vector-ref element 1) absent)
                         (vector-ref element 2) (vector-ref element 3) (vector-ref element 4))
        (put-word! bytes (+ offset 20) (length fields))
        (write (+ index 1) next))))
  (when metadata
    (put-word! bytes columns-end (length metadata))
    (let write ((remaining metadata) (offset (+ columns-end 4)))
      (unless (null? remaining)
        (let* ((value (car remaining)) (size (u8vector-length value)) (start (+ offset 4)))
          (put-word! bytes offset size)
          (u8vector-copy! bytes start value)
          (write (cdr remaining) (+ start size))))))
  bytes)
