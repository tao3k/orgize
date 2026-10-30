//! Isolated lifecycle gate; never a runtime/process parser fallback.
#[cfg(unix)]
static HOST_SIGNAL_SEEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[cfg(unix)]
extern "C" fn host_signal(_: libc::c_int) {
    HOST_SIGNAL_SEEN.store(true, std::sync::atomic::Ordering::Relaxed);
}
#[cfg(unix)]
fn main() {
    use std::{
        io::{BufRead, BufReader, Write},
        os::unix::process::ExitStatusExt,
        process::{Command, Stdio},
        sync::mpsc,
        time::{Duration, Instant},
    };
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--status-child") {
        std::process::exit(23);
    }
    if args.get(1).map(String::as_str) == Some("--child") {
        if args[2] == "native-custom" {
            let mut custom: libc::sigaction = unsafe { std::mem::zeroed() };
            custom.sa_sigaction = host_signal as *const () as usize;
            assert_eq!(unsafe { libc::sigemptyset(&mut custom.sa_mask) }, 0);
            assert_eq!(
                unsafe { libc::sigaction(libc::SIGTERM, &custom, std::ptr::null_mut()) },
                0
            );
        }
        let query_handler = || {
            let mut action = std::mem::MaybeUninit::<libc::sigaction>::uninit();
            assert_eq!(
                unsafe { libc::sigaction(libc::SIGTERM, std::ptr::null(), action.as_mut_ptr()) },
                0
            );
            unsafe { action.assume_init() }.sa_sigaction
        };
        let handler_before = query_handler();
        // Query the open-file flags; never restore or otherwise mutate them.
        let flags = || unsafe { libc::fcntl(libc::STDIN_FILENO, libc::F_GETFL) };
        let stdin_before = flags();
        assert!(stdin_before >= 0);
        if args[2] != "baseline" {
            assert!(orgize::Org::try_parse("* Before startup\n").is_err());
            // SAFETY: supervisor admits a fresh child before application work.
            unsafe { orgize::initialize_native_runtime() }.expect("explicit startup");
            let source = "* Lifecycle\nα [[id:gate][evidence]]\n";
            assert_eq!(orgize::Org::parse(source).to_org(), source);
        }
        let mut children_checked = 0;
        if args[2] == "native-host-children" {
            let mut children: Vec<_> = (0..12)
                .map(|_| {
                    Command::new(std::env::current_exe().unwrap())
                        .arg("--status-child")
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .spawn()
                        .unwrap()
                })
                .collect();
            std::thread::sleep(Duration::from_millis(100));
            for child in &mut children {
                assert_eq!(
                    child.wait().expect("host owns child wait status").code(),
                    Some(23)
                );
                children_checked += 1;
            }
        }
        let mut action = std::mem::MaybeUninit::<libc::sigaction>::uninit();
        // SAFETY: query only; sigaction initializes the supplied output.
        assert_eq!(
            unsafe { libc::sigaction(libc::SIGTERM, std::ptr::null(), action.as_mut_ptr()) },
            0
        );
        let handler = unsafe { action.assume_init() }.sa_sigaction;
        println!(
            "READY {}",
            serde_json::json!({
            "sigterm_handler": if handler == libc::SIG_DFL {
                "default"
            } else if handler == libc::SIG_IGN {
                "ignored"
            } else {
                "installed"
            },
            "stdin_flags_before": stdin_before, "stdin_flags_after": flags(),
            "host_stdio_preserved": stdin_before == flags(),
            "host_handler_preserved": handler_before == handler,
            "host_children_checked": children_checked,
            })
        );
        std::io::stdout().flush().unwrap();
        if args[2] == "native-active" {
            // Application workers start only after explicit native admission.
            std::thread::spawn(|| {
                loop {
                    let source = include_str!("../benches/fixtures/doc.org");
                    assert_eq!(orgize::Org::parse(source).to_org(), source);
                }
            });
        }
        let mut command = String::new();
        loop {
            match std::io::stdin().read_line(&mut command) {
                Ok(_) => break,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    // Probe protocol can tolerate nonblocking input, but the
                    // flag mutation remains an explicitly red host-stdio gate.
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(error) => panic!("probe input failed: {error}"),
            }
        }
        assert_eq!(command.trim(), "exit");
        if args[2] == "native-custom" && args.get(3).map(String::as_str) == Some("sigterm") {
            assert!(
                HOST_SIGNAL_SEEN.load(std::sync::atomic::Ordering::Relaxed),
                "captured custom host handler must run"
            );
        }
        return;
    }
    let mut passed = true;
    for phase in [
        "baseline",
        "native",
        "native-custom",
        "native-host-children",
        "native-active",
    ] {
        for termination in ["normal", "sigterm"] {
            if phase == "native-active" && termination == "normal" {
                continue;
            }
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args(["--child", phase, termination])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            let stdout = child.stdout.take().unwrap();
            let (send, receive) = mpsc::channel();
            let reader = std::thread::spawn(move || {
                for line in BufReader::new(stdout).lines() {
                    if send.send(line).is_err() {
                        break;
                    }
                }
            });
            let ready = receive.recv_timeout(Duration::from_secs(5));
            let metadata = ready
                .as_ref()
                .ok()
                .and_then(|line| line.as_ref().ok())
                .and_then(|line| line.strip_prefix("READY "))
                .and_then(|json| serde_json::from_str::<serde_json::Value>(json).ok());
            let admitted = metadata.is_some();
            if admitted {
                if termination == "normal" {
                    child.stdin.as_mut().unwrap().write_all(b"exit\n").unwrap();
                } else {
                    if phase == "native-active" {
                        std::thread::sleep(Duration::from_millis(20));
                    }
                    // SAFETY: signal only this supervised, still-live child PID.
                    assert_eq!(
                        unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGTERM) },
                        0
                    );
                    if phase == "native-custom" {
                        std::thread::sleep(Duration::from_millis(20));
                        child.stdin.as_mut().unwrap().write_all(b"exit\n").unwrap();
                    }
                }
            } else {
                // The child may already have failed before READY; still reap
                // it and publish a red receipt instead of panicking here.
                let _ = child.kill();
            }
            let started = Instant::now();
            let mut timed_out = false;
            let status = loop {
                if let Some(status) = child.try_wait().unwrap() {
                    break status;
                }
                if started.elapsed() >= Duration::from_secs(2) {
                    timed_out = true;
                    child.kill().unwrap();
                    break child.wait().unwrap();
                }
                std::thread::sleep(Duration::from_millis(5));
            };
            reader.join().unwrap();
            let termination_passed = admitted
                && !timed_out
                && if termination == "normal" {
                    status.success()
                } else {
                    status.signal() == Some(libc::SIGTERM) || status.success()
                };
            let stdio_passed = metadata
                .as_ref()
                .is_some_and(|m| m["host_stdio_preserved"] == true);
            let handler_passed = metadata
                .as_ref()
                .is_some_and(|m| m["host_handler_preserved"] == true);
            let ok = termination_passed && stdio_passed && handler_passed;
            passed &= ok;
            println!(
                "{}",
                serde_json::json!({
                    "schema": "orgize.runtime-lifecycle.v1", "backend": orgize::runtime_backend().name(),
                    "program": orgize::org_aot::org_event_parser_digest(),
                    "phase": phase, "termination": termination, "ready": ready.ok().and_then(Result::ok),
                    "metadata": metadata, "termination_passed": termination_passed, "host_stdio_preserved": stdio_passed,
                    "exit_code": status.code(), "signal": status.signal(), "timed_out": timed_out, "passed": ok,
                    "scope": "exclusive startup restores host resources; probe queries flags/dispositions and tests host-owned termination/children",
                })
            );
        }
    }
    if !passed {
        std::process::exit(1);
    }
}

#[cfg(not(unix))]
fn main() {
    panic!("POSIX lifecycle qualification requires Unix");
}
