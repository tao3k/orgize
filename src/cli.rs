//! Command-line interface implementation for the `orgize` binary.

use std::{
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    process::ExitCode,
};

use crate::{
    fmt::{format_org, FormatOptions},
    lint::{lint_org, lint_org_with_options, LintOptions},
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
    let mut paths = Vec::new();

    for arg in args {
        match arg.as_str() {
            "--check" => check = true,
            "--write" | "-w" => {}
            "-h" | "--help" => {
                print_fmt_usage();
                return Ok(ExitCode::SUCCESS);
            }
            _ if arg.starts_with('-') => return Err(format!("unknown fmt flag `{arg}`")),
            _ => paths.push(arg),
        }
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
        for path in collect_org_paths(&paths)? {
            let display_path = path.display().to_string();
            let source =
                fs::read_to_string(&path).map_err(|error| format!("{display_path}: {error}"))?;
            let formatted = format_org(&source, &options);
            changed |= formatted.changed;
            if check {
                if formatted.changed {
                    eprintln!("{display_path}: needs formatting");
                }
            } else {
                if formatted.changed {
                    fs::write(&path, formatted.output)
                        .map_err(|error| format!("{display_path}: {error}"))?;
                }
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
    let mut output_format = LintOutputFormat::Compact;
    let mut paths = Vec::new();
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--format" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("lint --format requires `compact`, `text`, or `json`".to_string());
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
        let report = lint_org(&source);
        reports.push(LintFileReport {
            path: "<stdin>".to_string(),
            source,
            report,
        });
    } else {
        for path in collect_org_paths(&paths)? {
            let display_path = path.display().to_string();
            let source =
                fs::read_to_string(&path).map_err(|error| format!("{display_path}: {error}"))?;
            let lint_options = LintOptions {
                include_base_dir: path.parent().map(Path::to_path_buf),
                attachment_base_dir: path.parent().map(Path::to_path_buf),
                file_base_dir: path.parent().map(Path::to_path_buf),
            };
            let report = lint_org_with_options(&source, &lint_options);
            reports.push(LintFileReport {
                path: display_path,
                source,
                report,
            });
        }
    }

    let has_findings = reports.iter().any(|file| !file.report.is_clean());
    match output_format {
        LintOutputFormat::Compact => {
            print!("{}", render_lint_compact(&reports));
        }
        LintOutputFormat::Text => {
            for file in &reports {
                print!("{}", file.report.to_text(&file.path));
            }
        }
        LintOutputFormat::Json => {
            print!("{{\"files\":[");
            for (index, file) in reports.iter().enumerate() {
                if index > 0 {
                    print!(",");
                }
                print!("{}", file.report.to_json_file(&file.path));
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

struct LintFileReport {
    path: String,
    source: String,
    report: crate::lint::LintReport,
}

fn render_lint_compact(reports: &[LintFileReport]) -> String {
    let rendered = reports
        .iter()
        .filter(|file| !file.report.is_clean())
        .map(|file| file.report.to_compact_text(&file.path, &file.source))
        .collect::<Vec<_>>();

    if rendered.is_empty() {
        "[ok] orgize lint\n".to_string()
    } else {
        rendered.join("\n")
    }
}

#[derive(Clone, Copy)]
enum LintOutputFormat {
    Compact,
    Text,
    Json,
}

impl LintOutputFormat {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "compact" => Ok(Self::Compact),
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
    eprintln!("Usage: orgize fmt [--check] [PATH ...]");
}

fn print_lint_usage() {
    eprintln!("Usage: orgize lint [--format compact|text|json] [PATH ...]");
}

fn collect_org_paths(paths: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for path in paths {
        collect_org_path(Path::new(path), &mut files)?;
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn collect_org_path(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if metadata.is_file() {
        if !is_org_file(path) {
            return Err(format!("{}: expected .org file", path.display()));
        }
        files.push(path.to_path_buf());
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(format!("{}: unsupported path type", path.display()));
    }

    let mut entries = fs::read_dir(path)
        .map_err(|error| format!("{}: {error}", path.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let entry_path = entry.path();
        let entry_type = entry
            .file_type()
            .map_err(|error| format!("{}: {error}", entry_path.display()))?;
        if entry_type.is_dir() {
            collect_org_path(&entry_path, files)?;
        } else if entry_type.is_file() && is_org_file(&entry_path) {
            files.push(entry_path);
        }
    }
    Ok(())
}

fn is_org_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("org"))
}
