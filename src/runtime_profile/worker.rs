//! Explicit worker-local observation transfer; no shared collector or scheduler.
pub(crate) fn query_batch_completed(documents: usize) {
    #[cfg(feature = "runtime-profile")]
    {
        super::collector::record("query.documents_completed", documents as u64);
        // Explicit diagnostic trace: report completed work, never a timer heartbeat.
        if std::env::var_os("ORGIZE_QUERY_WORKER_TRACE").is_some_and(|value| value == "1") {
            eprintln!(
                "QUERY-WORKER completed_documents={documents} thread={:?}",
                std::thread::current().id()
            );
        }
    }
    #[cfg(not(feature = "runtime-profile"))]
    let _ = documents;
}

pub(crate) struct WorkerReply<T> {
    value: T,
    #[cfg(feature = "runtime-profile")]
    stages: Option<std::collections::BTreeMap<&'static str, u64>>,
}

pub(crate) fn worker<T>(enabled: bool, work: impl FnOnce() -> T) -> WorkerReply<T> {
    #[cfg(feature = "runtime-profile")]
    if enabled {
        let (value, stages) =
            super::collector::measure(|| super::collector::stage("query.worker_wall_sum", work));
        return WorkerReply {
            value,
            stages: Some(stages),
        };
    }
    let _ = enabled;
    WorkerReply {
        value: work(),
        #[cfg(feature = "runtime-profile")]
        stages: None,
    }
}

impl<T> WorkerReply<T> {
    pub(crate) fn receive(self) -> T {
        #[cfg(feature = "runtime-profile")]
        if let Some(stages) = self.stages {
            // Called only after join, on the requesting collector's thread.
            // These are worker duration sums, not caller elapsed time.
            for (name, nanos) in stages {
                super::collector::record(name, nanos);
            }
            super::collector::record("query.workers_completed", 1);
        }
        self.value
    }
}
