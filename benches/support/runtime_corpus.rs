//! Fixed-count corpus qualification in the existing org_runtime entrypoint.
//! Criterion calibration would multiply 10,000 documents by its sample count;
//! this lane instead records exactly N distinct documents per batch/repetition.
use orgize::{Org, org_aot::org_event_parser_digest, runtime_backend};
use std::{
    sync::{
        Arc, Barrier,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

pub(super) struct Sample {
    pub(super) index: usize,
    pub(super) elapsed_ns: u64,
    pub(super) submitted_to_parse_ns: u64,
    pub(super) artifact: Option<blake3::Hash>,
    pub(super) stages: serde_json::Value,
}

pub(super) fn parse_sample(
    source: &str,
    index: usize,
    submitted: Instant,
) -> Result<Sample, String> {
    let started = Instant::now();
    #[cfg(feature = "runtime-profile")]
    let (result, timings) = orgize::runtime_profile::measure(|| Org::try_parse(source));
    #[cfg(not(feature = "runtime-profile"))]
    let result = Org::try_parse(source);
    let parsed = result.map_err(|error| format!("document {index}: {error:?}"))?;
    let elapsed_ns = u64::try_from(started.elapsed().as_nanos()).unwrap();
    let submitted_to_parse_ns = u64::try_from(submitted.elapsed().as_nanos()).unwrap();
    let validation = Instant::now();
    let performance = std::env::var("ORGIZE_RUNTIME_CORPUS_MODE").as_deref() == Ok("performance");
    if !performance && parsed.to_org() != source {
        return Err(format!("document {index}: lossless mismatch"));
    }
    let artifact = (!performance).then(|| {
        blake3::hash(format!("{:#?}\n{:?}", parsed.syntax(), parsed.records()).as_bytes())
    });
    let validation_ns = validation.elapsed().as_nanos() as u64;
    #[cfg(feature = "runtime-profile")]
    let mut stages = serde_json::to_value(timings).unwrap();
    #[cfg(not(feature = "runtime-profile"))]
    let mut stages = serde_json::json!({});
    stages["validation_checksum"] = serde_json::json!(validation_ns);
    let dropping = Instant::now();
    drop(parsed);
    stages["document_drop"] = serde_json::json!(dropping.elapsed().as_nanos() as u64);
    Ok(Sample {
        index,
        elapsed_ns,
        submitted_to_parse_ns,
        artifact,
        stages,
    })
}

pub(super) fn configuration(key: &str, fallback: &str, max: usize) -> Vec<usize> {
    let values: Vec<usize> = std::env::var(key)
        .unwrap_or_else(|_| fallback.to_owned())
        .split(',')
        .map(|part| part.parse().expect("comma-separated positive integers"))
        .collect();
    assert!(!values.is_empty());
    assert!(values.iter().all(|&value| value > 0 && value <= max));
    values
}

pub(super) fn corpus(count: usize) -> Vec<String> {
    (0..count)
        .map(|index| {
            let (_, fixture) = super::INPUTS[index % super::INPUTS.len()];
            format!(
                "#+TITLE: Native corpus document {index:05}\n{fixture}\n* Corpus-{index:05} 设计\nDocument {index:05}: [[id:corpus-{index:05}][evidence]].\n"
            )
        })
        .collect()
}

pub(super) fn source_identity(documents: &[String]) -> String {
    let mut hash = blake3::Hasher::new();
    hash.update(b"orgize.runtime-corpus-source.v1\0");
    for source in documents {
        hash.update(&(source.len() as u64).to_le_bytes());
        hash.update(source.as_bytes());
    }
    hash.finalize().to_hex().to_string()
}

// Native program identity does not cover the downstream Rust adapter.
// Bind receipts to these compiled adapter sources, independent of runtime feature.
pub(super) fn adapter_identity() -> String {
    let mut hash = blake3::Hasher::new();
    hash.update(b"orgize.runtime-corpus-adapter.v1\0");
    for source in [
        include_bytes!("../../src/org_native_events/mod.rs").as_slice(),
        include_bytes!("../../src/org_native_events/transport.rs").as_slice(),
        include_bytes!("../../src/org_native_events/batch.rs").as_slice(),
        include_bytes!("../../src/native_startup.rs").as_slice(),
        include_bytes!("../../src/runtime_backend.rs").as_slice(),
        include_bytes!("../../src/org_aot.rs").as_slice(),
        include_bytes!("../../src/config.rs").as_slice(),
        include_bytes!("../../src/org_element_query/mod.rs").as_slice(),
        include_bytes!("../../src/org_element_query/source_observation.rs").as_slice(),
        include_bytes!("../../src/org_native_semantic_functions.rs").as_slice(),
        include_bytes!("../../src/org_aot_affiliation.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/aot_link_resolution.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/lifecycle.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/aot_table_projection.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/aot_radio_projection.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/org_native_radio.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/org_native_values.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/publishing.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/property_model.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/lifecycle_model.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/agenda_time.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/runtime_metadata.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/settings.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/clock_table_time.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/clock_table_properties.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/property_profile.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/progress.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/link_protocols.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/tag_vocabulary.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/projection.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/includes.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/source_block_headers.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/source_block_references.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/org_contract/core.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/org_contract_model.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/agenda_match.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/datetree.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/special_properties.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/column_views.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/column_summaries.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/clock_rollup.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/property_schema.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/agenda_model.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/sdd_model.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/workspace_index.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/org_interactive.rs").as_slice(),
        include_bytes!("../../src/semantic_ast/table_visualization.rs").as_slice(),
        include_bytes!("../../src/org.rs").as_slice(),
        include_bytes!("../../src/c_ffi.rs").as_slice(),
        include_bytes!("../../src/runtime_profile/mod.rs").as_slice(),
        include_bytes!("../../src/runtime_profile/collector.rs").as_slice(),
        include_bytes!("../../src/runtime_profile/dispatch.rs").as_slice(),
        include_bytes!("../../src/runtime_profile/transport.rs").as_slice(),
        include_bytes!("../../src/runtime_profile/wire.rs").as_slice(),
    ] {
        hash.update(&(source.len() as u64).to_le_bytes());
        hash.update(source);
    }
    hash.finalize().to_hex().to_string()
}

pub(super) fn percentile(sorted: &[u64], numerator: usize) -> u64 {
    // Nearest-rank percentile, including p99 over all completed documents.
    sorted[(sorted.len() * numerator).div_ceil(100).saturating_sub(1)]
}

#[cfg(unix)]
fn peak_rss_bytes() -> Option<u64> {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: getrusage initializes this output struct on successful return.
    if unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) } != 0 {
        return None;
    }
    let peak = u64::try_from(unsafe { usage.assume_init() }.ru_maxrss).ok()?;
    #[cfg(target_os = "macos")]
    return Some(peak);
    #[cfg(not(target_os = "macos"))]
    peak.checked_mul(1024)
}

