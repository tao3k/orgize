//! Owned Citation values copied from native Scheme graph fields.
//! Header classification and affix content bounds belong to Scheme.

use super::{
    Citation, CiteReference, Diagnostic, DiagnosticKind, GraphProjector, Object, ParsedAnnotation,
    TextRange,
};

impl GraphProjector<'_> {
    pub(super) fn citation(&mut self, id: usize) -> Citation<ParsedAnnotation> {
        let (prefix_range, suffix_range, style, variant, children) = {
            let record = self.record(id);
            (
                record.field_range("global-prefix-content"),
                record.field_range("global-suffix-content"),
                // Absent native style/variant fields decode the public AST's
                // no-style sentinel; no citation source syntax is inspected.
                record.field("style").unwrap_or("nil").to_owned(),
                record.field("variant").unwrap_or_default().to_owned(),
                record.child_ids.clone(),
            )
        };
        let prefix_objects = self.citation_affix(prefix_range, &children);
        let suffix_objects = self.citation_affix(suffix_range, &children);
        let mut references = Vec::new();
        for child in children {
            let (ref_range, key, ref_prefix, ref_suffix, ref_children) = {
                let record = self.record(child);
                if record.kind == "citation-malformed" {
                    self.diagnostics.push(Diagnostic {
                        range: record.range,
                        kind: DiagnosticKind::Conversion,
                        message: "malformed citation segment".to_owned(),
                    });
                    continue;
                }
                if record.kind != "citation-reference" {
                    continue;
                }
                (
                    record.range,
                    record.field("key").unwrap_or_default().to_owned(),
                    record.field_range("prefix-content"),
                    record.field_range("suffix-content"),
                    record.child_ids.clone(),
                )
            };
            references.push(CiteReference {
                ann: self.annotation(ref_range),
                id: key,
                prefix: self.citation_affix(ref_prefix, &ref_children),
                suffix: self.citation_affix(ref_suffix, &ref_children),
            });
        }
        Citation {
            style,
            variant,
            prefix: prefix_objects,
            suffix: suffix_objects,
            references,
        }
    }

    fn citation_affix(
        &mut self,
        range: Option<TextRange>,
        children: &[usize],
    ) -> Vec<Object<ParsedAnnotation>> {
        // Absence is Scheme's explicit whitespace-only/no-affix projection.
        range.map_or_else(Vec::new, |range| self.objects_in_span(range, children))
    }
}
