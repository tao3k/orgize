use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum EvalBodyTransport {
    Argument,
    Stdin,
    TypstMarkupEvalArgument,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EvalCommandBinding {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    pub(crate) body_transport: EvalBodyTransport,
}

impl EvalCommandBinding {
    pub(crate) fn body_mode_label(&self) -> &'static str {
        match self.body_transport {
            EvalBodyTransport::Argument => "argument",
            EvalBodyTransport::Stdin => "stdin",
            EvalBodyTransport::TypstMarkupEvalArgument => "typst-markup-eval-argument",
        }
    }

    pub(crate) fn uses_stdin(&self) -> bool {
        self.body_transport == EvalBodyTransport::Stdin
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeCommandOutput {
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) exit_code: Option<i32>,
}

/// Process-level outcome captured without exposing runtime output bodies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RuntimeTerminationOutcome {
    Exited,
    TimedOut,
    OutputBudgetExceeded,
    SpawnFailed,
    IoFailed,
}

/// Bounded process observation used to construct dependency-facing receipts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeExecutionObservation {
    pub(crate) output: RuntimeCommandOutput,
    pub(crate) stdout_bytes: usize,
    pub(crate) stderr_bytes: usize,
    pub(crate) elapsed: Duration,
    pub(crate) termination_outcome: RuntimeTerminationOutcome,
    pub(crate) child_reaped: bool,
    pub(crate) diagnostic: Option<String>,
}

pub(crate) fn resolve_eval_binding(
    language: &str,
    runtime: &str,
    shell_override: Option<&str>,
) -> Result<EvalCommandBinding, String> {
    match runtime.to_ascii_lowercase().as_str() {
        "bash" if is_shell_language(language) => {
            Ok(shell_binding(shell_override.unwrap_or("bash"), "-lc"))
        }
        "sh" | "shell" | "shell-script" if is_shell_language(language) => {
            Ok(shell_binding(shell_override.unwrap_or("sh"), "-c"))
        }
        "typst" if language == "typst" => {
            if shell_override.is_some() {
                return Err("--shell is only valid for shell source blocks".to_string());
            }
            Ok(EvalCommandBinding {
                program: "typst".to_string(),
                args: vec!["eval".to_string()],
                body_transport: EvalBodyTransport::TypstMarkupEvalArgument,
            })
        }
        known if matches!(known, "bash" | "sh" | "shell" | "shell-script" | "typst") => {
            Err(format!(
                "registered runtime `{runtime}` is incompatible with source block language `{language}`"
            ))
        }
        _ => Err(format!(
            "source block language `{language}` has no registered eval runtime `{runtime}`"
        )),
    }
}

pub(crate) fn execute_runtime(
    binding: &EvalCommandBinding,
    body: &str,
    current_dir: Option<&Path>,
    timeout: Duration,
    output_byte_budget: usize,
) -> Result<RuntimeCommandOutput, String> {
    let observation =
        execute_runtime_observed(binding, body, current_dir, timeout, output_byte_budget);
    match observation.termination_outcome {
        RuntimeTerminationOutcome::Exited => Ok(observation.output),
        RuntimeTerminationOutcome::TimedOut
        | RuntimeTerminationOutcome::OutputBudgetExceeded
        | RuntimeTerminationOutcome::SpawnFailed
        | RuntimeTerminationOutcome::IoFailed => Err(observation
            .diagnostic
            .unwrap_or_else(|| "registered runtime execution failed".to_string())),
    }
}

