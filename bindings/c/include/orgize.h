#ifndef ORGIZE_H
#define ORGIZE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Borrowed UTF-8 strings. Each row is one admitted graph node with at most
 * one projected field; callers retain all input storage for the call. */
typedef struct {
  int64_t id;
  int64_t parent_id; /* -1 for a root */
  const char *kind;
  const char *field_name; /* empty string when absent */
  const char *field_value;
} orgize_element_row;

typedef struct {
  int32_t status; /* 0 success; -1 invalid input or evaluation error */
  uint32_t matched_count;
  int32_t passed; /* 0 or 1 on success */
} orgize_contract_result;

/* expectation: 0 exactly, 1 at least, 2 at most. All pointers are non-null
 * except rows when row_count is zero. Row string fields must be non-null.
 * A zero-length field_name means the query has no property predicate.
 * The caller owns both rows and result; the ABI retains no pointers. */
uint32_t orgize_abi_revision(void);
int32_t orgize_contract_evaluate(const orgize_element_row *rows,
                                 uint32_t row_count,
                                 int64_t scope_id,
                                 const char *query_kind,
                                 const char *field_name,
                                 const char *field_value,
                                 uint32_t expectation,
                                 uint32_t expected_count,
                                 orgize_contract_result *result);

#ifdef __cplusplus
}
#endif

#endif
