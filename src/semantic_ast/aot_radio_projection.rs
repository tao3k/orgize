//! Typed AST projection of matches selected by the Scheme-AOT source matcher.
//! This module does not search Org text or decide word boundaries/priority.

use rowan::TextRange;

use super::{GraphProjector, Link, LinkDescriptionState, LinkMediaKind, LinkPath, LinkTarget};
use crate::{
    ast::{Object, ObjectData, ParsedAnnotation},
    config::RadioLinkProjection,
};

#[path = "aot_radio_generated_matcher.rs"]
mod generated_matcher;

#[derive(Clone, Copy)]
struct ObjectSpan {
    start: usize,
    end: usize,
}

impl GraphProjector<'_> {
    pub(super) fn project_radio_links(
        &self,
        objects: Vec<Object<ParsedAnnotation>>,
    ) -> Vec<Object<ParsedAnnotation>> {
        if self.radio_targets.is_empty() {
            return objects;
        }
        match self.document.config().radio_link_projection {
            RadioLinkProjection::PlainText => self.project_plain_radio_links(objects),
            RadioLinkProjection::Semantic => self.project_semantic_radio_links(objects),
        }
    }

    fn project_plain_radio_links(
        &self,
        objects: Vec<Object<ParsedAnnotation>>,
    ) -> Vec<Object<ParsedAnnotation>> {
        let mut projected = Vec::with_capacity(objects.len());
        for object in objects {
            match object {
                Object {
                    ann,
                    data: ObjectData::Plain(value),
                } => self.extend_plain_radio_links(&mut projected, ann, value),
                other => projected.push(other),
            }
        }
        projected
    }

    fn extend_plain_radio_links(
        &self,
        projected: &mut Vec<Object<ParsedAnnotation>>,
        ann: ParsedAnnotation,
        value: String,
    ) {
        let mut cursor = 0;
        let base = usize::from(ann.range.start());
        while let Some((start, end, index)) =
            generated_matcher::org_radio_next_match(&value, cursor, &self.radio_targets)
        {
            if cursor < start {
                projected.push(self.plain_source_span(base + cursor, base + start));
            }
            let description = vec![self.plain_source_span(base + start, base + end)];
            projected.push(self.radio_link(base + start, base + end, index, description));
            cursor = end;
        }
        if cursor == 0 {
            projected.push(Object {
                ann,
                data: ObjectData::Plain(value),
            });
        } else if cursor < value.len() {
            projected.push(self.plain_source_span(base + cursor, base + value.len()));
        }
    }

    fn project_semantic_radio_links(
        &self,
        objects: Vec<Object<ParsedAnnotation>>,
    ) -> Vec<Object<ParsedAnnotation>> {
        let mut projected = Vec::with_capacity(objects.len());
        let mut run = Vec::new();
        for object in objects {
            if matches!(
                &object.data,
                ObjectData::Plain(_)
                    | ObjectData::Markup { .. }
                    | ObjectData::Code(_)
                    | ObjectData::Verbatim(_)
                    | ObjectData::Entity(_)
                    | ObjectData::LatexFragment(_)
            ) {
                run.push(object);
            } else {
                projected.extend(self.project_radio_run(std::mem::take(&mut run)));
                projected.push(object);
            }
        }
        projected.extend(self.project_radio_run(run));
        projected
    }

    fn project_radio_run(
        &self,
        objects: Vec<Object<ParsedAnnotation>>,
    ) -> Vec<Object<ParsedAnnotation>> {
        let Some(first) = objects.first() else {
            return objects;
        };
        let base = usize::from(first.ann.range.start());
        let raw = objects
            .iter()
            .map(|object| object.ann.raw.as_str())
            .collect::<String>();
        let spans = object_spans(&objects);
        let mut projected = Vec::with_capacity(objects.len());
        let mut emitted_until = 0;
        let mut search_cursor = 0;

        while let Some((start, end, index)) =
            generated_matcher::org_radio_next_match(&raw, search_cursor, &self.radio_targets)
        {
            if start < emitted_until {
                search_cursor = end;
                continue;
            }
            let Some(description) = self.slice_radio_objects(&objects, &spans, base, start, end)
            else {
                search_cursor = next_char_boundary(&raw, start);
                continue;
            };
            let Some(prefix) =
                self.slice_radio_objects(&objects, &spans, base, emitted_until, start)
            else {
                return objects;
            };
            projected.extend(prefix);
            projected.push(self.radio_link(base + start, base + end, index, description));
            emitted_until = end;
            search_cursor = end;
        }
        if emitted_until == 0 {
            return objects;
        }
        let Some(suffix) =
            self.slice_radio_objects(&objects, &spans, base, emitted_until, raw.len())
        else {
            return objects;
        };
        projected.extend(suffix);
        projected
    }

    fn slice_radio_objects(
        &self,
        objects: &[Object<ParsedAnnotation>],
        spans: &[ObjectSpan],
        base: usize,
        start: usize,
        end: usize,
    ) -> Option<Vec<Object<ParsedAnnotation>>> {
        if start == end {
            return Some(Vec::new());
        }
        let first = spans.partition_point(|span| span.end <= start);
        spans[first..]
            .iter()
            .zip(&objects[first..])
            .take_while(|(span, _)| span.start < end)
            .map(|(span, object)| self.slice_radio_object(object, *span, base, start, end))
            .collect()
    }

    fn slice_radio_object(
        &self,
        object: &Object<ParsedAnnotation>,
        span: ObjectSpan,
        base: usize,
        start: usize,
        end: usize,
    ) -> Option<Object<ParsedAnnotation>> {
        let slice_start = start.max(span.start);
        let slice_end = end.min(span.end);
        if slice_start == span.start && slice_end == span.end {
            return Some(object.clone());
        }
        let ObjectData::Plain(value) = &object.data else {
            return None;
        };
        let relative_start = slice_start - span.start;
        let relative_end = slice_end - span.start;
        let raw = value.get(relative_start..relative_end)?.to_owned();
        Some(Object {
            ann: self.annotation(text_range(base + slice_start, base + slice_end)),
            data: ObjectData::Plain(raw),
        })
    }

    fn radio_link(
        &self,
        start: usize,
        end: usize,
        target_index: usize,
        description: Vec<Object<ParsedAnnotation>>,
    ) -> Object<ParsedAnnotation> {
        let target = self.radio_targets[target_index].clone();
        let ann = self.annotation(text_range(start, end));
        Object {
            ann: ann.clone(),
            data: ObjectData::Link(Box::new(Link {
                path: LinkPath::new(target.clone()),
                target: LinkTarget::Internal(target),
                description,
                default_description: Vec::new(),
                raw_description: ann.raw,
                description_state: LinkDescriptionState::Explicit,
                media_kind: LinkMediaKind::Normal,
                caption: None,
                search: None,
                attachment: None,
                file: None,
            })),
        }
    }

    fn plain_source_span(&self, start: usize, end: usize) -> Object<ParsedAnnotation> {
        let range = text_range(start, end);
        self.plain(range, self.raw(range))
    }
}

fn object_spans(objects: &[Object<ParsedAnnotation>]) -> Vec<ObjectSpan> {
    objects
        .iter()
        .scan(0, |cursor, object| {
            let start = *cursor;
            *cursor += object.ann.raw.len();
            Some(ObjectSpan {
                start,
                end: *cursor,
            })
        })
        .collect()
}

fn next_char_boundary(source: &str, start: usize) -> usize {
    source
        .get(start..)
        .and_then(|tail| tail.chars().next())
        .map_or(source.len(), |character| start + character.len_utf8())
}

fn text_range(start: usize, end: usize) -> TextRange {
    TextRange::new((start as u32).into(), (end as u32).into())
}
