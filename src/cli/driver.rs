//! Command-line interface implementation for the `orgize` binary.

use std::{env, fs, process::ExitCode};

use crate::{Org, ast::OrgCapturePlanCommandOutput};

use super::{
    driver_fmt_lint::{run_fmt, run_lint},
    driver_paths::{collect_org_paths, format_path_error, read_stdin},
    driver_sdd::run_sdd,
    driver_tasks::{run_agent_planning, run_sparse_tree, run_task_list},
    driver_usage::{print_export_usage, print_usage},
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
    run_args(env::args().skip(1).collect())
}

pub(crate) fn run_args(args: Vec<String>) -> Result<ExitCode, String> {
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        print_usage();
        return Ok(ExitCode::from(2));
    };

    match command.as_str() {
        "agent-planning" => run_agent_planning(args.collect()),
        "capture-plan" => run_capture_plan(args.collect()),
        "contract" => super::org_contract_trace::run(args.collect()),
        "eval" => super::eval::run(args.collect()),
        "export" => run_export(args.collect()),
        "fmt" => run_fmt(args.collect()),
        "elements-query" | "guide" | "search" | "query" => {
            let mut command_args = vec![command.to_string()];
            command_args.extend(args);
            crate::document::run_org_command(command_args)
        }
        "lint" => run_lint(args.collect()),
        "md" | "markdown" => crate::document::run_md_command(args.collect()),
        "sdd" => run_sdd(args.collect()),
        "sparse-tree" => run_sparse_tree(args.collect()),
        "task-list" => run_task_list(args.collect()),
        "-h" | "--help" | "help" => {
            print_usage();
            Ok(ExitCode::SUCCESS)
        }
        command => Err(format!("unknown command `{command}`")),
    }
}

fn run_capture_plan(args: Vec<String>) -> Result<ExitCode, String> {
    match crate::ast::org_capture_plan_command(args)? {
        OrgCapturePlanCommandOutput::Help(usage) => {
            eprintln!("{usage}");
            Ok(ExitCode::SUCCESS)
        }
        OrgCapturePlanCommandOutput::Plan(plan) => {
            print!("{plan}");
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn run_export(args: Vec<String>) -> Result<ExitCode, String> {
    let mut args = args.into_iter();
    let Some(format) = args.next() else {
        print_export_usage();
        return Ok(ExitCode::from(2));
    };

    match format.as_str() {
        "md" | "markdown" => run_export_markdown(args.collect()),
        "-h" | "--help" | "help" => {
            print_export_usage();
            Ok(ExitCode::SUCCESS)
        }
        format => Err(format!("unknown export format `{format}`")),
    }
}

fn run_export_markdown(paths: Vec<String>) -> Result<ExitCode, String> {
    if paths.is_empty() {
        let source = read_stdin()?;
        print!("{}", Org::parse(source).to_markdown());
        return Ok(ExitCode::SUCCESS);
    }

    for path in collect_org_paths(&paths)? {
        let source = fs::read_to_string(&path).map_err(|error| format_path_error(&path, error))?;
        print!("{}", Org::parse(source).to_markdown());
    }

    Ok(ExitCode::SUCCESS)
}
