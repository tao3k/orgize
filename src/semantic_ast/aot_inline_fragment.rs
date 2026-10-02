//! Project source-backed inline fragments through the same Scheme-AOT parser.

use rowan::TextRange;

use crate::org_aot::parse_org_aot_with_config;

use super::{GraphProjector, Object, ParsedAnnotation};

impl GraphProjector<'_> {
    /// Consume source-bounded child Objects unless Scheme marked a recursive
    /// fragment that still needs the bounded AOT fragment parser.
    pub(super) fn graph_fragment(
        &mut self,
        id: usize,
        field: &str,
    ) -> Vec<Object<ParsedAnnotation>> {
        let record = self.record(id);
        let Some(span) = record.field_range(field) else {
            return Vec::new();
        };
        let fallback = record.field("fragment-fallback").is_some();
        let children = record.child_ids.clone();
        if fallback {
            self.inline_fragment(span)
        } else {
            self.objects_in_span(span, &children)
        }
    }

    pub(super) fn inline_fragment(&self, span: TextRange) -> Vec<Object<ParsedAnnotation>> {
        let source = self.raw(span);
        let Ok(parsed) = parse_org_aot_with_config(source, self.document.config()) else {
            return vec![self.plain(span, source)];
        };
        let paragraphs = parsed
            .records()
            .iter()
            .filter(|record| record.parent_id == Some(0) && record.kind == "paragraph")
            .map(|record| record.id)
            .collect::<Vec<_>>();
        let mut projector = GraphProjector::new(&parsed, source);
        let objects = paragraphs
            .into_iter()
            .flat_map(|id| projector.paragraph_objects(id))
            .collect::<Vec<_>>();
        let mut relocate = |annotation: &ParsedAnnotation| {
            let start = usize::from(span.start()) + usize::from(annotation.range.start());
            let end = usize::from(span.start()) + usize::from(annotation.range.end());
            let mut relocated =
                self.annotation(TextRange::new((start as u32).into(), (end as u32).into()));
            relocated.header_args = annotation.header_args.clone();
            relocated
        };
        if objects.is_empty() {
            vec![self.plain(span, source)]
        } else {
            objects
                .iter()
                .map(|object| object.map_ann_with(&mut relocate))
                .collect()
        }
    }
}
