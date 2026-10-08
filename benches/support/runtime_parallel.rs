//! High-concurrency qualification, not a production parser/process fallback.
//! Tokio admissions -> bounded worker queues -> independent exec'd AOT domains.
use super::runtime_corpus::{self, Sample};
use orgize::{org_aot::org_event_parser_digest, runtime_backend};
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

pub(super) fn worker() {
    // SAFETY: fresh exec; no application threads or children before startup.
    unsafe { orgize::initialize_native_runtime() }.expect("isolated worker startup");
    for &(_, source) in super::INPUTS {
        runtime_corpus::parse_sample(source, 0, Instant::now()).unwrap();
    }
    let mut output = std::io::stdout().lock();
    writeln!(
        output,
        "{}",
        serde_json::json!({
            "ready": true, "pid": std::process::id(),
            "backend": runtime_backend().name(), "program": org_event_parser_digest(),
            "profile_enabled": cfg!(feature = "runtime-profile"),
        })
    )
    .unwrap();
    output.flush().unwrap();
    for line in std::io::stdin().lock().lines() {
        let request: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let index = request["index"].as_u64().unwrap() as usize;
        let sample = runtime_corpus::parse_sample(
            request["source"].as_str().unwrap(),
            index,
            Instant::now(),
        )
        .unwrap();
        writeln!(
            output,
            "{}",
            serde_json::json!({
                "index": sample.index, "parse_ns": sample.elapsed_ns,
                "artifact": sample.artifact.map(|value| value.to_hex().to_string()),
                "stages_ns": sample.stages,
            })
        )
        .unwrap();
        output.flush().unwrap();
    }
}

// Parent owns child handles, independent of potentially blocked pipe threads.
// Panic/watchdog failure kills and reaps exact children before unwinding.
struct Children(Vec<Child>);
impl Drop for Children {
    fn drop(&mut self) {
        for child in &mut self.0 {
            let _ = child.kill();
        }
        for child in &mut self.0 {
            let _ = child.wait();
        }
    }
}

struct Request {
    index: usize,
    source: Arc<Vec<String>>,
    submitted: Instant,
    response: tokio::sync::oneshot::Sender<Result<Sample, String>>,
}

fn latency(values: &mut [u64]) -> serde_json::Value {
    values.sort_unstable();
    serde_json::json!({"p50": runtime_corpus::percentile(values, 50),
        "p95": runtime_corpus::percentile(values, 95),
        "p99": runtime_corpus::percentile(values, 99), "max": values.last()})
}

