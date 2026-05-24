//! Org Babel eval planning and result patch rendering.

use super::{
    BabelEvalOutput, BabelEvalPlan, BabelEvalPlanError, BabelEvalResultPatch,
    BabelEvalResultPatchKind, BabelEvalResultRange, Document, ParsedAnnotation, SourceBlockRecord,
    SourceBlockRecordKind, SourceBlockResultFormat, SourceBlockResultHandling,
};

impl Document<ParsedAnnotation> {
    /// Builds an execution contract for one named source block.
    ///
    /// This method does not execute source code. It only resolves the named
    /// Babel block and returns parser-owned metadata that a downstream runner
    /// can use.
    ///
    /// # Errors
    ///
    /// Returns an error when `name` is empty, absent, or ambiguous.
    pub fn babel_eval_plan(&self, name: &str) -> Result<BabelEvalPlan, BabelEvalPlanError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(BabelEvalPlanError::EmptyName);
        }

        let matches = self
            .source_block_records()
            .into_iter()
            .filter(|record| {
                record.kind == SourceBlockRecordKind::Block
                    && record.name.as_deref().is_some_and(|value| value == name)
            })
            .collect::<Vec<_>>();

        match matches.len() {
            0 => Err(BabelEvalPlanError::NotFound {
                name: name.to_string(),
            }),
            1 => Ok(BabelEvalPlan {
                name: name.to_string(),
                record: matches.into_iter().next().expect("one match"),
            }),
            count => Err(BabelEvalPlanError::Ambiguous {
                name: name.to_string(),
                matches: count,
            }),
        }
    }
}

impl BabelEvalPlan {
    /// Renders the host-supplied output as an Org `#+RESULTS:` patch.
    #[must_use]
    pub fn result_patch(&self, source: &str, output: &BabelEvalOutput) -> BabelEvalResultPatch {
        let handling = self.record.result_options.handling;
        if matches!(
            handling,
            SourceBlockResultHandling::Silent
                | SourceBlockResultHandling::None
                | SourceBlockResultHandling::Discard
        ) {
            return BabelEvalResultPatch {
                kind: BabelEvalResultPatchKind::Noop,
                range: None,
                replacement: String::new(),
                handling,
                message: Some(format!(
                    ":results {} does not write Org results",
                    handling_label(handling)
                )),
            };
        }

        let value = result_value_for_handling(&self.record, output, handling);
        let rendered = render_result_block(&self.name, &self.record, value.as_str());
        if let Some(result) = &self.record.result {
            BabelEvalResultPatch {
                kind: BabelEvalResultPatchKind::Replace,
                range: Some(BabelEvalResultRange {
                    start: result.source.range_start,
                    end: result.source.range_end,
                }),
                replacement: rendered,
                handling,
                message: None,
            }
        } else {
            let offset = self.record.source.range_end;
            let prefix = insertion_prefix(source, offset);
            BabelEvalResultPatch {
                kind: BabelEvalResultPatchKind::Insert,
                range: Some(BabelEvalResultRange {
                    start: offset,
                    end: offset,
                }),
                replacement: format!("{prefix}{rendered}"),
                handling,
                message: None,
            }
        }
    }
}

fn result_value_for_handling(
    record: &SourceBlockRecord,
    output: &BabelEvalOutput,
    handling: SourceBlockResultHandling,
) -> String {
    let next = primary_output(output);
    let existing = record
        .result
        .as_ref()
        .map(|result| result.value.trim_end())
        .filter(|value| !value.is_empty());
    match (handling, existing) {
        (SourceBlockResultHandling::Append, Some(existing)) if !next.is_empty() => {
            format!("{existing}\n{next}")
        }
        (SourceBlockResultHandling::Prepend, Some(existing)) if !next.is_empty() => {
            format!("{next}\n{existing}")
        }
        (
            SourceBlockResultHandling::Append | SourceBlockResultHandling::Prepend,
            Some(existing),
        ) => existing.to_string(),
        _ => next,
    }
}

fn primary_output(output: &BabelEvalOutput) -> String {
    if output.stdout.is_empty() {
        output.stderr.trim_end().to_string()
    } else {
        output.stdout.trim_end().to_string()
    }
}

fn render_result_block(name: &str, record: &SourceBlockRecord, value: &str) -> String {
    let mut rendered = format!("#+RESULTS: {name}\n");
    if matches!(
        record.result_options.format,
        Some(SourceBlockResultFormat::Raw | SourceBlockResultFormat::Org)
    ) {
        rendered.push_str(value);
        if !rendered.ends_with('\n') {
            rendered.push('\n');
        }
        return rendered;
    }

    if value.is_empty() {
        rendered.push_str(":\n");
        return rendered;
    }
    for line in value.lines() {
        rendered.push_str(": ");
        rendered.push_str(line);
        rendered.push('\n');
    }
    rendered
}

fn insertion_prefix(source: &str, offset: u32) -> &'static str {
    let offset = usize::try_from(offset)
        .unwrap_or(usize::MAX)
        .min(source.len());
    let prefix = &source[..offset];
    if prefix.ends_with("\n\n") {
        ""
    } else if prefix.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    }
}

fn handling_label(handling: SourceBlockResultHandling) -> &'static str {
    match handling {
        SourceBlockResultHandling::Replace => "replace",
        SourceBlockResultHandling::Silent => "silent",
        SourceBlockResultHandling::None => "none",
        SourceBlockResultHandling::Discard => "discard",
        SourceBlockResultHandling::Append => "append",
        SourceBlockResultHandling::Prepend => "prepend",
    }
}
