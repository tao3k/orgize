/* Private in-process ingress for the Scheme-owned lifecycle. No kernel hooks. */
#ifndef ORGIZE_RUNTIME_H
#define ORGIZE_RUNTIME_H
#include "orgize.h"
#include <string.h>
#if defined(__unix__) || defined(__APPLE__)
#include <pthread.h>
#include <stdlib.h>
#include <time.h>

/* Private diagnostic requests only; no clock runs on ordinary admissions. */
static uint64_t orgize_runtime_profile_now(void) {
  struct timespec value;
  if (clock_gettime(CLOCK_MONOTONIC, &value) != 0 || value.tv_sec < 0) return 0;
  return (uint64_t)value.tv_sec * UINT64_C(1000000000) + (uint64_t)value.tv_nsec;
}

static uint64_t orgize_runtime_profile_cpu(void) {
  struct timespec value;
  if (clock_gettime(CLOCK_THREAD_CPUTIME_ID, &value) != 0 || value.tv_sec < 0) return 0;
  return (uint64_t)value.tv_sec * UINT64_C(1000000000) + (uint64_t)value.tv_nsec;
}

/* Borrowed requests remain live until their blocking submit call returns.
 * Only the unique Scheme actor enters the Scheme ABI or touches GC roots.
 * This first comparative lane deliberately has one processor/one worker,
 * matching the Rust lane. It does not claim foreign-thread attachment or SMP.
 */
typedef struct orgize_runtime_job {
  int kind, done, status;
  const uint8_t *input;
  size_t length;
  uint8_t *output;
  size_t output_length;
  const orgize_element_row *rows;
  uint32_t count, expectation, expected;
  int64_t scope;
  const char *query_kind, *field, *value;
  orgize_contract_result result;
  struct orgize_runtime_job *next;
  pthread_mutex_t completion_lock;
  pthread_cond_t completion;
  uint64_t *profile;
  uint64_t profile_submitted, profile_started, profile_cpu_started;
  int profile_failed;
} orgize_runtime_job;

extern ___mod_or_lnk ___LNK_orgize__gerbil__program(___global_state_struct *);
extern int32_t gerbil_scheme_rust_runtime_init_program(
    ___mod_or_lnk (*)(___global_state_struct *));
extern int32_t gerbil_scheme_rust_runtime_cleanup(void);
extern uint32_t gerbil_scheme_rust_abi_version(void);
extern int32_t orgize_scheme_runtime_run(void);

static pthread_mutex_t orgize_runtime_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t orgize_runtime_changed = PTHREAD_COND_INITIALIZER;
static pthread_cond_t orgize_runtime_slots = PTHREAD_COND_INITIALIZER;
static pthread_once_t orgize_runtime_once = PTHREAD_ONCE_INIT;
/* 0 not started, 1 starting, 2 running, 3 terminal failure. */
static int orgize_runtime_state;
static size_t orgize_runtime_queued;
static orgize_runtime_job *orgize_runtime_head, *orgize_runtime_tail;
static orgize_runtime_job *orgize_runtime_active;

/* Queue membership and response publication have separate lock domains.
 * A completed caller never reacquires the busy admission queue's mutex.
 * When both are needed, always acquire queue then completion, never reverse. */
static void orgize_runtime_notify(orgize_runtime_job *job, int status) {
  pthread_mutex_lock(&job->completion_lock);
  job->status = status;
  job->done = 1;
  pthread_cond_signal(&job->completion);
  pthread_mutex_unlock(&job->completion_lock);
}

static void orgize_runtime_ready(void) {
  pthread_mutex_lock(&orgize_runtime_lock);
  orgize_runtime_state = 2;
  pthread_cond_broadcast(&orgize_runtime_changed);
  pthread_mutex_unlock(&orgize_runtime_lock);
}

static void orgize_runtime_failed(void) {
  pthread_mutex_lock(&orgize_runtime_lock);
  orgize_runtime_state = 3;
  while (orgize_runtime_head) {
    orgize_runtime_job *job = orgize_runtime_head;
    orgize_runtime_head = job->next;
    orgize_runtime_notify(job, -1);
  }
  if (orgize_runtime_active) {
    orgize_runtime_notify(orgize_runtime_active, -1);
    orgize_runtime_active = NULL;
  }
  orgize_runtime_tail = NULL;
  orgize_runtime_queued = 0;
  pthread_cond_broadcast(&orgize_runtime_changed);
  pthread_cond_broadcast(&orgize_runtime_slots);
  pthread_mutex_unlock(&orgize_runtime_lock);
}

