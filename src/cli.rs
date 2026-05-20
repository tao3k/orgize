//! Command-line interface implementation for the `orgize` binary.

use std::{
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    process::ExitCode,
};

use crate::{
    Org,
    ast::{PriorityProfile, PriorityValue},
    fmt::{FormatOptions, format_org},
    lint::{LintOptions, lint_org_with_options},
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
        "sdd" => run_sdd(args.collect()),
        "-h" | "--help" | "help" => {
            print_usage();
            Ok(ExitCode::SUCCESS)
        }
        command => Err(format!("unknown command `{command}`")),
    }
}

fn run_sdd(args: Vec<String>) -> Result<ExitCode, String> {
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        print_sdd_usage();
        return Ok(ExitCode::from(2));
    };

    match command.as_str() {
        "status" => run_sdd_status(args.collect()),
        "-h" | "--help" | "help" => {
            print_sdd_usage();
            Ok(ExitCode::SUCCESS)
        }
        command => Err(format!("unknown sdd command `{command}`")),
    }
}

fn run_sdd_status(args: Vec<String>) -> Result<ExitCode, String> {
    let args = parse_sdd_status_args(args)?;
    if args.help {
        print_sdd_status_usage();
        return Ok(ExitCode::SUCCESS);
    }

    if args.paths.is_empty() {
        let source = read_stdin()?;
        let document = Org::parse(&source).document();
        print!("{}", document.sdd_status().to_compact_text("<stdin>"));
        return Ok(ExitCode::SUCCESS);
    }

    for path in collect_org_paths(&args.paths)? {
        let display_path = path.display().to_string();
        let source =
            fs::read_to_string(&path).map_err(|error| format!("{display_path}: {error}"))?;
        let document = Org::parse(&source).document();
        print!("{}", document.sdd_status().to_compact_text(&display_path));
    }

    Ok(ExitCode::SUCCESS)
}

#[derive(Default)]
struct SddStatusArgs {
    help: bool,
    paths: Vec<String>,
}

fn parse_sdd_status_args(args: Vec<String>) -> Result<SddStatusArgs, String> {
    args.into_iter()
        .try_fold(SddStatusArgs::default(), |mut parsed, arg| {
            match arg.as_str() {
                "-h" | "--help" => parsed.help = true,
                _ if arg.starts_with('-') => {
                    return Err(format!("unknown sdd status flag `{arg}`"));
                }
                _ => parsed.paths.push(arg),
            }
            Ok(parsed)
        })
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
    let mut priority_highest = None;
    let mut priority_lowest = None;
    let mut priority_default = None;
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
            "--priority-highest" => {
                index += 1;
                priority_highest = Some(parse_priority_flag(&args, index, "--priority-highest")?);
            }
            "--priority-lowest" => {
                index += 1;
                priority_lowest = Some(parse_priority_flag(&args, index, "--priority-lowest")?);
            }
            "--priority-default" => {
                index += 1;
                priority_default = Some(parse_priority_flag(&args, index, "--priority-default")?);
            }
            "-h" | "--help" => {
                print_lint_usage();
                return Ok(ExitCode::SUCCESS);
            }
            _ if arg.starts_with('-') => return Err(format!("unknown lint flag `{arg}`")),
            _ => paths.push(arg.clone()),
        }
        index += 1;
    }

    let priority_profile =
        priority_profile_from_flags(priority_highest, priority_lowest, priority_default)?;
    let base_lint_options = LintOptions {
        priority_profile,
        ..LintOptions::default()
    };

    let mut reports = Vec::new();
    if paths.is_empty() {
        let source = read_stdin()?;
        let report = lint_org_with_options(&source, &base_lint_options);
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
                ..base_lint_options.clone()
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

fn parse_priority_flag(
    args: &[String],
    index: usize,
    flag: &'static str,
) -> Result<PriorityValue, String> {
    let Some(value) = args.get(index) else {
        return Err(format!("lint {flag} requires a priority value"));
    };
    PriorityValue::parse(value).ok_or_else(|| format!("unsupported priority value `{value}`"))
}

fn priority_profile_from_flags(
    highest: Option<PriorityValue>,
    lowest: Option<PriorityValue>,
    default: Option<PriorityValue>,
) -> Result<PriorityProfile, String> {
    if highest.is_none() && lowest.is_none() && default.is_none() {
        return Ok(PriorityProfile::org_default());
    }
    let profile = PriorityProfile::org_default();
    let highest = highest.unwrap_or_else(|| profile.highest().clone());
    let lowest = lowest.unwrap_or_else(|| profile.lowest().clone());
    let default = default.unwrap_or_else(|| profile.default_priority().clone());
    PriorityProfile::new(highest, lowest, default).ok_or_else(|| {
        "priority profile must use one priority family and satisfy highest <= default <= lowest"
            .to_string()
    })
}

fn print_usage() {
    eprintln!("Usage: orgize <fmt|lint|sdd> [options] [PATH ...]");
}

fn print_fmt_usage() {
    eprintln!("Usage: orgize fmt [--check] [PATH ...]");
}

fn print_lint_usage() {
    eprintln!(
        "Usage: orgize lint [--format compact|text|json] [--priority-highest VALUE] [--priority-default VALUE] [--priority-lowest VALUE] [PATH ...]"
    );
}

fn print_sdd_usage() {
    eprintln!("Usage: orgize sdd <status> [options] [PATH ...]");
}

fn print_sdd_status_usage() {
    eprintln!("Usage: orgize sdd status [PATH ...]");
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
