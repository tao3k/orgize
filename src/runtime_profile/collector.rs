//! Thread-local collection with panic-safe request cleanup.
use std::{
    cell::RefCell,
    collections::BTreeMap,
    time::{Duration, Instant},
};

#[derive(Default)]
pub(super) struct Observation {
    stages: BTreeMap<&'static str, u64>,
    owner: Option<(Instant, Instant)>,
}

impl Observation {
    fn include_owner(&mut self, first: Instant, last: Instant) {
        self.owner = Some(match self.owner {
            Some((previous_first, previous_last)) => {
                (previous_first.min(first), previous_last.max(last))
            }
            None => (first, last),
        });
    }
}

thread_local! {
    static CURRENT: RefCell<Option<Observation>> = const { RefCell::new(None) };
}
struct Reset;
impl Drop for Reset {
    fn drop(&mut self) {
        CURRENT.with(|slot| {
            slot.borrow_mut().take();
        });
    }
}

/// Measure one synchronous request; collector state stays on its caller thread.
pub fn measure<T>(work: impl FnOnce() -> T) -> (T, BTreeMap<&'static str, u64>) {
    let start = Instant::now();
    let (result, mut observation) = collect(work);
    let end = Instant::now();
    if let Some((first, last)) = observation.owner
        && let Some(before) = first.checked_duration_since(start)
        && let Some(after) = end.checked_duration_since(last)
    {
        for (stage, duration) in [
            ("query.before_owner_window", before),
            ("query.owner_window", last.duration_since(first)),
            ("query.after_owner_window", after),
        ] {
            observation.stages.insert(stage, nanos(duration));
        }
    }
    (result, observation.stages)
}

pub(super) fn collect<T>(work: impl FnOnce() -> T) -> (T, Observation) {
    CURRENT.with(|slot| {
        let mut slot = slot.borrow_mut();
        assert!(slot.is_none(), "nested runtime measurements are forbidden");
        *slot = Some(Observation::default());
    });
    let _reset = Reset;
    let result = work();
    let observation = CURRENT.with(|slot| slot.borrow_mut().take().unwrap());
    (result, observation)
}

pub(super) fn stage<T>(name: &'static str, work: impl FnOnce() -> T) -> T {
    if !is_active() {
        return work();
    }
    let started = Instant::now();
    let result = work();
    let elapsed = nanos(started.elapsed());
    record(name, elapsed);
    result
}

pub(super) fn is_active() -> bool {
    CURRENT.with(|slot| slot.borrow().is_some())
}

pub(super) fn record(name: &'static str, nanos: u64) {
    CURRENT.with(|slot| {
        if let Some(observation) = slot.borrow_mut().as_mut() {
            *observation.stages.entry(name).or_default() += nanos;
        }
    });
}

pub(super) fn record_owner(service: u64, handoff: u64) {
    CURRENT.with(|slot| {
        if let Some(observation) = slot.borrow_mut().as_mut()
            && service != 0
            && let Some(last) = Instant::now().checked_sub(Duration::from_nanos(handoff))
            && let Some(first) = last.checked_sub(Duration::from_nanos(service))
        {
            // Reconstruct the interval from the reply durations. The host
            // clock read follows receipt by the cost of returning through FFI.
            observation.include_owner(first, last);
        }
    });
}

pub(super) fn receive(worker: Observation) {
    CURRENT.with(|slot| {
        if let Some(observation) = slot.borrow_mut().as_mut() {
            for (name, duration) in worker.stages {
                *observation.stages.entry(name).or_default() += duration;
            }
            if let Some((first, last)) = worker.owner {
                observation.include_owner(first, last);
            }
        }
    });
}

fn nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).expect("diagnostic timing overflow")
}
