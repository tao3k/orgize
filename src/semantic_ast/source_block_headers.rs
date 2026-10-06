//! Source-block header argument projection for Babel-aware consumers.

use super::{
    BlockHeaderArg, Element, ParsedAnnotation, Property, SourceBlockHeaderArg,
    SourceBlockHeaderArgKind, SourceBlockHeaderArgSource, SourceBlockHeaderVar,
    SourceBlockRecordKind, SourceBlockResultCollection, SourceBlockResultFile,
    SourceBlockResultFileMode, SourceBlockResultFormat, SourceBlockResultHandling,
    SourceBlockResultOptions, SourceBlockResultValueType, SourceBlockTangle,
    SourceBlockTangleComments, SourceBlockTangleCommentsMode, SourceBlockTangleMkdirp,
    SourceBlockTangleMode, SourceBlockTangleNoweb, SourceBlockTangleNowebMode,
};

pub(super) fn explicit_source_block_header_args(
    element: &Element<ParsedAnnotation>,
    language: Option<&str>,
    properties: &[Property<ParsedAnnotation>],
    begin_line_args: &[BlockHeaderArg],
) -> Vec<BlockHeaderArg> {
    let mut header_args = property_header_args(properties, language);
    header_args.extend(
        element
            .affiliated_keywords
            .iter()
            .filter(|keyword| {
                keyword.key.eq_ignore_ascii_case("HEADER")
                    || keyword.key.eq_ignore_ascii_case("HEADERS")
            })
            .flat_map(|keyword| keyword.ann.header_args.iter().cloned()),
    );
    header_args.extend(begin_line_args.iter().cloned());
    header_args
}

pub(super) fn explicit_inline_source_header_args(
    language: &str,
    properties: &[Property<ParsedAnnotation>],
    inline_header_args: &[BlockHeaderArg],
) -> Vec<BlockHeaderArg> {
    let mut header_args = property_header_args(properties, Some(language));
    header_args.extend_from_slice(inline_header_args);
    header_args
}

fn property_header_args(
    properties: &[Property<ParsedAnnotation>],
    language: Option<&str>,
) -> Vec<BlockHeaderArg> {
    let mut header_args = Vec::new();
    header_args.extend(
        properties
            .iter()
            .filter(|property| property.key.eq_ignore_ascii_case("header-args"))
            .flat_map(|property| property.ann.header_args.iter().cloned()),
    );
    if let Some(language) = language {
        header_args.extend(
            properties
                .iter()
                .filter(|property| is_language_header_args_property(&property.key, language))
                .flat_map(|property| property.ann.header_args.iter().cloned()),
        );
    }
    header_args
}

fn is_language_header_args_property(key: &str, language: &str) -> bool {
    super::org_native_values::scalar("header-language", &[key, language]) == "true"
}

pub(super) fn source_block_tangle(header_args: &[BlockHeaderArg]) -> Option<SourceBlockTangle> {
    let arg = last_header_arg(header_args, "tangle")?;
    let raw_value = arg.value.clone().unwrap_or_else(|| "yes".to_string());
    let normalized = unquote_header_value(raw_value.trim());
    let mode = match header_policy("tangle", &normalized).as_str() {
        "no" => SourceBlockTangleMode::No,
        "yes" => SourceBlockTangleMode::Yes,
        "file" => SourceBlockTangleMode::File,
        _ => panic!("native tangle policy"),
    };
    let target = (mode == SourceBlockTangleMode::File).then_some(normalized);
    Some(SourceBlockTangle {
        raw: arg.raw.clone(),
        mode,
        target,
        mkdirp: source_block_tangle_mkdirp(header_args),
        comments: source_block_tangle_comments(header_args),
        shebang: source_block_tangle_shebang(header_args),
        noweb: source_block_tangle_noweb(header_args),
    })
}

fn source_block_tangle_mkdirp(header_args: &[BlockHeaderArg]) -> SourceBlockTangleMkdirp {
    let raw = header_value(header_args, "mkdirp").unwrap_or_else(|| "no".to_string());
    SourceBlockTangleMkdirp {
        enabled: header_policy("mkdirp", &raw) == "true",
        raw,
    }
}

fn source_block_tangle_comments(header_args: &[BlockHeaderArg]) -> SourceBlockTangleComments {
    let raw = header_value(header_args, "comments").unwrap_or_else(|| "no".to_string());
    let mode = match header_policy("comments", &raw).as_str() {
        "no" => SourceBlockTangleCommentsMode::No,
        "link" => SourceBlockTangleCommentsMode::Link,
        "yes" => SourceBlockTangleCommentsMode::Yes,
        "org" => SourceBlockTangleCommentsMode::Org,
        "both" => SourceBlockTangleCommentsMode::Both,
        "noweb" => SourceBlockTangleCommentsMode::Noweb,
        _ => SourceBlockTangleCommentsMode::Other,
    };
    SourceBlockTangleComments { raw, mode }
}

