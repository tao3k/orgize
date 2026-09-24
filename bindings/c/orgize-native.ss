;;; -*- Gerbil -*-
;;; C ABI owned by Orgize Scheme. C only transports graph facts and results.

(import (only-in :std/ffi C-declare C-ffi-macrology def-C-lambda def-C-type)
        (only-in :clan/poo/object .o .ref)
        (only-in "../../languages/org/v1/modules/org-elements/runtime-interface.ss"
                 make-org-element-graph-view make-org-element-query)
        (only-in "../../languages/org/v1/modules/org-contract/runtime-interface.ss"
                 make-org-contract-assertion make-org-contract-expectation
                 org-contract-evaluate-assertion
                 org-contract-result-matched-count
                 org-contract-result-passed?))
(export orgize-c-round-trip)

(C-ffi-macrology)

(C-declare #<<END-C
#include "include/orgize.h"

extern int32_t orgize_contract_evaluate_scheme(
  orgize_element_row *, uint32_t, int64_t, char *, char *, char *,
  uint32_t, uint32_t, orgize_contract_result *);

int32_t orgize_contract_evaluate(const orgize_element_row *rows,
                                 uint32_t row_count, int64_t scope_id,
                                 const char *query_kind,
                                 const char *field_name,
                                 const char *field_value,
                                 uint32_t expectation,
                                 uint32_t expected_count,
                                 orgize_contract_result *result) {
  return orgize_contract_evaluate_scheme(
    (orgize_element_row *)rows, row_count, scope_id,
    (char *)query_kind, (char *)field_name, (char *)field_value,
    expectation, expected_count, result);
}

static int32_t orgize_c_round_trip(void) {
  const orgize_element_row rows[] = {
    {0, -1, "org-data", "", ""},
    {1, 0, "headline", "title", "Evidence"}
  };
  orgize_contract_result result = {0, 0, 0};
  if (orgize_abi_revision() != 1u) return 1;
  if (orgize_contract_evaluate(rows, 2u, 0, "headline", "title",
                               "Evidence", 0u, 1u, &result) != 0) return 2;
  if (result.status != 0 || result.matched_count != 1u ||
      result.passed != 1) return 3;
  return 0;
}
END-C
)

(def-C-type orgize_element_row "orgize_element_row")
(def-C-type orgize_element_row-borrowed-ptr*
  (pointer orgize_element_row (orgize_element_row-borrowed-ptr*)))
(def-C-type orgize_contract_result "orgize_contract_result")
(def-C-type orgize_contract_result-borrowed-ptr*
  (pointer orgize_contract_result (orgize_contract_result-borrowed-ptr*)))

(def-C-lambda row-id (orgize_element_row-borrowed-ptr* unsigned-int32)
  int64 "___return (___arg1[___arg2].id);")
(def-C-lambda row-parent (orgize_element_row-borrowed-ptr* unsigned-int32)
  int64 "___return (___arg1[___arg2].parent_id);")
(def-C-lambda row-kind (orgize_element_row-borrowed-ptr* unsigned-int32)
  UTF-8-string "___return ((char*)___arg1[___arg2].kind);")
(def-C-lambda row-field-name (orgize_element_row-borrowed-ptr* unsigned-int32)
  UTF-8-string "___return ((char*)___arg1[___arg2].field_name);")
(def-C-lambda row-field-value (orgize_element_row-borrowed-ptr* unsigned-int32)
  UTF-8-string "___return ((char*)___arg1[___arg2].field_value);")
(def-C-lambda row-valid? (orgize_element_row-borrowed-ptr* unsigned-int32)
  bool "___return (___arg1[___arg2].kind != NULL &&
                    ___arg1[___arg2].field_name != NULL &&
                    ___arg1[___arg2].field_value != NULL &&
                    ___arg1[___arg2].id >= 0 &&
                    ___arg1[___arg2].parent_id >= -1);")
(def-C-lambda rows-valid? (orgize_element_row-borrowed-ptr* unsigned-int32)
  bool "___return (___arg2 == 0 || ___arg1 != NULL);")
(def-C-lambda result-valid? (orgize_contract_result-borrowed-ptr*)
  bool "___return (___arg1 != NULL);")
(def-C-lambda result-set!
  (orgize_contract_result-borrowed-ptr* int32 unsigned-int32 int32)
  void "___arg1->status = ___arg2;
        ___arg1->matched_count = ___arg3;
        ___arg1->passed = ___arg4;
        ___return;")
(def-C-lambda orgize-c-round-trip () int32 "orgize_c_round_trip")

(def (make-row rows index)
  (unless (row-valid? rows index)
    (error "invalid Org graph fact" index))
  (.o id: (row-id rows index)
      parent: (let (parent (row-parent rows index))
                (if (= parent -1) #f parent))
      kind: (row-kind rows index)
      field-name: (row-field-name rows index)
      field-value: (row-field-value rows index)))

(def (evaluate-rows rows count scope kind field value expectation expected result)
  (unless (and (result-valid? result) (rows-valid? rows count)
               (<= count 10000) (<= expected 10000)
               (>= scope 0) (not (equal? kind "")))
    (error "invalid Org contract ABI input"))
  (let* ((records (let loop ((index 0) (out '()))
                    (if (= index count) (reverse out)
                      (loop (+ index 1) (cons (make-row rows index) out)))))
         (graph
          (make-org-element-graph-view
           records
           (lambda (record) (.ref record 'id))
           (lambda (record) (.ref record 'parent))
           (lambda (record) (.ref record 'kind))
           (lambda (record name)
             (and (equal? name (.ref record 'field-name))
                  (.ref record 'field-value)))))
         (query (make-org-element-query
                 kind (if (equal? field "") #f field)
                 (if (equal? field "") #f value)))
         (operator (case expectation
                     ((0) 'exactly)
                     ((1) 'at-least)
                     ((2) 'at-most)
                     (else (error "invalid Org contract expectation"))))
         (assertion
          (make-org-contract-assertion
           "c-abi-evaluation" 'error query
           (make-org-contract-expectation operator expected)))
         (answer (org-contract-evaluate-assertion assertion graph scope)))
    (result-set! result 0 (org-contract-result-matched-count answer)
                 (if (org-contract-result-passed? answer) 1 0))
    0))

(begin-foreign
  (c-define (orgize-abi-revision)
    () unsigned-int32 "orgize_abi_revision" "extern" 1)

  (c-define (orgize-contract-evaluate rows count scope kind field value
                                      expectation expected result)
    (orgize_element_row-borrowed-ptr* unsigned-int32 int64
     UTF-8-string UTF-8-string UTF-8-string
     unsigned-int32 unsigned-int32 orgize_contract_result-borrowed-ptr*)
    int32 "orgize_contract_evaluate_scheme" "extern"
    (with-exception-catcher
     (lambda (exception)
       (when (orgize/bindings/c/orgize-native#result-valid? result)
         (orgize/bindings/c/orgize-native#result-set! result -1 0 0))
       -1)
     (lambda ()
       (orgize/bindings/c/orgize-native#evaluate-rows
        rows count scope kind field value expectation expected result)))))
