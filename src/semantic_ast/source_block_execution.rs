//! Non-executing Babel execution/export planning helpers.

use super::{
    SourceBlockBooleanHeader, SourceBlockCache, SourceBlockDirectory, SourceBlockDirectoryKind,
    SourceBlockEval, SourceBlockEvalPolicy, SourceBlockExecutionPlan, SourceBlockExports,
    SourceBlockExportsPolicy, SourceBlockHeaderArg, SourceBlockHeaderArgSource,
    SourceBlockNowebAction, SourceBlockNowebPlan, SourceBlockSession,
};

pub(super) fn source_block_execution_plan(
    header_args: &[SourceBlockHeaderArg],
) -> SourceBlockExecutionPlan {
    SourceBlockExecutionPlan {
        eval: source_block_eval(header_args),
        exports: source_block_exports(header_args),
        cache: source_block_cache(header_args),
        session: source_block_session(header_args),
        directory: source_block_directory(header_args),
        hlines: source_block_boolean_header(header_args, "hlines", "no"),
        noweb: source_block_noweb_plan(header_args),
    }
}

fn source_block_eval(header_args: &[SourceBlockHeaderArg]) -> SourceBlockEval {
    let arg = last_normalized_header_arg(header_args, "eval");
    let raw = header_arg_normalized_value(arg).unwrap_or_else(|| "yes".to_string());
    let source = arg
        .map(|arg| arg.source)
        .unwrap_or(SourceBlockHeaderArgSource::Default);
    let policy = match raw.to_ascii_lowercase().as_str() {
        "yes" => SourceBlockEvalPolicy::Yes,
        "no" => SourceBlockEvalPolicy::No,
        "no-export" => SourceBlockEvalPolicy::NoExport,
        "strip-export" => SourceBlockEvalPolicy::StripExport,
        "never-export" => SourceBlockEvalPolicy::NeverExport,
        "eval" => SourceBlockEvalPolicy::Eval,
        "never" => SourceBlockEvalPolicy::Never,
        "query" => SourceBlockEvalPolicy::Query,
        _ => SourceBlockEvalPolicy::Other,
    };
    SourceBlockEval {
        raw,
        source,
        policy,
    }
}

fn source_block_exports(header_args: &[SourceBlockHeaderArg]) -> SourceBlockExports {
    let arg = last_normalized_header_arg(header_args, "exports");
    let raw = header_arg_normalized_value(arg).unwrap_or_else(|| "code".to_string());
    let source = arg
        .map(|arg| arg.source)
        .unwrap_or(SourceBlockHeaderArgSource::Default);
    let policy = split_header_value(&raw)
        .iter()
        .rev()
        .find_map(|token| match token.to_ascii_lowercase().as_str() {
            "code" => Some(SourceBlockExportsPolicy::Code),
            "results" => Some(SourceBlockExportsPolicy::Results),
            "both" => Some(SourceBlockExportsPolicy::Both),
            "none" => Some(SourceBlockExportsPolicy::None),
            _ => None,
        })
        .unwrap_or(SourceBlockExportsPolicy::Other);
    SourceBlockExports {
        raw,
        source,
        policy,
    }
}

fn source_block_cache(header_args: &[SourceBlockHeaderArg]) -> SourceBlockCache {
    let arg = last_normalized_header_arg(header_args, "cache");
    let raw = header_arg_normalized_value(arg).unwrap_or_else(|| "no".to_string());
    let source = arg
        .map(|arg| arg.source)
        .unwrap_or(SourceBlockHeaderArgSource::Default);
    SourceBlockCache {
        enabled: raw.eq_ignore_ascii_case("yes") || raw.eq_ignore_ascii_case("t"),
        raw,
        source,
    }
}

fn source_block_session(header_args: &[SourceBlockHeaderArg]) -> SourceBlockSession {
    let arg = last_normalized_header_arg(header_args, "session");
    let raw = header_arg_normalized_value(arg).unwrap_or_else(|| "none".to_string());
    let source = arg
        .map(|arg| arg.source)
        .unwrap_or(SourceBlockHeaderArgSource::Default);
    let active = !raw.is_empty() && !raw.eq_ignore_ascii_case("none");
    SourceBlockSession {
        name: active.then(|| raw.clone()),
        raw,
        source,
        active,
    }
}