fn batch(
    documents: Arc<Vec<String>>,
    callers: usize,
    domains: usize,
    repetition: usize,
) -> serde_json::Value {
    assert!(domains <= callers, "every domain needs an admitted caller");
    let mut children = Children(Vec::new());
    let mut workers = Vec::new();
    let mut queues = Vec::new();
    let mut pids = Vec::new();
    let active = Arc::new(AtomicUsize::new(0));
    let max_active = Arc::new(AtomicUsize::new(0));
    let (ready_tx, ready_rx) = mpsc::channel();
    let queue_capacity = callers.div_ceil(domains);
    for _ in 0..domains {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .arg("--native-corpus-worker")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("fresh worker exec");
        pids.push(child.id());
        let mut input = child.stdin.take().unwrap();
        let mut output = BufReader::new(child.stdout.take().unwrap());
        children.0.push(child);
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Request>(queue_capacity);
        queues.push(tx);
        let ready = ready_tx.clone();
        let active = Arc::clone(&active);
        let max_active = Arc::clone(&max_active);
        workers.push(thread::spawn(move || -> Result<usize, String> {
            let mut line = String::new();
            output.read_line(&mut line).map_err(|e| e.to_string())?;
            let handshake: serde_json::Value =
                serde_json::from_str(&line).map_err(|e| e.to_string())?;
            if handshake["ready"] != true
                || handshake["backend"] != runtime_backend().name()
                || handshake["program"] != org_event_parser_digest()
                || handshake["profile_enabled"] != cfg!(feature = "runtime-profile")
            {
                return Err("worker identity mismatch".into());
            }
            ready.send(()).map_err(|e| e.to_string())?;
            let mut count = 0;
            while let Some(request) = rx.blocking_recv() {
                let now_active = active.fetch_add(1, Ordering::SeqCst) + 1;
                max_active.fetch_max(now_active, Ordering::SeqCst);
                let queue_ns = request.submitted.elapsed().as_nanos() as u64;
                let result = (|| {
                    serde_json::to_writer(
                        &mut input,
                        &serde_json::json!({"index": request.index,
                        "source": request.source[request.index]}),
                    )
                    .map_err(|e| e.to_string())?;
                    writeln!(input)
                        .and_then(|_| input.flush())
                        .map_err(|e| e.to_string())?;
                    line.clear();
                    output.read_line(&mut line).map_err(|e| e.to_string())?;
                    let response: serde_json::Value =
                        serde_json::from_str(&line).map_err(|e| e.to_string())?;
                    if response["index"].as_u64() != Some(request.index as u64) {
                        return Err("worker completion identity mismatch".into());
                    }
                    let submitted_ns = request.submitted.elapsed().as_nanos() as u64;
                    let mut stages = response["stages_ns"].clone();
                    if !stages.is_object() {
                        return Err("worker stage timings missing".into());
                    }
                    stages["supervisor.queue"] = serde_json::json!(queue_ns);
                    stages["supervisor.ipc_service_inclusive"] =
                        serde_json::json!(submitted_ns - queue_ns);
                    Ok(Sample {
                        index: request.index,
                        elapsed_ns: response["parse_ns"]
                            .as_u64()
                            .ok_or("missing parse timing")?,
                        submitted_to_parse_ns: submitted_ns,
                        artifact: response["artifact"]
                            .as_str()
                            .map(blake3::Hash::from_hex)
                            .transpose()
                            .map_err(|e| e.to_string())?,
                        stages,
                    })
                })();
                active.fetch_sub(1, Ordering::SeqCst);
                let passed = result.is_ok();
                let _ = request.response.send(result);
                if !passed {
                    return Err("worker protocol/parse failed".into());
                }
                count += 1;
            }
            Ok(count)
        }));
    }
    drop(ready_tx);
    for _ in 0..domains {
        ready_rx
            .recv_timeout(Duration::from_secs(30))
            .expect("worker startup handshake deadline");
    }
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_time()
        .build()
        .unwrap();
    let next = Arc::new(AtomicUsize::new(0));
    let in_flight = Arc::new(AtomicUsize::new(0));
    let max_in_flight = Arc::new(AtomicUsize::new(0));
    let (completed_tx, completed_rx) = mpsc::channel();
    let gate = Arc::new(tokio::sync::Barrier::new(callers + 1));
    let mut tasks = Vec::new();
    for caller in 0..callers {
        let queue = queues[caller % domains].clone();
        let documents = Arc::clone(&documents);
        let next = Arc::clone(&next);
        let in_flight = Arc::clone(&in_flight);
        let max_in_flight = Arc::clone(&max_in_flight);
        let completed = completed_tx.clone();
        let gate = Arc::clone(&gate);
        tasks.push(runtime.spawn(async move {
            gate.wait().await;
            loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                if index >= documents.len() {
                    break;
                }
                let current = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
                max_in_flight.fetch_max(current, Ordering::SeqCst);
                let submitted = Instant::now();
                let (response, incoming) = tokio::sync::oneshot::channel();
                queue
                    .send(Request {
                        index,
                        source: Arc::clone(&documents),
                        submitted,
                        response,
                    })
                    .await
                    .expect("worker queue closed");
                let sample = incoming.await.expect("worker response dropped");
                in_flight.fetch_sub(1, Ordering::SeqCst);
                completed.send(sample).expect("collector closed");
            }
        }));
    }
    drop(completed_tx);
    let started = Instant::now();
    runtime.block_on(gate.wait());
    let mut samples: Vec<Option<Sample>> = (0..documents.len()).map(|_| None).collect();
    let mut last_completed = started;
    let mut last_log = started;
    for count in 1..=documents.len() {
        let sample = loop {
            match completed_rx.recv_timeout(Duration::from_secs(1)) {
                Ok(result) => break result.expect("worker parse/protocol failure"),
                Err(mpsc::RecvTimeoutError::Timeout) => assert!(
                    last_completed.elapsed() < Duration::from_secs(5),
                    "five seconds without completion"
                ),
                Err(error) => panic!("incomplete caller results: {error}"),
            }
        };
        let index = sample.index;
        assert!(
            samples[index].replace(sample).is_none(),
            "duplicate completion"
        );
        last_completed = Instant::now();
        if last_log.elapsed() >= Duration::from_secs(1) || count == documents.len() {
            eprintln!(
                "org-runtime-parallel backend={} docs={} callers={} domains={} complete={count}/{} in_flight={} active_worker_requests={} elapsed_s={:.2}",
                runtime_backend().name(),
                documents.len(),
                callers,
                domains,
                documents.len(),
                in_flight.load(Ordering::SeqCst),
                active.load(Ordering::SeqCst),
                started.elapsed().as_secs_f64()
            );
            last_log = Instant::now();
        }
    }
    let elapsed = started.elapsed();
    runtime.block_on(async {
        for task in tasks {
            task.await.expect("Tokio caller failed");
        }
    });
    drop(queues);
    let worker_counts: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().unwrap().unwrap())
        .collect();
    // EOF shuts down workers. Exact exit statuses remain part of the gate.
    for child in &mut children.0 {
        let deadline = Instant::now();
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success(), "worker exit failure");
                break;
            }
            assert!(
                deadline.elapsed() < Duration::from_secs(5),
                "worker shutdown deadline"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
    children.0.clear();
    let mut artifact = blake3::Hasher::new();
    artifact.update(b"orgize.runtime-corpus-artifact.v1\0");
    let mut stage_samples = std::collections::BTreeMap::<String, Vec<u64>>::new();
    let performance = std::env::var("ORGIZE_RUNTIME_CORPUS_MODE").as_deref() == Ok("performance");
    let mut parse_latencies = Vec::new();
    let mut end_to_end = Vec::new();
    for sample in samples {
        let sample = sample.unwrap();
        artifact.update(&(sample.index as u64).to_le_bytes());
        if let Some(checksum) = sample.artifact {
            assert!(!performance, "performance lane must not run full checksums");
            artifact.update(checksum.as_bytes());
        } else {
            assert!(performance, "qualification checksum missing");
        }
        for (name, value) in sample.stages.as_object().expect("stage timings required") {
            stage_samples
                .entry(name.clone())
                .or_default()
                .push(value.as_u64().unwrap());
        }
        parse_latencies.push(sample.elapsed_ns);
        end_to_end.push(sample.submitted_to_parse_ns);
    }
    let stage_timings: serde_json::Map<String, serde_json::Value> = stage_samples
        .into_iter()
        .map(|(name, mut values)| {
            assert_eq!(
                values.len(),
                documents.len(),
                "every request must report each stage"
            );
            let total: u128 = values.iter().map(|&v| u128::from(v)).sum();
            let mut summary = latency(&mut values);
            summary["total"] = serde_json::json!(total);
            summary["mean"] = serde_json::json!(total as f64 / values.len() as f64);
            (name, summary)
        })
        .collect();
    serde_json::json!({
        "schema": "orgize.runtime-parallel.v2", "backend": runtime_backend().name(),
        "mode": if performance { "performance" } else { "qualification" },
        "profile_enabled": cfg!(feature = "runtime-profile"),
        "stage_timings_ns": stage_timings,
        "native_stage_scope": "transport includes native owner admission, Scheme parse, event tape serialization and result transfer; not isolated Scheme CPU time",
        "driver": "tokio-process-domains", "tokio_workers": 4,
        "program": org_event_parser_digest(), "adapter": runtime_corpus::adapter_identity(),
        "benchmark": blake3::hash(concat!(include_str!("runtime_parallel.rs"), include_str!("runtime_corpus.rs"), include_str!("../org_runtime.rs")).as_bytes()).to_hex().to_string(),
        "architecture": std::env::consts::ARCH, "corpus": runtime_corpus::source_identity(&documents),
        "artifact": if performance { None } else { Some(artifact.finalize().to_hex().to_string()) },
        "documents": documents.len(), "distinct_documents": documents.iter().collect::<std::collections::HashSet<_>>().len(),
        "completed": parse_latencies.len(), "bytes": documents.iter().map(String::len).sum::<usize>(),
        "callers": callers, "domains": domains, "repetition": repetition,
        "worker_pids": pids, "worker_completed": worker_counts, "worker_exit_success": true,
        "max_in_flight": max_in_flight.load(Ordering::SeqCst),
        "max_active_worker_requests": max_active.load(Ordering::SeqCst),
        "active_scope": "overlapping supervisor IPC requests to independent native domains; not sampled CPU utilization",
        "queue_capacity_per_domain": queue_capacity, "native_queue_capacity_per_domain": 64,
        "elapsed_ns": elapsed.as_nanos(), "documents_per_second": documents.len() as f64 / elapsed.as_secs_f64(),
        "parse_latency_ns": latency(&mut parse_latencies), "end_to_end_latency_ns": latency(&mut end_to_end),
        "end_to_end_scope": if performance { "Tokio submission through bounded queue, IPC, native parse/projection, drop and response; excludes full semantic validation/checksum and startup/warmup" } else { "Tokio submission through bounded queue, IPC, native parse/projection, lossless validation, checksum, drop and response; excludes startup/warmup" },
        "workload": "closed-loop bounded concurrent admissions; not an open-loop arrival-rate test",
    })
}

