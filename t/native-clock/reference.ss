;;; POSIX test-only OS API cross-check. Never part of the parser program closure.
(extern namespace: orgize/t/native-clock/reference reference-thread-cpu-ns reference-process-cpu-ns reference-spin reference-spin-snapshot)
(export reference-thread-cpu-ns reference-process-cpu-ns reference-spin reference-spin-snapshot)
(begin-foreign
  (namespace ("orgize/t/native-clock/reference#" reference-thread-cpu-ns reference-process-cpu-ns reference-spin reference-spin-snapshot))
  (c-declare #<<END-C
#include <stdint.h>
#include <time.h>
static uint64_t orgize_reference_clock(clockid_t id) {
  struct timespec value;
  if (clock_gettime(id, &value) != 0 || value.tv_sec < 0) return 0;
  return (uint64_t)value.tv_sec * UINT64_C(1000000000) + (uint64_t)value.tv_nsec;
}
static uint64_t orgize_reference_thread(void) {
  return orgize_reference_clock(CLOCK_THREAD_CPUTIME_ID);
}
static uint64_t orgize_reference_process(void) {
  return orgize_reference_clock(CLOCK_PROCESS_CPUTIME_ID);
}
static _Thread_local uint64_t orgize_spin_sample[3];
static uint64_t orgize_reference_spin(uint64_t count) {
  uint64_t process_begin = orgize_reference_process();
  uint64_t thread_begin = orgize_reference_thread();
  uint64_t wall_begin = orgize_reference_clock(CLOCK_MONOTONIC);
  volatile uint64_t value = 0;
  for (uint64_t i = 0; i < count; ++i) value += i;
  uint64_t wall_end = orgize_reference_clock(CLOCK_MONOTONIC);
  uint64_t thread_end = orgize_reference_thread();
  uint64_t process_end = orgize_reference_process();
  orgize_spin_sample[0] = wall_end - wall_begin;
  orgize_spin_sample[1] = thread_end - thread_begin;
  orgize_spin_sample[2] = process_end - process_begin;
  return value;
}
static uint64_t orgize_reference_spin_snapshot(int index) {
  return index >= 0 && index < 3 ? orgize_spin_sample[index] : 0;
}
END-C
  )
  (define reference-thread-cpu-ns (c-lambda () unsigned-int64 "orgize_reference_thread"))
  (define reference-process-cpu-ns (c-lambda () unsigned-int64 "orgize_reference_process"))
  (define reference-spin (c-lambda (unsigned-int64) unsigned-int64 "orgize_reference_spin"))
  (define reference-spin-snapshot (c-lambda (int) unsigned-int64 "orgize_reference_spin_snapshot")))
