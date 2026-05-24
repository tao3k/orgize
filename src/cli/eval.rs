use std::{fs, io::Read, path::PathBuf, process::ExitCode};

use crate::{
    Org,
    ast::{
        BabelEvalOutput, BabelEvalPlan, BabelEvalPlanError, BabelEvalResultPatch,
        BabelEvalResultPatchKind, SourceBlockEvalPolicy, SourceBlockHeaderArg,
        SourceBlockHeaderArgSource, SourceBlockResultHandling, SourceBlockResultValueType,
    },
};

pub(super) fn run(args: Vec<String>) -> Result<ExitCode, String> {
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        print_usage();
        return Ok(ExitCode::from(2));
    };

    match command.as_str() {
        "plan" => run_plan(args.collect()),
        "patch" => run_patch(args.collect()),
        "-h" | "--help" | "help" => {
            print_usage();
            Ok(ExitCode::SUCCESS)
        }
        command => Err(format!("unknown eval command `{command}`")),
    }
}

fn run_plan(args: Vec<String>) -> Result<ExitCode, String> {
    let args = parse_plan_args(args)?;
    if args.help {
        print_plan_usage();
        return Ok(ExitCode::SUCCESS);
    }
    let (source, display_path) = read_source(args.path.as_ref())?;
    let document = Org::parse(&source).document();
    let plan = document
        .babel_eval_plan(args.name.as_str())
        .map_err(render_plan_error)?;
    if args.json {
        println!("{}", plan_json(&plan, &display_path));
    } else {
        print!("{}", plan_compact(&plan, &display_path));
    }
    Ok(ExitCode::SUCCESS)
}

fn run_patch(args: Vec<String>) -> Result<ExitCode, String> {
    let args = parse_patch_args(args)?;
    if args.help {
        print_patch_usage();
        return Ok(ExitCode::SUCCESS);
    }
    let path = args
        .path
        .as_ref()
        .ok_or_else(|| "eval patch requires PATH".to_string())?;
    let display_path = path.display().to_string();
    let source = fs::read_to_string(path).map_err(|error| format!("{display_path}: {error}"))?;
    let document = Org::parse(&source).document();
    let plan = document
        .babel_eval_plan(args.name.as_str())
        .map_err(render_plan_error)?;
    let output = BabelEvalOutput {
        stdout: args.stdout.unwrap_or_default(),
        stderr: args.stderr.unwrap_or_default(),
        exit_code: args.exit_code,
    };
    let patch = plan.result_patch(&source, &output);
    if args.write {
        let next = patch.apply_to(&source);
        if next != source {
            fs::write(path, next).map_err(|error| format!("{display_path}: {error}"))?;
        }
    }
    if args.json {
        println!("{}", patch_json(&plan, &patch, &display_path, args.write));
    } else {
        print!(
            "{}",
            patch_compact(&plan, &patch, &display_path, args.write)
        );
    }
    Ok(ExitCode::SUCCESS)
}

#[derive(Default)]
struct PlanArgs {
    help: bool,
    json: bool,
    name: String,
    path: Option<PathBuf>,
}

fn parse_plan_args(args: Vec<String>) -> Result<PlanArgs, String> {
    let (mut parsed, positional) =
        args.into_iter()
            .try_fold((PlanArgs::default(), Vec::new()), |mut state, arg| {
                match arg.as_str() {
                    "--json" => state.0.json = true,
                    "-h" | "--help" => state.0.help = true,
                    _ if arg.starts_with('-') => {
                        return Err(format!("unknown eval plan flag `{arg}`"));
                    }
                    _ => state.1.push(arg),
                }
                Ok(state)
            })?;
    if parsed.help {
        return Ok(parsed);
    }
    parsed.name = positional
        .first()
        .cloned()
        .ok_or_else(|| "eval plan requires NAME".to_string())?;
    parsed.path = positional.get(1).map(PathBuf::from);
    if positional.len() > 2 {
        return Err("eval plan accepts at most NAME and PATH".to_string());
    }
    Ok(parsed)
}

#[derive(Default)]
struct PatchArgs {
    help: bool,
    json: bool,
    write: bool,
    name: String,
    path: Option<PathBuf>,
    stdout: Option<String>,
    stderr: Option<String>,
    exit_code: Option<i32>,
}

