//! Thread-local collection with panic-safe request cleanup.
use std::{cell::RefCell, collections::BTreeMap, time::Instant};
thread_local! {
    static CURRENT: RefCell<Option<BTreeMap<&'static str, u64>>> = const { RefCell::new(None) };
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
    CURRENT.with(|slot| {
        let mut slot = slot.borrow_mut();
        assert!(slot.is_none(), "nested runtime measurements are forbidden");
        *slot = Some(BTreeMap::new());
    });
    let _reset = Reset;
    let result = work();
    let timings = CURRENT.with(|slot| slot.borrow_mut().take().unwrap());
    (result, timings)
}

pub(super) fn stage<T>(name: &'static str, work: impl FnOnce() -> T) -> T {
    if !is_active() {
        return work();
    }
    let started = Instant::now();
    let result = work();
    let elapsed = u64::try_from(started.elapsed().as_nanos()).unwrap();
    record(name, elapsed);
    result
}

pub(super) fn is_active() -> bool {
    CURRENT.with(|slot| slot.borrow().is_some())
}

pub(super) fn record(name: &'static str, nanos: u64) {
    CURRENT.with(|slot| {
        if let Some(timings) = slot.borrow_mut().as_mut() {
            *timings.entry(name).or_default() += nanos;
        }
    });
}
