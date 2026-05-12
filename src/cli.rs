//! Command-line interface implementation for the `orgize` binary.

use std::{env, fs, io::Read, process::ExitCode};

use crate::{
    fmt::{format_org, FormatOptions},
    lint::lint_org,
};

/// Runs the command-line interface with process arguments and stdio.
pub fn run_from_env() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("orgize: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        return Ok(ExitCode::from(2));
    };

    match command.as_str() {
        "fmt" => run_fmt(args.collect()),
        "lint" => run_lint(args.collect()),
        "-h" | "--help" | "help" => {
            print_usage();
            Ok(ExitCode::SUCCESS)
        }
        command => Err(format!("unknown command `{command}`")),
    }
}

fn run_fmt(args: Vec<String>) -> Result<ExitCode, String> {
    let mut check = false;
    let mut write = false;
    let mut paths = Vec::new();

    for arg in args {
        match arg.as_str() {
            "--check" => check = true,
            "--write" | "-w" => write = true,
            "-h" | "--help" => {
                print_fmt_usage();
                return Ok(ExitCode::SUCCESS);
            }
            _ if arg.starts_with('-') => return Err(format!("unknown fmt flag `{arg}`")),
            _ => paths.push(arg),
        }
    }

    if check && write {
        return Err("fmt cannot combine --check and --write".to_string());
    }
    if write && paths.is_empty() {
        return Err("fmt --write requires at least one path".to_string());
    }

    let options = FormatOptions::default();
    let mut changed = false;

    if paths.is_empty() {
        let source = read_stdin()?;
        let formatted = format_org(&source, &options);
        changed |= formatted.changed;
        if check {
            if formatted.changed {
                eprintln!("<stdin>: needs formatting");
            }
        } else {
            print!("{}", formatted.output);
        }
    } else {
        for path in paths {
            let source = fs::read_to_string(&path).map_err(|error| format!("{path}: {error}"))?;
            let formatted = format_org(&source, &options);
            changed |= formatted.changed;
            if check {
                if formatted.changed {
                    eprintln!("{path}: needs formatting");
                }
            } else if write {
                if formatted.changed {
                    fs::write(&path, formatted.output)
                        .map_err(|error| format!("{path}: {error}"))?;
                }
            } else {
                print!("{}", formatted.output);
            }
        }
    }

    Ok(if check && changed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn run_lint(args: Vec<String>) -> Result<ExitCode, String> {
    let mut output_format = LintOutputFormat::Text;
    let mut paths = Vec::new();
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--format" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("lint --format requires `text` or `json`".to_string());
                };
                output_format = LintOutputFormat::parse(value)?;
            }
            "--json" => output_format = LintOutputFormat::Json,
            "-h" | "--help" => {
                print_lint_usage();
                return Ok(ExitCode::SUCCESS);
            }
            _ if arg.starts_with('-') => return Err(format!("unknown lint flag `{arg}`")),
            _ => paths.push(arg.clone()),
        }
        index += 1;
    }

    let mut reports = Vec::new();
    if paths.is_empty() {
        let source = read_stdin()?;
        reports.push(("<stdin>".to_string(), lint_org(&source)));
    } else {
        for path in paths {
            let source = fs::read_to_string(&path).map_err(|error| format!("{path}: {error}"))?;
            reports.push((path, lint_org(&source)));
        }
    }

    let has_findings = reports.iter().any(|(_, report)| !report.is_clean());
    match output_format {
        LintOutputFormat::Text => {
            for (path, report) in &reports {
                print!("{}", report.to_text(path));
            }
        }
        LintOutputFormat::Json => {
            print!("{{\"files\":[");
            for (index, (path, report)) in reports.iter().enumerate() {
                if index > 0 {
                    print!(",");
                }
                print!("{}", report.to_json_file(path));
            }
            println!("]}}");
        }
    }

    Ok(if has_findings {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn read_stdin() -> Result<String, String> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| format!("failed to read stdin: {error}"))?;
    Ok(input)
}

#[derive(Clone, Copy)]
enum LintOutputFormat {
    Text,
    Json,
}

impl LintOutputFormat {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            _ => Err(format!("unsupported lint output format `{value}`")),
        }
    }
}

fn print_usage() {
    eprintln!("Usage: orgize <fmt|lint> [options] [PATH ...]");
}

fn print_fmt_usage() {
    eprintln!("Usage: orgize fmt [--check|--write] [PATH ...]");
}

fn print_lint_usage() {
    eprintln!("Usage: orgize lint [--format text|json] [PATH ...]");
}
