/* The generated Gambit link source supplies ___VERSION and ORGIZE_LINKER. */
#include "gambit.h"

#include <stdint.h>
#include <stdio.h>

#ifndef ORGIZE_LINKER
#error "ORGIZE_LINKER must name the generated Gambit link-unit linker"
#endif

___BEGIN_NEW_LNK
___DEF_NEW_LNK(ORGIZE_LINKER)
___END_NEW_LNK

static int orgize_runtime_initialized = 0;
static int orgize_runtime_stopped = 0;

int32_t orgize_runtime_init(void) {
  ___setup_params_struct setup_params;
  ___SCMOBJ status;

  if (orgize_runtime_initialized) return 0;
  if (orgize_runtime_stopped) return -1;
  ___setup_params_reset(&setup_params);
  setup_params.version = ___VERSION;
  setup_params.linker = ORGIZE_LINKER;
  setup_params.debug_settings = ___DEBUG_SETTINGS_INITIAL;
  status = ___setup(&setup_params);
  if (status != ___FIX(___NO_ERR)) {
    (void)fprintf(stderr, "[orgize-native] setup-error=%ld\n",
                  (long)___INT(status));
    return -1;
  }
  orgize_runtime_initialized = 1;
  return 0;
}

void orgize_runtime_shutdown(void) {
  if (!orgize_runtime_initialized) return;
  ___cleanup();
  orgize_runtime_initialized = 0;
  orgize_runtime_stopped = 1;
}