static void *orgize_runtime_boot(void *unused) {
  (void)unused;
  if (gerbil_scheme_rust_runtime_init_program(___LNK_orgize__gerbil__program) != 0) {
    orgize_runtime_failed();
    return NULL;
  }
  if (gerbil_scheme_rust_abi_version() == 1) {
    /* Enters the native actor lifecycle, not a Rust request receiver. */
    orgize_scheme_runtime_run();
  }
  gerbil_scheme_rust_runtime_cleanup();
  orgize_runtime_failed();
  return NULL;
}

static void orgize_runtime_start(void) {
  pthread_t thread;
  pthread_attr_t attributes;
  pthread_mutex_lock(&orgize_runtime_lock);
  orgize_runtime_state = 1;
  pthread_mutex_unlock(&orgize_runtime_lock);
  if (pthread_attr_init(&attributes) != 0) {
    orgize_runtime_failed();
    return;
  }
  int status = pthread_attr_setstacksize(&attributes, 2 * 1024 * 1024);
  if (status == 0)
    status = pthread_create(&thread, &attributes, orgize_runtime_boot, NULL);
  pthread_attr_destroy(&attributes);
  if (status != 0) { orgize_runtime_failed(); return; }
  /* The program owns one process-lifetime native service. Restart is forbidden
   * by the shared lifecycle owner; no Python finalizer tears it down mid-call. */
  pthread_detach(thread);
}

static orgize_runtime_job *orgize_runtime_take(void) {
  pthread_mutex_lock(&orgize_runtime_lock);
  while (!orgize_runtime_head && (orgize_runtime_state == 1 || orgize_runtime_state == 2))
    pthread_cond_wait(&orgize_runtime_changed, &orgize_runtime_lock);
  orgize_runtime_job *job = orgize_runtime_head;
  if (job) {
    orgize_runtime_head = job->next;
    if (!orgize_runtime_head) orgize_runtime_tail = NULL;
    --orgize_runtime_queued;
    orgize_runtime_active = job;
    if (job->profile) {
      job->profile_started = orgize_runtime_profile_now();
      job->profile_cpu_started = orgize_runtime_profile_cpu();
      if (!job->profile_started || job->profile_started < job->profile_submitted)
        job->profile_failed = 1;
      else job->profile[0] = job->profile_started - job->profile_submitted;
    }
    pthread_cond_signal(&orgize_runtime_slots);
  }
  pthread_mutex_unlock(&orgize_runtime_lock);
  return job;
}

static void orgize_runtime_complete(orgize_runtime_job *job, int status) {
  pthread_mutex_lock(&orgize_runtime_lock);
  orgize_runtime_active = NULL;
  pthread_mutex_unlock(&orgize_runtime_lock);
  if (job->profile) {
    uint64_t cpu = orgize_runtime_profile_cpu();
    uint64_t now = orgize_runtime_profile_now();
    if (!now || now < job->profile_started || !job->profile_cpu_started ||
        cpu < job->profile_cpu_started || job->profile_failed) status = -1;
    else {
      job->profile[1] = now - job->profile_started;
      job->profile[3] = now;
      job->profile[4] = cpu - job->profile_cpu_started;
    }
  }
  orgize_runtime_notify(job, status);
}

/* The actor calls the Scheme parser directly. This only copies its live tape
 * during a nonallocating-GC C call; no Scheme value/root escapes to callers. */
static int32_t orgize_runtime_publish(orgize_runtime_job *job,
    const uint8_t *tape, size_t length) {
  uint64_t started = job->profile ? orgize_runtime_profile_now() : 0;
  job->output = length ? malloc(length) : NULL;
  if (!job->output) { orgize_runtime_complete(job, -1); return -1; }
  memcpy(job->output, tape, length);
  job->output_length = length;
  if (job->profile) {
    uint64_t end = orgize_runtime_profile_now();
    if (!started || end < started) job->profile_failed = 1;
    else job->profile[2] = end - started;
  }
  orgize_runtime_complete(job, 0);
  return 0;
}

static void orgize_runtime_fail_job(orgize_runtime_job *job) {
  orgize_runtime_complete(job, -1);
}

static void orgize_runtime_process_contract(orgize_runtime_job *job) {
  int status = orgize_contract_evaluate(job->rows, job->count, job->scope,
      job->query_kind, job->field, job->value, job->expectation,
      job->expected, &job->result);
  orgize_runtime_complete(job, status);
}

/* Only the explicit Rust/C/Python startup boundary may start the service. */
int32_t orgize_scheme_runtime_initialize(void) {
  pthread_once(&orgize_runtime_once, orgize_runtime_start);
  pthread_mutex_lock(&orgize_runtime_lock);
  while (orgize_runtime_state == 1)
    pthread_cond_wait(&orgize_runtime_changed, &orgize_runtime_lock);
  int status = orgize_runtime_state == 2 ? 0 : -1;
  pthread_mutex_unlock(&orgize_runtime_lock);
  return status;
}

