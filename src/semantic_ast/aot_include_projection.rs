//! Source-backed INCLUDE fields projected from the Scheme-owned event graph.

use super::{
    Diagnostic, DiagnosticKind, GraphProjector, IncludeDirective, IncludeOption, Keyword,
    ParsedAnnotation, TextRange,
};

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
        let mut options = Vec::new();
        let mut fields = record
            .fields
            .iter()
            .filter(|field| matches!(field.name, "include-option-key" | "include-option-value"))
            .collect::<Vec<_>>();
        fields.sort_unstable_by_key(|field| field.range.start());
        for (index, key) in fields.iter().enumerate() {
            if key.name != "include-option-key" {
                continue;
            }
            let value = fields
                .get(index + 1)
                .filter(|field| field.name == "include-option-value");
            let start = u32::from(key.range.start()).saturating_sub(1);
            let end = value.map_or(key.range.end(), |value| value.range.end());
            let raw = self.raw(TextRange::new(start.into(), end)).to_owned();
            options.push(IncludeOption {
                key: key.value.clone(),
                value: value.map(|value| {
                    value
                        .value
                        .strip_prefix('"')
                        .and_then(|quoted| quoted.strip_suffix('"'))
                        .unwrap_or(&value.value)
                        .to_owned()
                }),
                raw,
            });
        }
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
