//! Native embedding must not consume exit statuses owned by the Rust host.

#[cfg(unix)]
#[test]
fn native_parser_keeps_rust_host_child_wait_ownership() {
    use std::process::Command;

    let child = || {
        Command::new(env!("CARGO_BIN_EXE_orgize"))
            .arg("--version")
            .output()
    };
    let before = child().expect("host child wait works before native initialization");
    assert!(before.status.success());

    // SAFETY: prior child is reaped; application worker threads start below.
    unsafe { orgize::initialize_native_runtime() }.expect("explicit startup");

    assert_eq!(
        orgize::Org::parse("* Native owner\n").to_org(),
        "* Native owner\n"
    );

    let results = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..4)
            .map(|_| scope.spawn(|| (0..12).map(|_| child()).collect::<Vec<_>>()))
            .collect();
        workers
            .into_iter()
            .flat_map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>()
    });
    for (index, result) in results.into_iter().enumerate() {
        let output = result.unwrap_or_else(|error| {
            panic!("native initialization broke Rust child wait {index}: {error}");
        });
        assert!(
            output.status.success(),
            "host child {index} failed: {output:?}"
        );
    }
}
