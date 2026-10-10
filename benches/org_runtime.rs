//! Same public parse/projection work, selected execution owner, matched queues.
use criterion::{Criterion, Throughput, criterion_group};
use orgize::{Org, org_aot::org_event_parser_digest, runtime_backend};
use std::{
    hint::black_box,
    sync::{Arc, Barrier, mpsc},
    thread,
    time::Instant,
};

const INPUTS: &[(&str, &str)] = &[
    ("doc.org", include_str!("fixtures/doc.org")),
    ("plain-links.org", include_str!("fixtures/plain-links.org")),
    ("quote-heavy.org", include_str!("fixtures/quote-heavy.org")),
];

#[path = "support/tokio_corpus.rs"]
mod tokio_corpus;

struct Callers {
    requests: Vec<mpsc::SyncSender<u64>>,
    finished: mpsc::Receiver<()>,
    ready: Arc<Barrier>,
    threads: Vec<thread::JoinHandle<()>>,
}

impl Callers {
    fn new(source: &'static str, count: usize) -> Self {
        let ready = Arc::new(Barrier::new(count + 1));
        let (completed, finished) = mpsc::channel();
        let mut requests = Vec::new();
        let mut threads = Vec::new();
        for _ in 0..count {
            let (request, incoming) = mpsc::sync_channel(0);
            let ready = Arc::clone(&ready);
            let completed = completed.clone();
            threads.push(thread::spawn(move || {
                while let Ok(iterations) = incoming.recv() {
                    if iterations == 0 {
                        break;
                    }
                    ready.wait();
                    for _ in 0..iterations {
                        black_box(Org::parse(black_box(source)));
                    }
                    completed.send(()).unwrap();
                }
            }));
            requests.push(request);
        }
        Self {
            requests,
            finished,
            ready,
            threads,
        }
    }

    fn measure(&self, iterations: u64) -> std::time::Duration {
        if iterations == 0 {
            return std::time::Duration::ZERO;
        }
        // Thread creation, input generation and warmup stay outside timing.
        for request in &self.requests {
            request.send(iterations).unwrap();
        }
        let started = Instant::now();
        self.ready.wait();
        for _ in &self.requests {
            self.finished.recv().unwrap();
        }
        started.elapsed()
    }
}

impl Drop for Callers {
    fn drop(&mut self) {
        for request in &self.requests {
            let _ = request.send(0);
        }
        for thread in self.threads.drain(..) {
            thread.join().unwrap();
        }
    }
}

fn benchmark(c: &mut Criterion) {
    assert!(
        !cfg!(feature = "runtime-profile"),
        "Criterion production measurements must be unprofiled"
    );
    // SAFETY: explicit benchmark startup precedes all caller/runtime workers.
    unsafe { orgize::initialize_native_runtime() }.expect("native benchmark startup");
    eprintln!(
        "org-runtime: backend={} program={} consumers=1 queue=64",
        runtime_backend().name(),
        org_event_parser_digest()
    );
    for &(fixture, source) in INPUTS {
        // Identical semantic admission and warmup for both features.
        assert_eq!(Org::parse(source).to_org(), source);
        black_box(Org::parse(source));
        let mut group = c.benchmark_group(format!(
            "OrgRuntime/{}/{}",
            runtime_backend().name(),
            fixture
        ));
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_function("warm-single", |b| {
            b.iter(|| black_box(Org::parse(black_box(source))))
        });
        for count in [2, 4, 8] {
            let callers = Callers::new(source, count);
            // One Criterion iteration is count completed documents.
            group.throughput(Throughput::Bytes(source.len() as u64 * count as u64));
            group.bench_function(format!("warm-callers-{count}"), |b| {
                b.iter_custom(|iterations| callers.measure(iterations))
            });
        }
        group.finish();
    }
    tokio_corpus::benchmark(c);
}
criterion_group!(benches, benchmark);

fn main() {
    benches();
    Criterion::default().configure_from_args().final_summary();
}
