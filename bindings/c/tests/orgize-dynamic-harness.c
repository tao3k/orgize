#include "orgize_standalone.h"

#include <dlfcn.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

typedef int32_t (*init_fn)(void);
typedef void (*shutdown_fn)(void);
typedef uint32_t (*revision_fn)(void);
typedef int32_t (*evaluate_fn)(const orgize_element_row *, uint32_t,
                                int64_t, const char *, const char *,
                                const char *, uint32_t, uint32_t,
                                orgize_contract_result *);

static int symbol(void *library, const char *name, void *target, size_t size) {
  void *address = dlsym(library, name);
  if (address == NULL || size != sizeof(address)) return 0;
  memcpy(target, &address, sizeof(address));
  return 1;
}

int main(int argc, char **argv) {
  void *library;
  init_fn initialize;
  shutdown_fn shutdown;
  revision_fn revision;
  evaluate_fn evaluate;
  const orgize_element_row rows[] = {
      {0, -1, "org-data", "", ""},
      {1, 0, "headline", "title", "Evidence"},
  };
  orgize_contract_result result = {0, 0, 0};

  if (argc != 2) return 64;
  (void)fprintf(stderr, "[orgize-smoke] loading\n");
  library = dlopen(argv[1], RTLD_NOW | RTLD_LOCAL);
  if (library == NULL) {
    (void)fprintf(stderr, "%s\n", dlerror());
    return 1;
  }
  if (!symbol(library, "orgize_runtime_init", &initialize,
              sizeof(initialize)) ||
      !symbol(library, "orgize_runtime_shutdown", &shutdown,
              sizeof(shutdown)) ||
      !symbol(library, "orgize_abi_revision", &revision,
              sizeof(revision)) ||
      !symbol(library, "orgize_contract_evaluate", &evaluate,
              sizeof(evaluate))) {
    (void)dlclose(library);
    return 2;
  }
  (void)fprintf(stderr, "[orgize-smoke] initializing\n");
  if (initialize() != 0 || revision() != 1u) return 3;
  (void)fprintf(stderr, "[orgize-smoke] evaluating\n");
  if (evaluate(rows, 2u, 0, "headline", "title", "Evidence", 0u, 1u,
               &result) != 0 ||
      result.status != 0 || result.matched_count != 1u || result.passed != 1)
    return 4;
  if (evaluate(rows, 2u, 0, "headline", "title", "Missing", 0u, 1u,
               &result) != 0 ||
      result.status != 0 || result.matched_count != 0u || result.passed != 0)
    return 5;
  shutdown();
  (void)fprintf(stderr, "[orgize-smoke] stopped\n");
  if (initialize() == 0) return 6;
  if (dlclose(library) != 0) return 8;
  return 0;
}