fn source_block_tangle_shebang(header_args: &[BlockHeaderArg]) -> Option<String> {
    header_value(header_args, "shebang").filter(|value| !value.is_empty())
}

fn source_block_tangle_noweb(header_args: &[BlockHeaderArg]) -> SourceBlockTangleNoweb {
    let raw = header_value(header_args, "noweb").unwrap_or_else(|| "no".to_string());
    let mode = match header_policy("noweb", &raw).as_str() {
        "strip" => SourceBlockTangleNowebMode::Strip,
        "expand" => SourceBlockTangleNowebMode::Expand,
        "disabled" => SourceBlockTangleNowebMode::Disabled,
        _ => panic!("native noweb policy"),
    };
    SourceBlockTangleNoweb { raw, mode }
}

pub(super) fn source_block_result_options(
    header_args: &[SourceBlockHeaderArg],
) -> SourceBlockResultOptions {
    let mut options = SourceBlockResultOptions {
        raw: ":results replace".to_string(),
        source: SourceBlockHeaderArgSource::Default,
        tokens: vec!["replace".to_string()],
        collection: None,
        format: None,
        handling: SourceBlockResultHandling::Replace,
        value_type: SourceBlockResultValueType::Value,
        unknown: Vec::new(),
        file: None,
    };

    for arg in result_header_args(header_args) {
        apply_result_header_arg(&mut options, arg);
    }

    options.file = source_block_result_file(header_args);
    options
}

fn result_header_args(
    header_args: &[SourceBlockHeaderArg],
) -> impl Iterator<Item = &SourceBlockHeaderArg> {
    header_args
        .iter()
        .filter(|arg| arg.kind == SourceBlockHeaderArgKind::Results)
}

fn apply_result_header_arg(options: &mut SourceBlockResultOptions, arg: &SourceBlockHeaderArg) {
    options.raw = arg.raw.clone();
    options.source = arg.source;
    options.tokens = arg.tokens.clone();
    for token in &arg.tokens {
        apply_result_token(options, token);
    }
}

fn apply_result_token(options: &mut SourceBlockResultOptions, token: &str) {
    match header_policy("result", token).as_str() {
        "file" => options.collection = Some(SourceBlockResultCollection::File),
        "list" => options.collection = Some(SourceBlockResultCollection::List),
        "vector" => options.collection = Some(SourceBlockResultCollection::Vector),
        "table" => options.collection = Some(SourceBlockResultCollection::Table),
        "scalar" => options.collection = Some(SourceBlockResultCollection::Scalar),
        "verbatim" => options.collection = Some(SourceBlockResultCollection::Verbatim),
        "raw" => options.format = Some(SourceBlockResultFormat::Raw),
        "html" => options.format = Some(SourceBlockResultFormat::Html),
        "latex" => options.format = Some(SourceBlockResultFormat::Latex),
        "org" => options.format = Some(SourceBlockResultFormat::Org),
        "code" => options.format = Some(SourceBlockResultFormat::Code),
        "pp" => options.format = Some(SourceBlockResultFormat::Pp),
        "drawer" => options.format = Some(SourceBlockResultFormat::Drawer),
        "link" => options.format = Some(SourceBlockResultFormat::Link),
        "graphics" => options.format = Some(SourceBlockResultFormat::Graphics),
        "replace" => options.handling = SourceBlockResultHandling::Replace,
        "silent" => options.handling = SourceBlockResultHandling::Silent,
        "none" => options.handling = SourceBlockResultHandling::None,
        "discard" => options.handling = SourceBlockResultHandling::Discard,
        "append" => options.handling = SourceBlockResultHandling::Append,
        "prepend" => options.handling = SourceBlockResultHandling::Prepend,
        "output" => options.value_type = SourceBlockResultValueType::Output,
        "value" => options.value_type = SourceBlockResultValueType::Value,
        _ => push_unknown_result_token(options, token),
    }
}

fn push_unknown_result_token(options: &mut SourceBlockResultOptions, token: &str) {
    if !options.unknown.iter().any(|unknown| unknown == token) {
        options.unknown.push(token.to_string());
    }
}

fn source_block_result_file(header_args: &[SourceBlockHeaderArg]) -> Option<SourceBlockResultFile> {
    last_normalized_header_value(header_args, "file")
        .filter(|target| !target.is_empty())
        .map(|target| SourceBlockResultFile {
            target,
            description: last_normalized_header_value(header_args, "file-desc")
                .filter(|value| !value.is_empty()),
            extension: last_normalized_header_value(header_args, "file-ext")
                .filter(|value| !value.is_empty()),
            file_mode: last_normalized_header_value(header_args, "file-mode")
                .filter(|value| !value.is_empty())
                .map(|raw| SourceBlockResultFileMode { raw }),
            output_dir: last_normalized_header_value(header_args, "output-dir")
                .filter(|value| !value.is_empty()),
        })
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

fn last_normalized_header_value(header_args: &[SourceBlockHeaderArg], key: &str) -> Option<String> {
    header_arg_normalized_value(last_normalized_header_arg(header_args, key))
}

fn header_value(header_args: &[BlockHeaderArg], key: &str) -> Option<String> {
    last_header_arg(header_args, key)
        .and_then(|arg| arg.value.as_deref())
        .map(str::trim)
        .map(unquote_header_value)
}

fn last_header_arg<'a>(header_args: &'a [BlockHeaderArg], key: &str) -> Option<&'a BlockHeaderArg> {
    header_args
        .iter()
        .rev()
        .find(|arg| arg.key.eq_ignore_ascii_case(key))
}