fn source_block_directory(header_args: &[SourceBlockHeaderArg]) -> Option<SourceBlockDirectory> {
    let arg = last_normalized_header_arg(header_args, "dir")?;
    let raw = header_arg_normalized_value(Some(arg))?;
    if raw.is_empty() {
        return None;
    }
    let kind = if raw.eq_ignore_ascii_case("attach") || raw.eq_ignore_ascii_case("'attach") {
        SourceBlockDirectoryKind::Attachment
    } else {
        SourceBlockDirectoryKind::Path
    };
    Some(SourceBlockDirectory {
        target: raw.clone(),
        raw,
        source: arg.source,
        kind,
    })
}

fn source_block_boolean_header(
    header_args: &[SourceBlockHeaderArg],
    key: &str,
    default: &str,
) -> SourceBlockBooleanHeader {
    let arg = last_normalized_header_arg(header_args, key);
    let raw = header_arg_normalized_value(arg).unwrap_or_else(|| default.to_string());
    let source = arg
        .map(|arg| arg.source)
        .unwrap_or(SourceBlockHeaderArgSource::Default);
    SourceBlockBooleanHeader {
        enabled: raw.eq_ignore_ascii_case("yes") || raw.eq_ignore_ascii_case("t"),
        raw,
        source,
    }
}

fn source_block_noweb_plan(header_args: &[SourceBlockHeaderArg]) -> SourceBlockNowebPlan {
    let arg = last_normalized_header_arg(header_args, "noweb");
    let raw = header_arg_normalized_value(arg).unwrap_or_else(|| "no".to_string());
    let source = arg
        .map(|arg| arg.source)
        .unwrap_or(SourceBlockHeaderArgSource::Default);
    let tokens = split_header_value(&raw);
    SourceBlockNowebPlan {
        eval: source_block_noweb_action(&tokens, NowebContext::Eval),
        export: source_block_noweb_action(&tokens, NowebContext::Export),
        tangle: source_block_noweb_action(&tokens, NowebContext::Tangle),
        raw,
        source,
        tokens,
    }
}

#[derive(Clone, Copy)]
enum NowebContext {
    Eval,
    Export,
    Tangle,
}

fn source_block_noweb_action(tokens: &[String], context: NowebContext) -> SourceBlockNowebAction {
    let mut action = SourceBlockNowebAction::Disabled;
    for token in tokens {
        let token = token.to_ascii_lowercase();
        let next = match (context, token.as_str()) {
            (
                NowebContext::Eval,
                "yes" | "no-export" | "strip-export" | "eval" | "strip-tangle",
            ) => SourceBlockNowebAction::Expand,
            (NowebContext::Export, "strip-export" | "strip-tangle") => {
                SourceBlockNowebAction::Strip
            }
            (NowebContext::Export, "yes") => SourceBlockNowebAction::Expand,
            (NowebContext::Tangle, "strip-tangle") => SourceBlockNowebAction::Strip,
            (NowebContext::Tangle, "yes" | "tangle" | "no-export" | "strip-export") => {
                SourceBlockNowebAction::Expand
            }
            _ => action,
        };
        action = next;
    }
    action
}

fn last_normalized_header_arg<'a>(
    header_args: &'a [SourceBlockHeaderArg],
    key: &str,
) -> Option<&'a SourceBlockHeaderArg> {
    header_args
        .iter()
        .rev()
        .find(|arg| arg.key.eq_ignore_ascii_case(key))
}

fn header_arg_normalized_value(arg: Option<&SourceBlockHeaderArg>) -> Option<String> {
    arg.and_then(|arg| arg.value.as_deref())
        .map(str::trim)
        .map(unquote_header_value)
}

fn split_header_value(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;

    for ch in value.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if quote == Some(ch) {
            quote = None;
        } else if quote.is_none() && matches!(ch, '"' | '\'') {
            quote = Some(ch);
        } else if quote.is_none() && ch.is_whitespace() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }

    if escaped {
        current.push('\\');
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn unquote_header_value(value: &str) -> String {
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}