static int orgize_runtime_submit(orgize_runtime_job *job) {
  if (pthread_mutex_init(&job->completion_lock, NULL) != 0) return -1;
  if (pthread_cond_init(&job->completion, NULL) != 0) {
    pthread_mutex_destroy(&job->completion_lock);
    return -1;
  }
  pthread_mutex_lock(&orgize_runtime_lock);
  while (orgize_runtime_state == 2 && orgize_runtime_queued >= 64)
    pthread_cond_wait(&orgize_runtime_slots, &orgize_runtime_lock);
  if (orgize_runtime_state != 2) {
    pthread_mutex_unlock(&orgize_runtime_lock);
    pthread_cond_destroy(&job->completion);
    pthread_mutex_destroy(&job->completion_lock);
    return -1;
  }
  ++orgize_runtime_queued;
  if (orgize_runtime_tail) orgize_runtime_tail->next = job;
  else orgize_runtime_head = job;
  orgize_runtime_tail = job;
  pthread_cond_signal(&orgize_runtime_changed);
  pthread_mutex_unlock(&orgize_runtime_lock);
  pthread_mutex_lock(&job->completion_lock);
  while (!job->done) pthread_cond_wait(&job->completion, &job->completion_lock);
  int status = job->status;
  pthread_mutex_unlock(&job->completion_lock);
  pthread_cond_destroy(&job->completion);
  pthread_mutex_destroy(&job->completion_lock);
  return status;
}

static int32_t orgize_scheme_parse_impl(const uint8_t *input, size_t length,
                            uint8_t **output, size_t *output_length,
                            uint64_t *profile) {
  if (!output || !output_length) return -1;
  *output = NULL;
  *output_length = 0;
  if (!input && length) return -1;
  orgize_runtime_job job = {0};
  job.input = input;
  job.length = length;
  job.profile = profile;
  if (profile) {
    memset(profile, 0, 5 * sizeof(uint64_t));
    job.profile_submitted = orgize_runtime_profile_now();
    if (!job.profile_submitted) return -1;
  }
  int status = orgize_runtime_submit(&job);
  if (profile && status == 0) {
    uint64_t now = orgize_runtime_profile_now();
    if (!profile[3] || now < profile[3]) status = -1;
    else profile[3] = now - profile[3];
  }
  if (status != 0) {
    free(job.output);
    job.output = NULL;
    job.output_length = 0;
  }
  *output = job.output;
  *output_length = job.output_length;
  return status;
}

int32_t orgize_scheme_parse(const uint8_t *input, size_t length,
                            uint8_t **output, size_t *output_length) {
  return orgize_scheme_parse_impl(input, length, output, output_length, NULL);
}

/* Private timing projection, not part of the public orgize.h ABI. */
int32_t orgize_scheme_parse_profiled(const uint8_t *input, size_t length,
    uint8_t **output, size_t *output_length, uint64_t *profile) {
  if (!profile) return -1;
  return orgize_scheme_parse_impl(input, length, output, output_length, profile);
}

void orgize_scheme_free(uint8_t *output) { free(output); }

int32_t orgize_scheme_contract(const orgize_element_row *rows, uint32_t count,
    int64_t scope, const char *kind, const char *field, const char *value,
    uint32_t expectation, uint32_t expected, orgize_contract_result *result) {
  if (!result) return -1;
  *result = (orgize_contract_result){.status = -1};
  if ((!rows && count) || !kind || !field || !value) return -1;
  orgize_runtime_job job = {0};
  job.kind = 1;
  job.rows = rows;
  job.count = count;
  job.scope = scope;
  job.query_kind = kind;
  job.field = field;
  job.value = value;
  job.expectation = expectation;
  job.expected = expected;
  job.result.status = -1;
  int status = orgize_runtime_submit(&job);
  *result = job.result;
  return status;
}
#else
/* The Rust lane may link this program on other platforms, but the Scheme
 * lane is rejected by its Rust feature gate. No second executor or fallback. */
typedef struct orgize_runtime_job {
  int kind;
  const uint8_t *input;
  size_t length;
} orgize_runtime_job;
static void *orgize_runtime_take(void) { return NULL; }
static void orgize_runtime_process_contract(void *job) { (void)job; }
static void orgize_runtime_fail_job(void *job) { (void)job; }
static int32_t orgize_runtime_publish(void *job, const uint8_t *bytes, size_t length) {
  (void)job; (void)bytes; (void)length; return -1;
}
static void orgize_runtime_ready(void) {}
#endif
#endif