fn parse_patch_args(args: Vec<String>) -> Result<PatchArgs, String> {
    let mut parsed = PatchArgs::default();
    let mut positional = Vec::new();
    let mut index = 0usize;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--json" => parsed.json = true,
            "--write" => parsed.write = true,
            "--stdout" => {
                index += 1;
                parsed.stdout = Some(required_arg(&args, index, "--stdout")?.to_string());
            }
            "--stderr" => {
                index += 1;
                parsed.stderr = Some(required_arg(&args, index, "--stderr")?.to_string());
            }
            "--stdout-file" => {
                index += 1;
                parsed.stdout = Some(read_flag_file(required_arg(
                    &args,
                    index,
                    "--stdout-file",
                )?)?);
            }
            "--stderr-file" => {
                index += 1;
                parsed.stderr = Some(read_flag_file(required_arg(
                    &args,
                    index,
                    "--stderr-file",
                )?)?);
            }
            "--exit-code" => {
                index += 1;
                let value = required_arg(&args, index, "--exit-code")?;
                parsed.exit_code = Some(
                    value
                        .parse::<i32>()
                        .map_err(|_| format!("unsupported exit code `{value}`"))?,
                );
            }
            "-h" | "--help" => parsed.help = true,
            _ if arg.starts_with('-') => return Err(format!("unknown eval patch flag `{arg}`")),
            _ => positional.push(arg.clone()),
        }
        index += 1;
    }
    if parsed.help {
        return Ok(parsed);
    }
    parsed.name = positional
        .first()
        .cloned()
        .ok_or_else(|| "eval patch requires NAME".to_string())?;
    parsed.path = positional.get(1).map(PathBuf::from);
    if positional.len() > 2 {
        return Err("eval patch accepts at most NAME and PATH".to_string());
    }
    Ok(parsed)
}

fn required_arg<'a>(args: &'a [String], index: usize, flag: &str) -> Result<&'a str, String> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn read_flag_file(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))
}

fn read_source(path: Option<&PathBuf>) -> Result<(String, String), String> {
    let Some(path) = path else {
        return read_stdin().map(|source| (source, "<stdin>".to_string()));
    };
    let display_path = path.display().to_string();
    let source = fs::read_to_string(path).map_err(|error| format!("{display_path}: {error}"))?;
    Ok((source, display_path))
}

fn read_stdin() -> Result<String, String> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| format!("failed to read stdin: {error}"))?;
    Ok(input)
}

fn render_plan_error(error: BabelEvalPlanError) -> String {
    match error {
        BabelEvalPlanError::EmptyName => "eval name is empty".to_string(),
        BabelEvalPlanError::NotFound { name } => format!("eval block `{name}` not found"),
        BabelEvalPlanError::Ambiguous { name, matches } => {
            format!("eval block `{name}` is ambiguous ({matches} matches)")
        }
    }
}

fn plan_compact(plan: &BabelEvalPlan, path: &str) -> String {
    let record = &plan.record;
    let mut rendered = String::new();
    rendered.push_str("orgize eval plan\n");
    rendered.push_str(&format!("source: {path}\n"));
    rendered.push_str(&format!("name: {}\n", plan.name));
    if let Some(language) = record.language.as_deref() {
        rendered.push_str(&format!("language: {language}\n"));
    }
    rendered.push_str(&format!(
        "eval: {}\n",
        eval_policy_label(record.execution.eval.policy)
    ));
    rendered.push_str(&format!(
        "results: {} {}\n",
        result_value_type_label(record.result_options.value_type),
        result_handling_label(record.result_options.handling)
    ));
    rendered.push_str(&format!(
        "block-range: {}..{}\n",
        record.source.range_start, record.source.range_end
    ));
    if let Some(result) = &record.result {
        rendered.push_str(&format!(
            "result-range: {}..{}\n",
            result.source.range_start, result.source.range_end
        ));
    } else {
        rendered.push_str("result-range: none\n");
    }
    rendered
}

fn patch_compact(
    plan: &BabelEvalPlan,
    patch: &BabelEvalResultPatch,
    path: &str,
    written: bool,
) -> String {
    let mut rendered = String::new();
    rendered.push_str("orgize eval patch\n");
    rendered.push_str(&format!("source: {path}\n"));
    rendered.push_str(&format!("name: {}\n", plan.name));
    rendered.push_str(&format!("kind: {}\n", patch_kind_label(patch.kind)));
    rendered.push_str(&format!(
        "handling: {}\n",
        result_handling_label(patch.handling)
    ));
    if let Some(range) = patch.range {
        rendered.push_str(&format!("range: {}..{}\n", range.start, range.end));
    } else {
        rendered.push_str("range: none\n");
    }
    rendered.push_str(&format!("written: {written}\n"));
    if let Some(message) = patch.message.as_deref() {
        rendered.push_str(&format!("message: {message}\n"));
    }
    if !written && !patch.replacement.is_empty() {
        rendered.push_str("replacement:\n");
        rendered.push_str(&patch.replacement);
    }
    rendered
}