/// Executes one exact binding and retains only bounded observation metadata and
/// diagnostic text needed by the compatibility lint surface.
pub(crate) fn execute_runtime_observed(
    binding: &EvalCommandBinding,
    body: &str,
    current_dir: Option<&Path>,
    timeout: Duration,
    output_byte_budget: usize,
) -> RuntimeExecutionObservation {
    let started = Instant::now();
    let command = runtime_command(binding, body, current_dir);
    let mut child = match spawn_runtime(command, &binding.program) {
        Ok(child) => child,
        Err(error) => {
            return failed_observation(
                started,
                RuntimeTerminationOutcome::SpawnFailed,
                error,
                false,
            );
        }
    };
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            let child_reaped = terminate_and_reap(&mut child);
            return failed_observation(
                started,
                RuntimeTerminationOutcome::IoFailed,
                "registered runtime did not open stdout".to_string(),
                child_reaped,
            );
        }
    };
    let stderr = match child.stderr.take() {
        Some(stderr) => stderr,
        None => {
            let child_reaped = terminate_and_reap(&mut child);
            return failed_observation(
                started,
                RuntimeTerminationOutcome::IoFailed,
                "registered runtime did not open stderr".to_string(),
                child_reaped,
            );
        }
    };
    let output_budget = Arc::new(AtomicUsize::new(0));
    let output_overflowed = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stdout_budget = Arc::clone(&output_budget);
    let stderr_budget = Arc::clone(&output_budget);
    let stdout_overflowed = Arc::clone(&output_overflowed);
    let stderr_overflowed = Arc::clone(&output_overflowed);
    let stdout_reader = thread::spawn(move || {
        read_stream(stdout, stdout_budget, stdout_overflowed, output_byte_budget)
    });
    let stderr_reader = thread::spawn(move || {
        read_stream(stderr, stderr_budget, stderr_overflowed, output_byte_budget)
    });
    let stdin_writer = if binding.uses_stdin() {
        let mut stdin = match child.stdin.take() {
            Some(stdin) => stdin,
            None => {
                let child_reaped = terminate_and_reap(&mut child);
                let stdout = join_stream(stdout_reader, "stdout");
                let stderr = join_stream(stderr_reader, "stderr");
                return observation_from_streams(
                    started,
                    stdout,
                    stderr,
                    RuntimeTerminationOutcome::IoFailed,
                    None,
                    child_reaped,
                    Some("registered runtime did not open stdin".to_string()),
                );
            }
        };
        let body = body.as_bytes().to_vec();
        Some(thread::spawn(move || {
            stdin.write_all(&body).map_err(|error| error.to_string())
        }))
    } else {
        None
    };

    let deadline = Instant::now() + timeout;
    let status = loop {
        if output_overflowed.load(Ordering::Acquire) {
            let child_reaped = terminate_and_reap(&mut child);
            let _ = join_stdin(stdin_writer);
            let stdout = join_stream(stdout_reader, "stdout");
            let stderr = join_stream(stderr_reader, "stderr");
            return observation_from_streams(
                started,
                stdout,
                stderr,
                RuntimeTerminationOutcome::OutputBudgetExceeded,
                None,
                child_reaped,
                Some(format!(
                    "combined runtime output exceeded {output_byte_budget} bytes"
                )),
            );
        }
        let polled = match child.try_wait() {
            Ok(polled) => polled,
            Err(error) => {
                let child_reaped = terminate_and_reap(&mut child);
                let _ = join_stdin(stdin_writer);
                let stdout = join_stream(stdout_reader, "stdout");
                let stderr = join_stream(stderr_reader, "stderr");
                return observation_from_streams(
                    started,
                    stdout,
                    stderr,
                    RuntimeTerminationOutcome::IoFailed,
                    None,
                    child_reaped,
                    Some(format!(
                        "failed to poll registered runtime `{}`: {error}",
                        binding.program
                    )),
                );
            }
        };
        if let Some(status) = polled {
            break status;
        }
        if Instant::now() >= deadline {
            let child_reaped = terminate_and_reap(&mut child);
            let _ = join_stdin(stdin_writer);
            let stdout = join_stream(stdout_reader, "stdout");
            let stderr = join_stream(stderr_reader, "stderr");
            return observation_from_streams(
                started,
                stdout,
                stderr,
                RuntimeTerminationOutcome::TimedOut,
                None,
                child_reaped,
                Some(format!(
                    "registered runtime `{}` timed out after {} ms",
                    binding.program,
                    timeout.as_millis()
                )),
            );
        }
        thread::sleep(Duration::from_millis(10));
    };

    let stdin_result = join_stdin(stdin_writer);
    let stdout = join_stream(stdout_reader, "stdout");
    let stderr = join_stream(stderr_reader, "stderr");
    if output_overflowed.load(Ordering::Acquire) {
        return observation_from_streams(
            started,
            stdout,
            stderr,
            RuntimeTerminationOutcome::OutputBudgetExceeded,
            status.code(),
            true,
            Some(format!(
                "combined runtime output exceeded {output_byte_budget} bytes"
            )),
        );
    }
    if let Err(error) = stdin_result {
        return observation_from_streams(
            started,
            stdout,
            stderr,
            RuntimeTerminationOutcome::IoFailed,
            status.code(),
            true,
            Some(error),
        );
    }
    observation_from_streams(
        started,
        stdout,
        stderr,
        RuntimeTerminationOutcome::Exited,
        status.code(),
        true,
        None,
    )
}

