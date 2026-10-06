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