#[cfg(not(unix))]
fn peak_rss_bytes() -> Option<u64> {
    None
}

fn batch(
    documents: &Arc<Vec<String>>,
    callers: usize,
    repetition: usize,
    driver: &str,
) -> serde_json::Value {
    let bytes: usize = documents.iter().map(String::len).sum();
    let corpus_digest = source_identity(documents);
    let next = Arc::new(AtomicUsize::new(0));
    let ready = Arc::new(Barrier::new(callers + 1));
    // Tokio callers await bounded blocking workers: the public native call is
    // synchronous and must never block Tokio's async scheduler threads.
    let runtime = (driver == "tokio").then(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .max_blocking_threads(callers)
            .enable_time()
            .build()
            .unwrap()
    });
    let mut tasks = Vec::new();
    // Completion telemetry, not an additional parsing request scheduler.
    let (completed, incoming) = mpsc::sync_channel::<Result<Sample, String>>(callers);
    let mut samples: Vec<Option<Sample>> = (0..documents.len()).map(|_| None).collect();
    let rss_before = peak_rss_bytes();
    let elapsed = thread::scope(|scope| {
        for _ in 0..callers {
            let completed = completed.clone();
            let next = Arc::clone(&next);
            let ready = Arc::clone(&ready);
            let documents = Arc::clone(documents);
            if let Some(runtime) = &runtime {
                tasks.push(runtime.spawn(async move {
                    // Only the untimed startup gate uses a blocking barrier.
                    tokio::task::spawn_blocking(move || ready.wait())
                        .await
                        .expect("blocking startup gate failed");
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        if index >= documents.len() {
                            break;
                        }
                        let documents = Arc::clone(&documents);
                        let completed = completed.clone();
                        let submitted = Instant::now();
                        // One submission and async await per document, not one
                        // long blocking batch with an idle Tokio scheduler.
                        let keep_going = tokio::task::spawn_blocking(move || {
                            let sample = parse_sample(&documents[index], index, submitted);
                            let passed = sample.is_ok();
                            // Completion backpressure also stays off async workers.
                            completed.send(sample).is_ok() && passed
                        })
                        .await
                        .expect("blocking document task failed");
                        if !keep_going {
                            break;
                        }
                    }
                }));
            } else {
                scope.spawn(move || {
                    ready.wait();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(source) = documents.get(index) else {
                            break;
                        };
                        let sample = parse_sample(source, index, Instant::now());
                        let passed = sample.is_ok();
                        if completed.send(sample).is_err() || !passed {
                            break;
                        }
                    }
                });
            }
        }
        drop(completed);
        let started = Instant::now();
        ready.wait();
        let mut last_completed = started;
        let mut last_log = started;
        let mut count = 0;
        while count < documents.len() {
            match incoming.recv_timeout(Duration::from_secs(1)) {
                Ok(Ok(sample)) => {
                    let index = sample.index;
                    assert!(samples[index].is_none(), "duplicate completion");
                    samples[index] = Some(sample);
                    count += 1;
                    last_completed = Instant::now();
                }
                Ok(Err(error)) => {
                    eprintln!("org-runtime-corpus FAILED: {error}");
                    std::process::exit(1);
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    eprintln!("org-runtime-corpus FAILED: incomplete worker results");
                    std::process::exit(1);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if last_completed.elapsed() >= Duration::from_secs(5) {
                        // Exit the benchmark process instead of joining a hung
                        // native call; never publish partial completion as green.
                        eprintln!("org-runtime-corpus FAILED: five seconds without completion");
                        std::process::exit(2);
                    }
                }
            }
            if last_log.elapsed() >= Duration::from_secs(1) || count == documents.len() {
                eprintln!(
                    "org-runtime-corpus backend={} docs={} callers={} repeat={} complete={}/{} elapsed_s={:.2}",
                    runtime_backend().name(),
                    documents.len(),
                    callers,
                    repetition,
                    count,
                    documents.len(),
                    started.elapsed().as_secs_f64()
                );
                last_log = Instant::now();
            }
        }
        started.elapsed()
    });
    if let Some(runtime) = &runtime {
        runtime.block_on(async {
            for task in tasks {
                task.await.expect("Tokio caller failed");
            }
        });
    }
    let mut artifact = blake3::Hasher::new();
    artifact.update(b"orgize.runtime-corpus-artifact.v1\0");
    let mut latencies = Vec::with_capacity(samples.len());
    let mut submitted_latencies = Vec::with_capacity(samples.len());
    for sample in samples {
        let sample = sample.expect("every document completed");
        artifact.update(&(sample.index as u64).to_le_bytes());
        artifact.update(
            sample
                .artifact
                .expect("qualification checksum required")
                .as_bytes(),
        );
        latencies.push(sample.elapsed_ns);
        submitted_latencies.push(sample.submitted_to_parse_ns);
    }
    latencies.sort_unstable();
    submitted_latencies.sort_unstable();
    serde_json::json!({
        "schema": "orgize.runtime-corpus.v1",
        "backend": runtime_backend().name(),
        "driver": driver,
        "tokio_workers": if runtime.is_some() { 4 } else { 0 },
        "blocking_callers_limit": callers,
        "program": org_event_parser_digest(),
        "adapter": adapter_identity(),
        "benchmark": blake3::hash(include_bytes!("runtime_corpus.rs")).to_hex().to_string(),
        "architecture": std::env::consts::ARCH,
        "corpus": corpus_digest, "artifact": artifact.finalize().to_hex().to_string(),
        "documents": documents.len(), "distinct_documents": documents.len(),
        "completed": latencies.len(), "bytes": bytes, "callers": callers,
        "repetition": repetition, "consumers": 1, "queue_capacity": 64,
        "elapsed_ns": elapsed.as_nanos(),
        "documents_per_second": documents.len() as f64 / elapsed.as_secs_f64(),
        "bytes_per_second": bytes as f64 / elapsed.as_secs_f64(),
        "parse_latency_ns": {
            "p50": percentile(&latencies, 50), "p95": percentile(&latencies, 95),
            "p99": percentile(&latencies, 99), "max": latencies.last(),
        },
        "submitted_to_parse_latency_ns": {
            "p50": percentile(&submitted_latencies, 50), "p95": percentile(&submitted_latencies, 95),
            "p99": percentile(&submitted_latencies, 99), "max": submitted_latencies.last(),
        },
        "submission_latency_scope": "driver submission through parse/projection completion; excludes validation/checksum, includes Tokio blocking-pool admission",
        "process_peak_rss_bytes_before": rss_before,
        "process_peak_rss_bytes_after": peak_rss_bytes(),
        "wall_time_includes": "parse, lossless validation, tree/graph checksum, drop and completion collection",
        "rss_scope": "whole-process cumulative high-water mark, not native GC bytes or a live RSS delta",
    })
}

