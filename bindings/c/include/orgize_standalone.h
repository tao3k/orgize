#ifndef ORGIZE_STANDALONE_H
#define ORGIZE_STANDALONE_H

#include "orgize.h"

#ifdef __cplusplus
extern "C" {
#endif

/* Only the standalone liborgize link unit exports these lifecycle functions.
 * A host that embeds Orgize (for example Aitia) initializes its own Gambit
 * link unit instead. Calls must be serialized within one process. Gambit is
 * initialized once; after shutdown, this link unit cannot be restarted. */
int32_t orgize_runtime_init(void);
void orgize_runtime_shutdown(void);

#ifdef __cplusplus
}
#endif

#endif
