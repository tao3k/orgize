/* Synchronization/ownership tests only; this does not substitute a parser. */
#include <assert.h>
#include <pthread.h>
#include <stdio.h>
#include <unistd.h>
#include <gambit.h>

static int checked_wait(pthread_cond_t *, pthread_mutex_t *);
#define pthread_cond_wait checked_wait
#include "orgize_runtime.h"
#undef pthread_cond_wait

/* Bootstrap is deliberately not entered in these private queue tests. */
___mod_or_lnk ___LNK_orgize__gerbil__program(___global_state_struct *state) {
  (void)state; abort();
}
int32_t gerbil_scheme_rust_runtime_init_program(
    ___mod_or_lnk (*linker)(___global_state_struct *)) {
  (void)linker; abort();
}
int32_t gerbil_scheme_rust_runtime_cleanup(void) { abort(); }
uint32_t gerbil_scheme_rust_abi_version(void) { abort(); }
int32_t orgize_scheme_runtime_run(void) { abort(); }
int32_t orgize_contract_evaluate(const orgize_element_row *rows, uint32_t count,
    int64_t scope, const char *kind, const char *field, const char *value,
    uint32_t expectation, uint32_t expected, orgize_contract_result *result) {
  (void)rows; (void)count; (void)scope; (void)kind; (void)field; (void)value;
  (void)expectation; (void)expected; (void)result; abort();
}

static pthread_mutex_t test_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t test_changed = PTHREAD_COND_INITIALIZER;
static int gate_enabled, parked, awakened, released, finished, slot_waiters;

static int checked_wait(pthread_cond_t *condition, pthread_mutex_t *lock) {
  int completion = condition != &orgize_runtime_changed &&
                   condition != &orgize_runtime_slots;
  pthread_mutex_lock(&test_lock);
  if (condition == &orgize_runtime_slots) ++slot_waiters;
  if (completion && gate_enabled) parked = 1;
  pthread_cond_broadcast(&test_changed);
  pthread_mutex_unlock(&test_lock);
  int status = pthread_cond_wait(condition, lock);
  pthread_mutex_lock(&test_lock);
  if (completion && gate_enabled) {
    awakened = 1;
    pthread_cond_broadcast(&test_changed);
    while (!released) pthread_cond_wait(&test_changed, &test_lock);
  }
  pthread_mutex_unlock(&test_lock);
  return status;
}

static void wait_flag(int *flag, int minimum) {
  pthread_mutex_lock(&test_lock);
  while (*flag < minimum) pthread_cond_wait(&test_changed, &test_lock);
  pthread_mutex_unlock(&test_lock);
}

static void *single_success(void *unused) {
  (void)unused;
  uint64_t profile[7] = {UINT64_MAX, 0, 0, 0, 0, 0, UINT64_MAX};
  orgize_runtime_job job = {0};
  job.profile = profile + 1;
  job.profile_submitted = orgize_runtime_profile_now();
  assert(orgize_runtime_submit(&job) == 0);
  assert(profile[0] == UINT64_MAX && profile[6] == UINT64_MAX);
  assert(job.profile_started >= job.profile_submitted);
  assert(job.profile_cpu_started > 0);
  assert(profile[4] >= job.profile_started);
  pthread_mutex_lock(&test_lock);
  finished = 1;
  pthread_cond_broadcast(&test_changed);
  pthread_mutex_unlock(&test_lock);
  return NULL;
}

static void completion_does_not_reacquire_queue(void) {
  pthread_t caller;
  orgize_runtime_ready();
  gate_enabled = 1;
  assert(pthread_create(&caller, NULL, single_success, NULL) == 0);
  wait_flag(&parked, 1);
  orgize_runtime_job *job = orgize_runtime_take();
  assert(job);
  orgize_runtime_complete(job, 0);
  wait_flag(&awakened, 1);
  pthread_mutex_lock(&orgize_runtime_lock);
  pthread_mutex_lock(&test_lock);
  released = 1;
  pthread_cond_broadcast(&test_changed);
  pthread_mutex_unlock(&test_lock);
  /* Caller must return while admission is deliberately locked. */
  wait_flag(&finished, 1);
  pthread_mutex_unlock(&orgize_runtime_lock);
  assert(pthread_join(caller, NULL) == 0);
  gate_enabled = 0;
  puts("QUEUE-OK completion-independent-of-admission");
}

