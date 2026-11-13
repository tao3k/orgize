//! Criterion-calibrated full batches; Tokio schedules each synchronous FFI call.
use criterion::{BenchmarkId, Criterion, Throughput};
use orgize::{Org, runtime_backend};
use std::{
    hint::black_box,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
};

async fn parse_batch(documents: Arc<Vec<String>>, callers: usize) -> usize {
    let next = Arc::new(AtomicUsize::new(0));
    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..callers {
        let documents = Arc::clone(&documents);
        let next = Arc::clone(&next);
        tasks.spawn(async move {
            let mut completed = 0;
            loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                if index >= documents.len() {
                    break;
                }
                let sources = Arc::clone(&documents);
                tokio::task::spawn_blocking(move || {
                    black_box(Org::parse(black_box(&sources[index])));
                })
                .await
                .expect("native parse task");
                completed += 1;
            }
            completed
        });
    }
    let mut completed = 0;
    while let Some(result) = tasks.join_next().await {
        completed += result.expect("Tokio caller");
    }
    assert_eq!(completed, documents.len());
    completed
}

fn admitted_corpus(count: usize) -> Arc<Vec<String>> {
    let documents = Arc::new((0..count).map(|index| {
        let (_, fixture) = super::INPUTS[index % super::INPUTS.len()];
        format!("#+TITLE: Native corpus document {index:05}\n{fixture}\n* Corpus-{index:05} 设计\nDocument {index:05}: [[id:corpus-{index:05}][evidence]].\n")
    }).collect::<Vec<_>>());
    for (index, source) in documents.iter().enumerate() {
        assert_eq!(Org::parse(source).to_org(), *source);
        if (index + 1) % 1000 == 0 {
            eprintln!("tokio-corpus preflight completed={}/{}", index + 1, count);
        }
    }
    documents
}

pub(super) fn benchmark(c: &mut Criterion) {
    let runtime = OnceLock::new();
    let mut group = c.benchmark_group(format!(
        "OrgRuntime/{}/tokio-corpus",
        runtime_backend().name()
    ));
    for count in [1000, 10000] {
        let documents = OnceLock::new();
        group.throughput(Throughput::Elements(count as u64));
        for callers in [1, 8, 64] {
            group.bench_with_input(
                BenchmarkId::new(count.to_string(), callers),
                &callers,
                |b, &callers| {
                    // Criterion applies its filter before this callback. Build
                    // and validate once per selected corpus, outside b.iter.
                    let documents = documents.get_or_init(|| admitted_corpus(count));
                    let runtime = runtime.get_or_init(|| {
                        tokio::runtime::Builder::new_multi_thread()
                            .worker_threads(4)
                            .max_blocking_threads(64)
                            .build()
                            .unwrap()
                    });
                    b.iter(|| {
                        black_box(runtime.block_on(parse_batch(Arc::clone(documents), callers)))
                    });
                },
            );
        }
    }
    group.finish();
}
