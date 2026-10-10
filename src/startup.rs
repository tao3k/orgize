//! Explicit startup admission. The parser does not own host stdio or children.
use std::sync::OnceLock;

static STARTUP: OnceLock<Result<(), String>> = OnceLock::new();

/// Initialize the selected native owner once, before application concurrency.
/// Repeated calls only return the original result; runtime restart is forbidden.
///
/// # Safety
/// The host must provide an exclusive startup window: no live or newly spawned
/// subprocesses, no concurrent native initialization, and no application workers
/// using stdio or changing signal dispositions until this call returns. The
/// linked parser must not use Scheme-owned stdio, terminals or subprocesses:
/// host control signals and stdio are restored before requests are admitted.
/// Runtime heartbeat/processor signals remain native-owned.
pub unsafe fn initialize_native_runtime() -> Result<(), String> {
    STARTUP
        .get_or_init(|| {
            #[cfg(unix)]
            {
                let host = HostResources::capture()?;
                let initialized = crate::org_aot::initialize_native_owner();
                // Restore even when initialization fails; never admit on failure.
                let restored = host.restore();
                initialized.and(restored)
            }
            #[cfg(not(unix))]
            {
                Err("explicit host-resource startup is currently qualified on POSIX only".into())
            }
        })
        .clone()
}

pub(crate) fn require_initialized() -> Result<(), String> {
    STARTUP.get().ok_or_else(||
        "native runtime not initialized: call initialize_native_runtime during exclusive host startup".to_owned()
    )?.clone()
}

#[cfg(unix)]
struct HostResources {
    signals: Vec<(libc::c_int, libc::sigaction)>,
    descriptors: Vec<(libc::c_int, libc::c_int)>,
}

#[cfg(unix)]
impl HostResources {
    fn capture() -> Result<Self, String> {
        let mut signals = Vec::new();
        // Process-control/terminal/child signals belong to the embedding host.
        // Native timer and processor wakeup signals are deliberately excluded.
        for number in [
            libc::SIGINT,
            libc::SIGTERM,
            libc::SIGCHLD,
            libc::SIGPIPE,
            libc::SIGQUIT,
            libc::SIGWINCH,
            libc::SIGCONT,
        ] {
            let mut action = std::mem::MaybeUninit::uninit();
            // SAFETY: query-only POSIX call initializes output on success.
            if unsafe { libc::sigaction(number, std::ptr::null(), action.as_mut_ptr()) } != 0 {
                return Err(format!(
                    "capture host signal {number}: {}",
                    std::io::Error::last_os_error()
                ));
            }
            signals.push((number, unsafe { action.assume_init() }));
        }
        let mut descriptors = Vec::new();
        for fd in [libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO] {
            // SAFETY: query only. Startup requires valid host stdio descriptors.
            let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
            if flags < 0 {
                return Err(format!(
                    "capture host descriptor {fd}: {}",
                    std::io::Error::last_os_error()
                ));
            }
            descriptors.push((fd, flags));
        }
        Ok(Self {
            signals,
            descriptors,
        })
    }

    fn restore(self) -> Result<(), String> {
        let mut errors = Vec::new();
        for (fd, flags) in self.descriptors {
            // SAFETY: exclusive startup keeps the captured descriptor live.
            if unsafe { libc::fcntl(fd, libc::F_SETFL, flags) } < 0 {
                errors.push(format!(
                    "restore host descriptor {fd}: {}",
                    std::io::Error::last_os_error()
                ));
            }
        }
        for (number, action) in self.signals {
            // SAFETY: captured valid disposition; startup excludes host races.
            if unsafe { libc::sigaction(number, &action, std::ptr::null_mut()) } != 0 {
                errors.push(format!(
                    "restore host signal {number}: {}",
                    std::io::Error::last_os_error()
                ));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }
}
