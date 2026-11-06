// Semantic rejection must not hide a broken native parse/ABI call.
pub fn assert_rejected(result: Result<String, String>) {
    let error = result.expect_err("invalid feature declaration must be rejected");
    assert!(!error.starts_with("invalid Org feature source:"), "{error}");
}

// Each example executable owns exclusive startup before its concurrent cases.
pub fn run(suite: &str, expected_cases: usize, cases: &[(&str, fn())]) {
    let startup_error = orgize::Org::try_parse("* before startup\n")
        .expect_err("parsing must reject an uninitialized host");
    assert!(format!("{startup_error:?}").contains("native runtime not initialized"));
    // SAFETY: This is the executable's only test entry; no workers exist yet.
    unsafe { orgize::initialize_native_runtime() }.expect("exclusive native startup");
    orgize::Org::try_parse("* initialized\n").expect("startup admits real parsing");

    let mut names = std::collections::HashSet::new();
    assert_eq!(
        cases.len(),
        expected_cases,
        "complete example assertion catalog"
    );
    assert!(expected_cases > 0);
    assert!(cases.iter().all(|(name, _)| names.insert(*name)));
    let panic_case = std::env::var("ORGIZE_NATIVE_EXAMPLE_CASE_PANIC").ok();
    let panic_case = panic_case.as_deref();
    assert!(panic_case.is_none_or(|name| names.contains(name)));
    let failed = std::thread::scope(|scope| {
        let workers: Vec<_> = cases
            .iter()
            .map(|&(name, case)| {
                scope.spawn(move || {
                    println!("native-example case={name} START");
                    let result = std::panic::catch_unwind(|| {
                        orgize::Org::try_parse("* worker readiness\n")
                            .expect("native parsing is available on each case worker");
                        assert_ne!(panic_case, Some(name), "deliberate case failure control");
                        case();
                    });
                    println!(
                        "native-example case={name} {}",
                        if result.is_ok() { "OK" } else { "FAILED" }
                    );
                    result.is_err()
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().expect("case worker"))
            .filter(|failed| *failed)
            .count()
    });
    assert_eq!(failed, 0, "{suite}: failed native example cases");
    println!(
        "startup-native suite={suite} concurrent-cases={} complete OK",
        cases.len()
    );
}
