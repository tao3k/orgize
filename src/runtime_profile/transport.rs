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
}

#[cfg(not(feature = "runtime-scheme"))]
pub(crate) struct OwnerReply<T> {
    pub(crate) value: T,
    #[cfg(feature = "runtime-profile")]
    timings: [u64; 3],
    #[cfg(feature = "runtime-profile")]
    completed: Stamp,
}

#[cfg(not(feature = "runtime-scheme"))]
impl<T> OwnerReply<T> {
    pub(crate) fn new(value: T, timings: [u64; 3], completed: Stamp) -> Self {
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
pub(crate) fn record_transport(timings: [u64; 3], handoff: u64) {
    for (name, value) in [
        ("native.owner_admission", timings[0]),
        ("native.owner_service_inclusive", timings[1]),
        ("native.result_copy", timings[2]),
        ("native.completion_handoff", handoff),
    ] {
        super::collector::record(name, value);
    }
}
