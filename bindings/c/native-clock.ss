;;; Private diagnostic clock; no Gambit runtime changes or wall-clock timings.
(extern namespace: orgize/bindings/c/native-clock native-monotonic-ns native-thread-cpu-ns)
(export native-monotonic-ns native-thread-cpu-ns)
(begin-foreign
  (namespace ("orgize/bindings/c/native-clock#" native-monotonic-ns native-thread-cpu-ns))
  (c-declare #<<END-C
#include <stdint.h>
#if defined(_WIN32)
#include <windows.h>
static uint64_t orgize_monotonic_ns(void) {
  LARGE_INTEGER count, frequency;
  if (!QueryPerformanceCounter(&count) || !QueryPerformanceFrequency(&frequency)
      || count.QuadPart < 0 || frequency.QuadPart <= 0) return 0;
  uint64_t ticks = (uint64_t)count.QuadPart;
  uint64_t rate = (uint64_t)frequency.QuadPart;
  return (ticks / rate) * UINT64_C(1000000000)
       + (uint64_t)((long double)(ticks % rate) * 1000000000 / rate);
}
static uint64_t orgize_thread_cpu_ns(void) {
  FILETIME creation, exit, kernel, user;
  ULARGE_INTEGER k, u;
  if (!GetThreadTimes(GetCurrentThread(), &creation, &exit, &kernel, &user)) return 0;
  k.LowPart = kernel.dwLowDateTime; k.HighPart = kernel.dwHighDateTime;
  u.LowPart = user.dwLowDateTime; u.HighPart = user.dwHighDateTime;
  return (k.QuadPart + u.QuadPart) * UINT64_C(100);
}
#elif defined(__APPLE__)
#include <mach/mach.h>
#include <mach/mach_time.h>
#include <pthread.h>
static uint64_t orgize_monotonic_ns(void) {
  mach_timebase_info_data_t rate;
  if (mach_timebase_info(&rate) != KERN_SUCCESS || !rate.denom) return 0;
  uint64_t ticks = mach_absolute_time();
  return (ticks / rate.denom) * rate.numer
       + (ticks % rate.denom) * rate.numer / rate.denom;
}
static uint64_t orgize_thread_cpu_ns(void) {
  thread_basic_info_data_t value;
  mach_msg_type_number_t count = THREAD_BASIC_INFO_COUNT;
  if (thread_info(pthread_mach_thread_np(pthread_self()), THREAD_BASIC_INFO,
                  (thread_info_t)&value, &count) != KERN_SUCCESS) return 0;
  return ((uint64_t)value.user_time.seconds + value.system_time.seconds)
           * UINT64_C(1000000000)
       + ((uint64_t)value.user_time.microseconds + value.system_time.microseconds)
           * UINT64_C(1000);
}
#else
#include <time.h>
static uint64_t orgize_monotonic_ns(void) {
  struct timespec value;
  if (clock_gettime(CLOCK_MONOTONIC, &value) != 0 || value.tv_sec < 0) return 0;
  return (uint64_t)value.tv_sec * UINT64_C(1000000000) + (uint64_t)value.tv_nsec;
}
static uint64_t orgize_thread_cpu_ns(void) {
  struct timespec value;
  if (clock_gettime(CLOCK_THREAD_CPUTIME_ID, &value) != 0 || value.tv_sec < 0) return 0;
  return (uint64_t)value.tv_sec * UINT64_C(1000000000) + (uint64_t)value.tv_nsec;
}
#endif
END-C
  )
  (define native-monotonic-ns
    (c-lambda () unsigned-int64 "orgize_monotonic_ns"))
  (define native-thread-cpu-ns
    (c-lambda () unsigned-int64 "orgize_thread_cpu_ns")))
