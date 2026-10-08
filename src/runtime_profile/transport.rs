//! Private durations cross an owner boundary; runtime handles never do.
#[derive(Clone, Copy)]
#[cfg(not(feature = "runtime-scheme"))]
pub(crate) struct Stamp {
    #[cfg(feature = "runtime-profile")]
    started: Option<std::time::Instant>,
}

#[cfg(not(feature = "runtime-scheme"))]
impl Stamp {
    pub(crate) fn start() -> Self {
        Self {
            #[cfg(feature = "runtime-profile")]
            started: super::is_active().then(std::time::Instant::now),
        }
    }

    pub(crate) fn child(self) -> Self {
        Self {
            #[cfg(feature = "runtime-profile")]
            started: self.started.map(|_| std::time::Instant::now()),
        }
    }

    pub(crate) fn elapsed(self) -> u64 {
        #[cfg(feature = "runtime-profile")]
        {
            self.started.map_or(0, |start| {
                u64::try_from(start.elapsed().as_nanos()).expect("native timing overflow")
            })
        }
        #[cfg(not(feature = "runtime-profile"))]
        {
            0
        }
    }

    #[cfg(all(feature = "runtime-profile", unix))]
    pub(crate) fn thread_cpu(self) -> Option<u64> {
        self.started.map(|_| {
            let mut value = std::mem::MaybeUninit::<libc::timespec>::uninit();
            // SAFETY: the OS writes this owned timespec; no Scheme handle crosses.
            assert_eq!(
                unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, value.as_mut_ptr()) },
                0,
                "native owner thread CPU clock"
            );
            // SAFETY: the successful clock call initialized both fields.
            let value = unsafe { value.assume_init() };
            u64::try_from(value.tv_sec)
                .expect("native owner CPU seconds")
                .checked_mul(1_000_000_000)
                .and_then(|seconds| seconds.checked_add(u64::try_from(value.tv_nsec).ok()?))
                .expect("native owner CPU overflow")
        })
    }
}

#[cfg(not(feature = "runtime-scheme"))]
pub(crate) struct OwnerReply<T> {
    pub(crate) value: T,
    #[cfg(feature = "runtime-profile")]
    timings: [u64; 4],
    #[cfg(feature = "runtime-profile")]
    completed: Stamp,
}

#[cfg(not(feature = "runtime-scheme"))]
impl<T> OwnerReply<T> {
    pub(crate) fn new(value: T, timings: [u64; 4], completed: Stamp) -> Self {
        #[cfg(not(feature = "runtime-profile"))]
        let _ = (timings, completed);
        Self {
            value,
            #[cfg(feature = "runtime-profile")]
            timings,
            #[cfg(feature = "runtime-profile")]
            completed,
        }
    }

    pub(crate) fn receive(self) -> T {
        #[cfg(feature = "runtime-profile")]
        record_transport(self.timings, self.completed.elapsed());
        self.value
    }
}

#[cfg(feature = "runtime-profile")]
pub(crate) fn record_transport(timings: [u64; 4], handoff: u64) {
    // Count, not a duration: includes every physical native parse/value reply.
    super::collector::record("native.requests_completed", 1);
    for (name, value) in [
        ("native.owner_admission", timings[0]),
        ("native.owner_service_inclusive", timings[1]),
        ("native.result_copy", timings[2]),
        ("native.owner_thread_cpu_inclusive", timings[3]),
        ("native.completion_handoff", handoff),
    ] {
        super::collector::record(name, value);
    }
}
