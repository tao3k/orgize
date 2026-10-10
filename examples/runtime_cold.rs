//! Fresh-process initialization measurement, never a production parser fallback.
use orgize::{Org, org_aot::org_event_parser_digest, runtime_backend};
use std::{
    io::Read,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const SOURCE: &str = include_str!("../benches/fixtures/doc.org");

fn main() {
    let argument = std::env::args().nth(1).unwrap_or_else(|| "10".into());
    if argument == "--sample" {
        let started = Instant::now();
        // SAFETY: fresh child, no application workers or live subprocesses.
        unsafe { orgize::initialize_native_runtime() }.expect("native startup");
        let document = Org::parse(SOURCE);
        let elapsed = started.elapsed().as_nanos();
        assert_eq!(document.to_org(), SOURCE);
        println!(
            "{}",
            serde_json::json!({
                "schema": "orgize.runtime-cold.v1", "backend": runtime_backend().name(),
                "program": org_event_parser_digest(), "fixture": "doc.org",
                "bytes": SOURCE.len(), "elapsed_ns": elapsed,
                "consumers": 1, "queue_capacity": 64,
            })
        );
        return;
    }
    let samples: usize = argument.parse().expect("positive sample count");
    assert!((1..=100).contains(&samples));
    // The supervisor never initializes Gerbil. Every child gets a genuinely
    // fresh process-global lifecycle; no unsupported runtime restart is used.
    for _ in 0..samples {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .arg("--sample")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(8);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("native cold sample exceeded eight seconds");
            }
            thread::sleep(Duration::from_millis(5));
        };
        assert!(status.success(), "cold sample failed: {status}");
        let mut receipt = String::new();
        child
            .stdout
            .take()
            .unwrap()
            .read_to_string(&mut receipt)
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(&receipt).unwrap();
        assert_eq!(value["backend"], runtime_backend().name());
        assert_eq!(value["program"], org_event_parser_digest());
        println!("{value}");
    }
}