fn plan_json(plan: &BabelEvalPlan, path: &str) -> serde_json::Value {
    let record = &plan.record;
    serde_json::json!({
        "source": path,
        "name": plan.name,
        "language": record.language,
        "parameters": record.parameters,
        "body": record.value,
        "eval": {
            "raw": record.execution.eval.raw,
            "policy": eval_policy_label(record.execution.eval.policy),
            "source": header_source_label(record.execution.eval.source),
        },
        "results": {
            "raw": record.result_options.raw,
            "handling": result_handling_label(record.result_options.handling),
            "valueType": result_value_type_label(record.result_options.value_type),
            "tokens": record.result_options.tokens,
        },
        "headerArgs": header_args_json(&record.normalized_header_args),
        "blockRange": {
            "start": record.source.range_start,
            "end": record.source.range_end,
        },
        "resultRange": record.result.as_ref().map(|result| serde_json::json!({
            "start": result.source.range_start,
            "end": result.source.range_end,
        })),
    })
}

fn patch_json(
    plan: &BabelEvalPlan,
    patch: &BabelEvalResultPatch,
    path: &str,
    written: bool,
) -> serde_json::Value {
    serde_json::json!({
        "source": path,
        "name": plan.name,
        "kind": patch_kind_label(patch.kind),
        "handling": result_handling_label(patch.handling),
        "range": patch.range.map(|range| serde_json::json!({
            "start": range.start,
            "end": range.end,
        })),
        "replacement": patch.replacement,
        "written": written,
        "message": patch.message,
    })
}

fn header_args_json(args: &[SourceBlockHeaderArg]) -> Vec<serde_json::Value> {
    args.iter()
        .map(|arg| {
            serde_json::json!({
                "key": arg.key,
                "value": arg.value,
                "raw": arg.raw,
                "source": header_source_label(arg.source),
                "tokens": arg.tokens,
            })
        })
        .collect()
}

fn eval_policy_label(policy: SourceBlockEvalPolicy) -> &'static str {
    match policy {
        SourceBlockEvalPolicy::Yes => "yes",
        SourceBlockEvalPolicy::No => "no",
        SourceBlockEvalPolicy::NoExport => "no-export",
        SourceBlockEvalPolicy::StripExport => "strip-export",
        SourceBlockEvalPolicy::NeverExport => "never-export",
        SourceBlockEvalPolicy::Eval => "eval",
        SourceBlockEvalPolicy::Never => "never",
        SourceBlockEvalPolicy::Query => "query",
        SourceBlockEvalPolicy::Other => "other",
    }
}

fn result_handling_label(handling: SourceBlockResultHandling) -> &'static str {
    match handling {
        SourceBlockResultHandling::Replace => "replace",
        SourceBlockResultHandling::Silent => "silent",
        SourceBlockResultHandling::None => "none",
        SourceBlockResultHandling::Discard => "discard",
        SourceBlockResultHandling::Append => "append",
        SourceBlockResultHandling::Prepend => "prepend",
    }
}

fn result_value_type_label(value_type: SourceBlockResultValueType) -> &'static str {
    match value_type {
        SourceBlockResultValueType::Value => "value",
        SourceBlockResultValueType::Output => "output",
    }
}

fn header_source_label(source: SourceBlockHeaderArgSource) -> &'static str {
    match source {
        SourceBlockHeaderArgSource::Explicit => "explicit",
        SourceBlockHeaderArgSource::Default => "default",
    }
}

fn patch_kind_label(kind: BabelEvalResultPatchKind) -> &'static str {
    match kind {
        BabelEvalResultPatchKind::Insert => "insert",
        BabelEvalResultPatchKind::Replace => "replace",
        BabelEvalResultPatchKind::Noop => "noop",
    }
}

fn print_usage() {
    eprintln!("Usage: orgize eval <plan|patch> [options]");
}

fn print_plan_usage() {
    eprintln!("Usage: orgize eval plan [--json] NAME [PATH]");
}

fn print_patch_usage() {
    eprintln!(
        "Usage: orgize eval patch [--json] [--write] [--stdout TEXT|--stdout-file PATH] [--stderr TEXT|--stderr-file PATH] [--exit-code CODE] NAME PATH"
    );
}
