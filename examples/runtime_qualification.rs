//! Complete corpus/lifecycle qualification. Criterion owns performance sampling.
#[path = "../tests/support/runtime_config.rs"]
mod runtime_config;
#[path = "../tests/support/runtime_corpus.rs"]
mod runtime_corpus;
#[path = "../tests/support/runtime_parallel.rs"]
mod runtime_parallel;

const INPUTS: &[(&str, &str)] = &[
    ("doc.org", include_str!("../benches/fixtures/doc.org")),
    (
        "plain-links.org",
        include_str!("../benches/fixtures/plain-links.org"),
    ),
    (
        "quote-heavy.org",
        include_str!("../benches/fixtures/quote-heavy.org"),
    ),
];

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if let [worker, mode] = args.as_slice() {
        if worker == "--native-corpus-worker" {
            assert!(matches!(mode.as_str(), "qualification" | "performance"));
            runtime_parallel::worker(mode == "performance");
            return;
        }
    }
    let config = runtime_config::Configuration::parse(&args).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(2);
    });
    if matches!(config.mode.as_str(), "qualification" | "performance") {
        // Supervisor never initializes or forks an initialized runtime.
        runtime_parallel::run(&config);
    } else {
        // SAFETY: native startup precedes application threads and work.
        unsafe { orgize::initialize_native_runtime() }.expect("qualification startup");
        runtime_corpus::run(&config);
    }
}