pub(super) fn source_block_header_args(
    kind: SourceBlockRecordKind,
    header_args: &[BlockHeaderArg],
) -> Vec<SourceBlockHeaderArg> {
    let mut normalized = default_source_block_header_args(kind);
    for arg in header_args {
        let projected = source_block_header_arg(arg, SourceBlockHeaderArgSource::Explicit);
        if matches!(
            projected.kind,
            SourceBlockHeaderArgKind::Var | SourceBlockHeaderArgKind::Results
        ) {
            normalized.push(projected);
        } else if let Some(existing) = normalized
            .iter_mut()
            .find(|existing| existing.key.eq_ignore_ascii_case(&projected.key))
        {
            *existing = projected;
        } else {
            normalized.push(projected);
        }
    }
    normalized
}

fn default_source_block_header_args(kind: SourceBlockRecordKind) -> Vec<SourceBlockHeaderArg> {
    let kind = match kind {
        SourceBlockRecordKind::Block => "block",
        SourceBlockRecordKind::InlineSource => "inline",
    };
    let defaults = super::org_native_values::rows("header-defaults", &[kind]);
    defaults
        .into_iter()
        .map(|row| {
            let [key, value]: [String; 2] = row.try_into().expect("native header default arity");
            let raw = format!(":{key} {value}");
            let arg = BlockHeaderArg {
                key: key.to_string(),
                value: Some(value.to_string()),
                raw,
            };
            source_block_header_arg(&arg, SourceBlockHeaderArgSource::Default)
        })
        .collect()
}

fn source_block_header_arg(
    arg: &BlockHeaderArg,
    source: SourceBlockHeaderArgSource,
) -> SourceBlockHeaderArg {
    let kind = source_block_header_arg_kind(&arg.key);
    let tokens = arg
        .value
        .as_deref()
        .map(split_header_value)
        .unwrap_or_default();
    SourceBlockHeaderArg {
        key: arg.key.clone(),
        value: arg.value.clone(),
        raw: arg.raw.clone(),
        kind,
        source,
        variable: (kind == SourceBlockHeaderArgKind::Var)
            .then_some(arg.value.as_deref())
            .flatten()
            .map(source_block_header_var),
        tokens,
    }
}

fn source_block_header_arg_kind(key: &str) -> SourceBlockHeaderArgKind {
    match header_policy("kind", key).as_str() {
        "cache" => SourceBlockHeaderArgKind::Cache,
        "dir" => SourceBlockHeaderArgKind::Dir,
        "eval" => SourceBlockHeaderArgKind::Eval,
        "exports" => SourceBlockHeaderArgKind::Exports,
        "file" => SourceBlockHeaderArgKind::File,
        "file-desc" => SourceBlockHeaderArgKind::FileDesc,
        "file-ext" => SourceBlockHeaderArgKind::FileExt,
        "file-mode" => SourceBlockHeaderArgKind::FileMode,
        "format" => SourceBlockHeaderArgKind::Format,
        "hlines" => SourceBlockHeaderArgKind::Hlines,
        "lint" => SourceBlockHeaderArgKind::Lint,
        "noweb" => SourceBlockHeaderArgKind::Noweb,
        "output-dir" => SourceBlockHeaderArgKind::OutputDir,
        "results" => SourceBlockHeaderArgKind::Results,
        "runtime" => SourceBlockHeaderArgKind::Runtime,
        "session" => SourceBlockHeaderArgKind::Session,
        "tangle" => SourceBlockHeaderArgKind::Tangle,
        "var" => SourceBlockHeaderArgKind::Var,
        _ => SourceBlockHeaderArgKind::Other,
    }
}

fn source_block_header_var(value: &str) -> SourceBlockHeaderVar {
    let row = super::org_native_values::rows("source-variable", &[value])
        .pop()
        .expect("native variable row");
    let [name, assignment, present]: [String; 3] = row.try_into().expect("native variable arity");
    SourceBlockHeaderVar {
        name,
        assignment: match present.as_str() {
            "true" => Some(assignment),
            "false" => None,
            _ => panic!("native variable presence"),
        },
    }
}
fn split_header_value(value: &str) -> Vec<String> {
    super::org_native_values::rows("header-tokens", &[value])
        .pop()
        .expect("native header token row")
}
fn unquote_header_value(value: &str) -> String {
    super::org_native_values::scalar("source-unquote", &[value])
}
fn header_policy(mode: &str, value: &str) -> String {
    super::org_native_values::scalar("header-policy", &[mode, value])
}
