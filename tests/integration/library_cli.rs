use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Output};

const REQUEST_ENV: &str = "ORGIZE_LIBRARY_CLI_REQUEST";
const ARG_PREFIX: &str = "ORGIZE_LIBRARY_CLI_ARG_";
const STDOUT_MARKER: &str = "[orgize-library-cli-output]\n";

#[derive(Default)]
pub(crate) struct OrgizeLibraryCliCommand {
    args: Vec<OsString>,
    current_dir: Option<PathBuf>,
    env: Vec<(OsString, OsString)>,
    stdin: Option<std::process::Stdio>,
    stdout: Option<std::process::Stdio>,
    stderr: Option<std::process::Stdio>,
}

pub(crate) struct OrgizeLibraryCliChild {
    pub(crate) stdin: Option<std::process::ChildStdin>,
    child: std::process::Child,
}

pub(crate) fn orgize_cli_command() -> OrgizeLibraryCliCommand {
    OrgizeLibraryCliCommand::default()
}

impl OrgizeLibraryCliCommand {
    pub(crate) fn current_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.current_dir = Some(path.as_ref().to_path_buf());
        self
    }

    pub(crate) fn env(
        &mut self,
        key: impl Into<OsString>,
        value: impl Into<OsString>,
    ) -> &mut Self {
        self.env.push((key.into(), value.into()));
        self
    }

    pub(crate) fn arg(&mut self, arg: impl Into<OsString>) -> &mut Self {
        self.args.push(arg.into());
        self
    }

    pub(crate) fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub(crate) fn stdin(&mut self, stdin: std::process::Stdio) -> &mut Self {
        self.stdin = Some(stdin);
        self
    }

    pub(crate) fn stdout(&mut self, stdout: std::process::Stdio) -> &mut Self {
        self.stdout = Some(stdout);
        self
    }

    pub(crate) fn stderr(&mut self, stderr: std::process::Stdio) -> &mut Self {
        self.stderr = Some(stderr);
        self
    }

    fn command(&mut self) -> std::io::Result<Command> {
        let mut command = Command::new(std::env::current_exe()?);
        command.args([
            "--exact",
            "library_cli::orgize_cli_library_subprocess",
            "--ignored",
            "--nocapture",
        ]);
        command.env(REQUEST_ENV, self.args.len().to_string());
        for (index, arg) in self.args.iter().enumerate() {
            command.env(format!("{ARG_PREFIX}{index}"), arg);
        }
        if let Some(current_dir) = &self.current_dir {
            command.current_dir(current_dir);
        }
        for (key, value) in &self.env {
            command.env(key, value);
        }
        if let Some(stdin) = self.stdin.take() {
            command.stdin(stdin);
        }
        if let Some(stdout) = self.stdout.take() {
            command.stdout(stdout);
        }
        if let Some(stderr) = self.stderr.take() {
            command.stderr(stderr);
        }
        Ok(command)
    }

    pub(crate) fn output(&mut self) -> std::io::Result<Output> {
        let mut output = self.command()?.output()?;
        strip_harness_prefix(&mut output);
        Ok(output)
    }

    pub(crate) fn spawn(&mut self) -> std::io::Result<OrgizeLibraryCliChild> {
        let mut child = self.command()?.spawn()?;
        let stdin = child.stdin.take();
        Ok(OrgizeLibraryCliChild { stdin, child })
    }
}

impl OrgizeLibraryCliChild {
    pub(crate) fn wait_with_output(mut self) -> std::io::Result<Output> {
        drop(self.stdin.take());
        let mut output = self.child.wait_with_output()?;
        strip_harness_prefix(&mut output);
        Ok(output)
    }
}

fn strip_harness_prefix(output: &mut Output) {
    if let Some(offset) = output
        .stdout
        .windows(STDOUT_MARKER.len())
        .position(|window| window == STDOUT_MARKER.as_bytes())
    {
        output.stdout.drain(..offset + STDOUT_MARKER.len());
    }
}

#[test]
#[ignore = "test-only library CLI subprocess entrypoint"]
fn orgize_cli_library_subprocess() {
    let Ok(arg_count) = std::env::var(REQUEST_ENV) else {
        return;
    };
    let arg_count = arg_count
        .parse::<usize>()
        .expect("library CLI argument count");
    let args = (0..arg_count)
        .map(|index| std::env::var(format!("{ARG_PREFIX}{index}")).expect("library CLI argument"))
        .collect::<Vec<_>>();

    print!("{STDOUT_MARKER}");
    std::io::Write::flush(&mut std::io::stdout()).expect("flush library CLI marker");
    let exit_code = match orgize::cli::run_args(args) {
        Ok(exit_code) => exit_code,
        Err(error) => {
            eprintln!("orgize: {error}");
            ExitCode::from(2)
        }
    };
    std::io::Write::flush(&mut std::io::stdout()).expect("flush library CLI stdout");
    std::io::Write::flush(&mut std::io::stderr()).expect("flush library CLI stderr");
    let status = if exit_code == ExitCode::SUCCESS {
        0
    } else if exit_code == ExitCode::from(1) {
        1
    } else if exit_code == ExitCode::from(2) {
        2
    } else {
        255
    };
    std::process::exit(status);
}