pub(super) fn run() {
    let mode =
        std::env::var("ORGIZE_RUNTIME_CORPUS_MODE").unwrap_or_else(|_| "qualification".into());
    assert!(matches!(mode.as_str(), "qualification" | "performance"));
    let sizes = runtime_corpus::configuration("ORGIZE_RUNTIME_CORPUS_DOCS", "1000,10000", 10000);
    let callers = runtime_corpus::configuration("ORGIZE_RUNTIME_CORPUS_CALLERS", "64", 1024);
    let domains = runtime_corpus::configuration("ORGIZE_RUNTIME_CORPUS_DOMAINS", "1,4,8", 64);
    let repeats = runtime_corpus::configuration("ORGIZE_RUNTIME_CORPUS_REPEATS", "1", 10);
    assert_eq!(repeats.len(), 1);
    let output = std::env::var_os("ORGIZE_RUNTIME_CORPUS_OUTPUT").expect("receipt output required");
    assert!(
        !std::path::Path::new(&output).exists(),
        "refuse to overwrite receipts"
    );
    let mut receipts = Vec::new();
    for count in sizes {
        let documents = Arc::new(runtime_corpus::corpus(count));
        let mut reference = None;
        for &callers in &callers {
            for &domains in &domains {
                for repetition in 1..=repeats[0] {
                    let receipt = batch(Arc::clone(&documents), callers, domains, repetition);
                    if let Some(expected) = &reference {
                        assert_eq!(expected, &receipt["artifact"]);
                    } else {
                        reference = Some(receipt["artifact"].clone());
                    }
                    println!("{receipt}");
                    receipts.push(receipt);
                }
            }
        }
    }
    // Only a fully completed matrix publishes a receipt. Generated data only.
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .unwrap();
    serde_json::to_writer_pretty(&mut file, &receipts).unwrap();
    writeln!(file).unwrap();
}
