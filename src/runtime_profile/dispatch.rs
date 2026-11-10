//! Compile-time clock-free dispatch for ordinary builds.
#[inline]
pub(crate) fn is_active() -> bool {
    #[cfg(feature = "runtime-profile")]
    {
        super::collector::is_active()
    }
    #[cfg(not(feature = "runtime-profile"))]
    {
        false
    }
}

#[cfg(feature = "runtime-profile")]
pub(crate) fn native_tape(bytes: &[u8]) -> Result<&[u8], String> {
    let (fold, tape, cpu, payload) = super::wire::decode(bytes)?;
    super::collector::record("fold.service", fold);
    super::collector::record("event_tape.encode", tape);
    super::collector::record("fold.thread_cpu", cpu);
    Ok(payload)
}

#[inline]
pub(crate) fn stage<T>(name: &'static str, work: impl FnOnce() -> T) -> T {
    #[cfg(feature = "runtime-profile")]
    {
        super::collector::stage(name, work)
    }
    #[cfg(not(feature = "runtime-profile"))]
    {
        let _ = name;
        work()
    }
}

/// Attribute synchronous crossings without changing admission or scheduling.
#[inline]
pub(crate) fn operation<T>(operation: u8, work: impl FnOnce() -> T) -> T {
    #[cfg(feature = "runtime-profile")]
    {
        const NAMES: [&str; 27] = [
            "ffi.operation.00",
            "ffi.operation.01",
            "ffi.operation.02",
            "ffi.operation.03",
            "ffi.operation.04",
            "ffi.operation.05",
            "ffi.operation.06",
            "ffi.operation.07",
            "ffi.operation.08",
            "ffi.operation.09",
            "ffi.operation.10",
            "ffi.operation.11",
            "ffi.operation.12",
            "ffi.operation.13",
            "ffi.operation.14",
            "ffi.operation.15",
            "ffi.operation.16",
            "ffi.operation.17",
            "ffi.operation.18",
            "ffi.operation.19",
            "ffi.operation.20",
            "ffi.operation.21",
            "ffi.operation.22",
            "ffi.operation.23",
            "ffi.operation.24",
            "ffi.operation.25",
            "ffi.operation.26",
        ];
        stage(NAMES[usize::from(operation)], work)
    }
    #[cfg(not(feature = "runtime-profile"))]
    {
        let _ = operation;
        work()
    }
}

#[cfg(feature = "runtime-profile")]
pub(crate) fn record_native_batch(fields: [u64; 6]) {
    // VM/process interval sums are NOT exclusive costs when intervals overlap.
    for (name, value) in [
        "fold.batch_body_wall_sum",
        "fold.batch_owner_thread_cpu_sum",
        "fold.batch_process_cpu_interval_sum",
        "fold.batch_vm_gc_cpu_interval_sum",
        "fold.batch_vm_gc_wall_interval_sum",
        "fold.batch_vm_gc_count_interval_sum",
    ]
    .into_iter()
    .zip(fields)
    {
        super::collector::record(name, value);
    }
}