static void *failure_caller(void *unused) {
  (void)unused;
  orgize_runtime_job job = {0};
  assert(orgize_runtime_submit(&job) == -1);
  return NULL;
}

static void *profile_failure_caller(void *unused) {
  (void)unused;
  uint64_t profile[7] = {UINT64_MAX, 0, 0, 0, 0, 0, UINT64_MAX};
  uint8_t *output = (uint8_t *)(uintptr_t)1;
  size_t length = 1;
  assert(orgize_scheme_parse_profiled(NULL, 0, &output, &length, profile + 1) == -1);
  assert(!output && length == 0);
  assert(profile[0] == UINT64_MAX && profile[6] == UINT64_MAX);
  return NULL;
}

static void profile_failure_releases_copied_result(void) {
  pthread_t caller;
  assert(pthread_create(&caller, NULL, profile_failure_caller, NULL) == 0);
  orgize_runtime_job *job = orgize_runtime_take();
  assert(job && job->profile);
  job->profile_cpu_started = UINT64_MAX; /* Inject counter regression. */
  const uint8_t tape[] = {1};
  assert(orgize_runtime_publish(job, tape, sizeof(tape)) == 0);
  assert(pthread_join(caller, NULL) == 0);
  puts("QUEUE-OK profile-failure-releases-copied-result");
}

static void failure_releases_all_ownership(void) {
  pthread_t callers[128];
  for (int i = 0; i < 128; ++i)
    assert(pthread_create(&callers[i], NULL, failure_caller, NULL) == 0);
  wait_flag(&slot_waiters, 64);
  pthread_mutex_lock(&orgize_runtime_lock);
  assert(orgize_runtime_queued == 64);
  pthread_mutex_unlock(&orgize_runtime_lock);
  orgize_runtime_job *active = orgize_runtime_take();
  assert(active);
  orgize_runtime_failed();
  for (int i = 0; i < 128; ++i) assert(pthread_join(callers[i], NULL) == 0);
  assert(!orgize_runtime_head && !orgize_runtime_tail &&
         !orgize_runtime_active && orgize_runtime_queued == 0);
  orgize_runtime_job rejected = {0};
  assert(orgize_runtime_submit(&rejected) == -1);
  puts("QUEUE-OK failure-active-queued-and-capacity-waiters=128");
}

static void *copy_caller(void *unused) {
  (void)unused;
  for (int i = 0; i < 16; ++i) {
    const uint8_t input[] = {0, 255, (uint8_t)i, 42};
    uint8_t *output = NULL;
    size_t length = 0;
    assert(orgize_scheme_parse(input, sizeof(input), &output, &length) == 0);
    assert(length == sizeof(input) && memcmp(input, output, length) == 0);
    orgize_scheme_free(output);
  }
  return NULL;
}

static void concurrent_copy_ownership(void) {
  pthread_t callers[64];
  /* Test-only state reset, not a supported production runtime restart. */
  orgize_runtime_ready();
  for (int i = 0; i < 64; ++i)
    assert(pthread_create(&callers[i], NULL, copy_caller, NULL) == 0);
  for (int i = 0; i < 1024; ++i) {
    orgize_runtime_job *job = orgize_runtime_take();
    assert(job);
    assert(orgize_runtime_publish(job, job->input, job->length) == 0);
    if ((i + 1) % 64 == 0) printf("QUEUE-PROGRESS completed=%d/1024\n", i + 1);
  }
  for (int i = 0; i < 64; ++i) assert(pthread_join(callers[i], NULL) == 0);
  assert(!orgize_runtime_head && !orgize_runtime_active && orgize_runtime_queued == 0);
  puts("QUEUE-OK copied-result-ownership callers=64 requests=1024");
}

int main(void) {
  setvbuf(stdout, NULL, _IONBF, 0);
  alarm(4); /* Deadlocks fail rather than extending the real-output watchdog. */
  orgize_runtime_job rejected = {0};
  assert(orgize_runtime_submit(&rejected) == -1);
  completion_does_not_reacquire_queue();
  profile_failure_releases_copied_result();
  failure_releases_all_ownership();
  concurrent_copy_ownership();
  alarm(0);
  puts("QUEUE-OK complete");
  return 0;
}
