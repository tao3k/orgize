#ifndef ORGIZE_PARSER_H
#define ORGIZE_PARSER_H
#include <stddef.h>
#include <stdint.h>
#include "orgize.h"
#ifdef __cplusplus
extern "C" {
#endif
/* The maturin SDK extension embeds one Gerbil program. Python parsing uses
 * PyO3/Rust; this C ABI dispatches Contract calls to the same runtime owner. */
uint32_t orgize_shared_abi_revision(void);
/* Shared ABI revision 2: explicit startup, never implicit initialization.
 * First call: no live/new host children or concurrent stdio/signal users.
 * Scheme-owned terminals/stdio/subprocesses are outside this parser contract.
 * Returns 0 on admission, -1 on terminal startup failure. */
int32_t orgize_shared_runtime_initialize(void);
/* Same row contract as orgize.h, but dispatches on the shared runtime owner. */
int32_t orgize_shared_contract_evaluate(const orgize_element_row *rows,
    uint32_t row_count, int64_t scope_id, const char *kind,
    const char *field_name, const char *field_value,
    uint32_t expectation, uint32_t expected_count, orgize_contract_result *result);
#ifdef __cplusplus
}
#endif
#endif
