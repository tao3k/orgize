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
    super::collector::record("native.scheme_fold", fold);
    super::collector::record("native.tape_encode", tape);
    super::collector::record("native.scheme_thread_cpu", cpu);
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
        const NAMES: [&str; 25] = [
            "native.operation.00",
            "native.operation.01",
            "native.operation.02",
            "native.operation.03",
            "native.operation.04",
            "native.operation.05",
            "native.operation.06",
            "native.operation.07",
            "native.operation.08",
            "native.operation.09",
            "native.operation.10",
            "native.operation.11",
            "native.operation.12",
            "native.operation.13",
            "native.operation.14",
            "native.operation.15",
            "native.operation.16",
            "native.operation.17",
            "native.operation.18",
            "native.operation.19",
            "native.operation.20",
            "native.operation.21",
            "native.operation.22",
            "native.operation.23",
            "native.operation.24",
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
        "native.batch_body_wall_sum",
        "native.batch_owner_thread_cpu_sum",
        "native.batch_process_cpu_interval_sum",
        "native.batch_vm_gc_cpu_interval_sum",
        "native.batch_vm_gc_wall_interval_sum",
        "native.batch_vm_gc_count_interval_sum",
    ]
    .into_iter()
    .zip(fields)
    {
        super::collector::record(name, value);
    }
}
