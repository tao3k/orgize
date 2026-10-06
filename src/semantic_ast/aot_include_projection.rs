//! Source-backed INCLUDE fields projected from the Scheme-owned event graph.

use super::{
    Diagnostic, DiagnosticKind, GraphProjector, IncludeDirective, IncludeOption, Keyword,
    ParsedAnnotation,
};

use crate::ast::aot_block_switches::project_keyword_header_args;

impl GraphProjector<'_> {
    pub(super) fn include_directive(
        &self,
        id: usize,
        keyword: Keyword<ParsedAnnotation>,
    ) -> Result<IncludeDirective<ParsedAnnotation>, Diagnostic> {
        let record = self.record(id);
        let path = record.field("include-path").ok_or_else(|| Diagnostic {
            range: keyword.ann.range,
            kind: DiagnosticKind::Conversion,
            message: "INCLUDE keyword requires a valid path".to_owned(),
        })?;
        let raw_path = record.field("include-raw-path").unwrap_or(path);
        let arguments = record
            .values("include-argument")
            .map(str::to_owned)
            .collect();
        let options = project_keyword_header_args(record)
            .into_iter()
            .map(|argument| IncludeOption {
                key: argument.key,
                value: argument.value,
                raw: argument.raw,
            })
            .collect();
        Ok(IncludeDirective {
            ann: keyword.ann,
            path: path.to_owned(),
            raw_path: raw_path.to_owned(),
            arguments,
            options,
            raw_value: keyword.value,
        })
    }
}