fn runtime_command(
    binding: &EvalCommandBinding,
    body: &str,
    current_dir: Option<&Path>,
) -> Command {
    let mut command = Command::new(&binding.program);
    command
        .args(&binding.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    match binding.body_transport {
        EvalBodyTransport::Argument => {
            command.arg(body);
        }
        EvalBodyTransport::Stdin => {
            command.stdin(Stdio::piped());
        }
        EvalBodyTransport::TypstMarkupEvalArgument => {
            let body =
                serde_json::to_string(body).expect("serializing a Rust string as JSON cannot fail");
            command.arg(format!("eval({body}, mode: \"markup\")"));
        }
    }
    if let Some(current_dir) = current_dir
        && !current_dir.as_os_str().is_empty()
    {
        command.current_dir(current_dir);
    }
    command
}

fn spawn_runtime(mut command: Command, program: &str) -> Result<Child, String> {
    command
        .spawn()
        .map_err(|error| format!("failed to run registered runtime `{program}`: {error}"))
}

fn read_stream(
    mut stream: impl Read,
    output_bytes: Arc<AtomicUsize>,
    overflowed: Arc<std::sync::atomic::AtomicBool>,
    output_byte_budget: usize,
) -> Result<StreamRead, String> {
    let mut bytes = Vec::new();
    let mut observed_bytes = 0_usize;
    let mut chunk = [0_u8; 8_192];
    loop {
        if overflowed.load(Ordering::Acquire) {
            return Ok(StreamRead {
                bytes,
                observed_bytes,
            });
        }
        let count = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if count == 0 {
            return Ok(StreamRead {
                bytes,
                observed_bytes,
            });
        }
        observed_bytes = observed_bytes.saturating_add(count);
        let previous = output_bytes.fetch_add(count, Ordering::AcqRel);
        if previous.saturating_add(count) > output_byte_budget {
            overflowed.store(true, Ordering::Release);
            return Ok(StreamRead {
                bytes,
                observed_bytes,
            });
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
}

fn join_stream(
    reader: thread::JoinHandle<Result<StreamRead, String>>,
    name: &str,
) -> Result<StreamRead, String> {
    reader
        .join()
        .map_err(|_| format!("registered runtime {name} reader panicked"))?
        .map_err(|error| format!("failed to read registered runtime {name}: {error}"))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct StreamRead {
    bytes: Vec<u8>,
    observed_bytes: usize,
}

fn join_stdin(writer: Option<thread::JoinHandle<Result<(), String>>>) -> Result<(), String> {
    let Some(writer) = writer else {
        return Ok(());
    };
    writer
        .join()
        .map_err(|_| "registered runtime stdin writer panicked".to_string())?
        .map_err(|error| format!("failed to write registered runtime stdin: {error}"))
}

fn terminate_and_reap(child: &mut Child) -> bool {
    let _ = child.kill();
    child.wait().is_ok()
}

fn failed_observation(
    started: Instant,
    termination_outcome: RuntimeTerminationOutcome,
    diagnostic: String,
    child_reaped: bool,
) -> RuntimeExecutionObservation {
    RuntimeExecutionObservation {
        output: RuntimeCommandOutput {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
        },
        stdout_bytes: 0,
        stderr_bytes: 0,
        elapsed: started.elapsed(),
        termination_outcome,
        child_reaped,
        diagnostic: Some(diagnostic),
    }
}

fn observation_from_streams(
    started: Instant,
    stdout: Result<StreamRead, String>,
    stderr: Result<StreamRead, String>,
    termination_outcome: RuntimeTerminationOutcome,
    exit_code: Option<i32>,
    child_reaped: bool,
    diagnostic: Option<String>,
) -> RuntimeExecutionObservation {
    let (stdout, stdout_error) = match stdout {
        Ok(stdout) => (stdout, None),
        Err(error) => (
            StreamRead::default(),
            Some(format!("failed to read registered runtime stdout: {error}")),
        ),
    };
    let (stderr, stderr_error) = match stderr {
        Ok(stderr) => (stderr, None),
        Err(error) => (
            StreamRead::default(),
            Some(format!("failed to read registered runtime stderr: {error}")),
        ),
    };
    let stream_error = stdout_error.or(stderr_error);
    RuntimeExecutionObservation {
        output: RuntimeCommandOutput {
            stdout: String::from_utf8_lossy(&stdout.bytes).to_string(),
            stderr: String::from_utf8_lossy(&stderr.bytes).to_string(),
            exit_code,
        },
        stdout_bytes: stdout.observed_bytes,
        stderr_bytes: stderr.observed_bytes,
        elapsed: started.elapsed(),
        termination_outcome: if stream_error.is_some() {
            RuntimeTerminationOutcome::IoFailed
        } else {
            termination_outcome
        },
        child_reaped,
        diagnostic: stream_error.or(diagnostic),
    }
}

fn shell_binding(program: &str, eval_arg: &str) -> EvalCommandBinding {
    EvalCommandBinding {
        program: program.to_string(),
        args: vec![eval_arg.to_string()],
        body_transport: EvalBodyTransport::Argument,
    }
}

fn is_shell_language(language: &str) -> bool {
    matches!(language, "bash" | "sh" | "shell" | "shell-script")
}
