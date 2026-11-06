use std::process::ExitCode;

fn main() -> ExitCode {
    // SAFETY: CLI startup precedes application workers and child processes.
    if let Err(error) = unsafe { orgize::initialize_native_runtime() } {
        eprintln!("native startup failed: {error}");
        return ExitCode::FAILURE;
    }
    orgize::cli::run_from_env()
}