pub(super) fn run() {
    assert!(
        std::env::var("ORGIZE_RUNTIME_CORPUS_MODE").as_deref() != Ok("performance"),
        "use the separately labelled parallel performance lane"
    );
    let driver = std::env::var("ORGIZE_RUNTIME_CORPUS_DRIVER").unwrap_or_else(|_| "std".into());
    assert!(
        matches!(driver.as_str(), "std" | "tokio"),
        "select std or tokio corpus driver"
    );
    let sizes = configuration("ORGIZE_RUNTIME_CORPUS_DOCS", "1000,10000", 10000);
    let callers = configuration("ORGIZE_RUNTIME_CORPUS_CALLERS", "1,8", 64);
    let repeats = configuration("ORGIZE_RUNTIME_CORPUS_REPEATS", "1", 10);
    assert_eq!(repeats.len(), 1);
    let output = std::env::var_os("ORGIZE_RUNTIME_CORPUS_OUTPUT").map(std::path::PathBuf::from);
    if let Some(path) = &output {
        assert!(!path.exists(), "refuse to overwrite corpus receipts");
    }
    // Input preparation and one native warmup are outside every batch timer.
    for &(_, source) in super::INPUTS {
        assert_eq!(Org::parse(source).to_org(), source);
    }
    let mut receipts = Vec::new();
    for count in sizes {
        let documents = Arc::new(corpus(count));
        let unique: std::collections::HashSet<_> = documents.iter().collect();
        assert_eq!(unique.len(), count, "each document must be different");
        let mut reference_artifact = None;
        for &caller_count in &callers {
            for repetition in 1..=repeats[0] {
                let receipt = batch(&documents, caller_count, repetition, &driver);
                let artifact = receipt["artifact"].as_str().unwrap().to_owned();
                if let Some(reference) = &reference_artifact {
                    assert_eq!(
                        reference, &artifact,
                        "callers/repetitions changed parse artifacts"
                    );
                } else {
                    reference_artifact = Some(artifact);
                }
                println!("{receipt}");
                receipts.push(receipt);
            }
        }
    }
    if let Some(path) = output {
        use std::io::Write;
        // Generated benchmark data, never edits source files. No partial run
        // creates a passing receipt; create_new also protects against races.
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .expect("create fresh corpus receipt");
        serde_json::to_writer_pretty(&mut file, &receipts).expect("write corpus receipt");
        writeln!(file).unwrap();
    }
}
