//! Annotation, raw text, object parsing, and diagnostic helpers for conversion.

use super::conversion::Converter;
use super::conversion_util::{position_range, trimmed_range};
use super::{Diagnostic, DiagnosticKind, Object, ParsedAnnotation};
use crate::syntax::{
    SyntaxKind, SyntaxNode, SyntaxToken,
    combinator::node,
    object::{minimal_object_nodes, standard_object_nodes},
};
use rowan::TextRange;

impl<'a> Converter<'a> {
    pub(super) fn node_ann(&self, node: &SyntaxNode) -> ParsedAnnotation {
        self.ann(node.text_range())
    }

    pub(super) fn token_ann(&self, token: &SyntaxToken) -> ParsedAnnotation {
        self.ann(token.text_range())
    }

    pub(super) fn ann(&self, range: TextRange) -> ParsedAnnotation {
        let start = self.lines.position(range.start());
        let end = self.lines.position(range.end());
        let raw = self.raw(range).to_string();
        ParsedAnnotation {
            range,
            start,
            end,
            raw,
        }
    }

    pub(super) fn raw(&self, range: TextRange) -> &str {
        let start: usize = range.start().into();
        let end: usize = range.end().into();
        self.source.get(start..end).unwrap_or_default()
    }

    pub(super) fn objects_from_raw(
        &mut self,
        value: &str,
        absolute_start: usize,
    ) -> Vec<Object<ParsedAnnotation>> {
        self.objects_from_raw_with(value, absolute_start, false)
    }

    pub(super) fn objects_from_raw_minimal(
        &mut self,
        value: &str,
        absolute_start: usize,
    ) -> Vec<Object<ParsedAnnotation>> {
        self.objects_from_raw_with(value, absolute_start, true)
    }

    pub(super) fn objects_from_raw_with(
        &mut self,
        value: &str,
        absolute_start: usize,
        minimal: bool,
    ) -> Vec<Object<ParsedAnnotation>> {
        let Some((start, end)) = trimmed_range(value) else {
            return Vec::new();
        };
        let raw = &value[start..end];
        let base = absolute_start + start;
        let children = if minimal {
            minimal_object_nodes((raw, self.config).into())
        } else {
            standard_object_nodes((raw, self.config).into())
        };
        let root = SyntaxNode::new_root(
            node(SyntaxKind::PARAGRAPH, children)
                .into_node()
                .expect("paragraph node"),
        );
        let mut converter = Converter::new(raw, self.config);
        let objects = converter.objects_from_elements(root.children_with_tokens());
        self.diagnostics.extend(
            converter
                .diagnostics
                .into_iter()
                .map(|diagnostic| Diagnostic {
                    range: position_range(diagnostic.range, base),
                    kind: diagnostic.kind,
                    message: diagnostic.message,
                }),
        );
        let mut map_ann = |ann: &ParsedAnnotation| self.ann(position_range(ann.range, base));

        objects
            .iter()
            .map(|object| object.map_ann_with(&mut map_ann))
            .collect()
    }

    pub(super) fn diagnostic(&mut self, range: TextRange, kind: DiagnosticKind, message: String) {
        self.diagnostics.push(Diagnostic {
            range,
            kind,
            message,
        });
    }
}
