//! Host execution model for explicit Org element bindings.

use std::fmt;

/// Explicit host execution directives projected from Org keywords.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementsExecutionPlan<A = ()> {
    pub python_directives: Vec<PythonDirective<A>>,
}

/// One executable Python directive from `#+PYTHON:` or `#+PYTHON_FILE:`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonDirective<A = ()> {
    pub ann: A,
    pub kind: PythonDirectiveKind,
    pub value: String,
    pub raw: String,
}

/// Supported Python directive sources.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PythonDirectiveKind {
    /// Inline Python code from `#+PYTHON:`.
    Inline,
    /// Python script path from `#+PYTHON_FILE:`.
    File,
}

/// Python program selected by an explicit host call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PythonExecutionProgram {
    Inline(String),
    File(String),
}

/// Generic host process selected by an explicit caller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementsHostExecutionOptions {
    pub command: String,
    pub args: Vec<String>,
}

impl OrgElementsHostExecutionOptions {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }
}

/// Options for running Python with a JSON Org elements payload on stdin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonExecutionOptions {
    pub interpreter: String,
    pub isolated: bool,
    pub program: PythonExecutionProgram,
}

impl PythonExecutionOptions {
    pub fn inline(code: impl Into<String>) -> Self {
        Self {
            interpreter: "python3".to_string(),
            isolated: true,
            program: PythonExecutionProgram::Inline(code.into()),
        }
    }

    pub fn file(path: impl Into<String>) -> Self {
        Self {
            interpreter: "python3".to_string(),
            isolated: true,
            program: PythonExecutionProgram::File(path.into()),
        }
    }

    pub fn with_interpreter(mut self, interpreter: impl Into<String>) -> Self {
        self.interpreter = interpreter.into();
        self
    }

    pub fn without_isolated(mut self) -> Self {
        self.isolated = false;
        self
    }

    pub fn to_host_options(&self) -> OrgElementsHostExecutionOptions {
        let mut options = OrgElementsHostExecutionOptions::new(self.interpreter.clone());
        if self.isolated {
            options.args.push("-I".to_string());
        }
        match &self.program {
            PythonExecutionProgram::Inline(code) => {
                options.args.push("-c".to_string());
                options.args.push(code.clone());
            }
            PythonExecutionProgram::File(path) => {
                options.args.push(path.clone());
            }
        }
        options
    }
}

/// Exit status from a host execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementsHostExecutionStatus {
    pub success: bool,
    pub code: Option<i32>,
}

/// Captured output from a host execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementsHostExecutionOutput {
    pub status: OrgElementsHostExecutionStatus,
    pub stdout: String,
    pub stderr: String,
}

/// Host process error while starting or communicating with a tool.
#[derive(Debug)]
pub enum OrgElementsHostExecutionError {
    Spawn(std::io::Error),
    Stdin(std::io::Error),
    Wait(std::io::Error),
}

impl fmt::Display for OrgElementsHostExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn(error) => write!(f, "failed to start Org elements host: {error}"),
            Self::Stdin(error) => write!(f, "failed to write Org elements to host: {error}"),
            Self::Wait(error) => write!(f, "failed to wait for Org elements host: {error}"),
        }
    }
}

impl std::error::Error for OrgElementsHostExecutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spawn(error) | Self::Stdin(error) | Self::Wait(error) => Some(error),
        }
    }
}
