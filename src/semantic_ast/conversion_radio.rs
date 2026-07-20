//! Semantic radio-link projection and object-run slicing helpers.

use super::conversion::Converter;
use super::conversion_util::{range_from_elements, text_range};
use super::radio_links::{is_semantic_radio_link_candidate, next_char_boundary, next_radio_link};
use super::{
    Element, ElementData, Link, LinkDescriptionState, LinkMediaKind, LinkPath, LinkTarget, Object,
    ObjectData, ParsedAnnotation,
};
use crate::{config::RadioLinkProjection, syntax::SyntaxElement};

impl<'a> Converter<'a> {
    pub(super) fn paragraph_from_elements(
        &mut self,
        elements: Vec<SyntaxElement>,
    ) -> Option<Element<ParsedAnnotation>> {
        let range = range_from_elements(&elements)?;
        let objects = self.objects_from_elements(elements);

        Some(Element {
            ann: self.ann(range),
            affiliated_keywords: Vec::new(),
            data: ElementData::Paragraph(objects),
        })
    }

    pub(super) fn objects_from_elements(
        &mut self,
        elements: impl IntoIterator<Item = SyntaxElement>,
    ) -> Vec<Object<ParsedAnnotation>> {
        let objects = elements
            .into_iter()
            .filter_map(|element| self.object(element))
            .collect();
        self.project_radio_links(objects)
    }

    pub(super) fn project_radio_links(
        &self,
        objects: Vec<Object<ParsedAnnotation>>,
    ) -> Vec<Object<ParsedAnnotation>> {
        if self.radio_targets.is_empty() {
            return objects;
        }

        match self.config.radio_link_projection {
            RadioLinkProjection::PlainText => self.project_plain_text_radio_links(objects),
            RadioLinkProjection::Semantic => self.project_semantic_radio_links(objects),
        }
    }

    pub(super) fn project_plain_text_radio_links(
        &self,
        objects: Vec<Object<ParsedAnnotation>>,
    ) -> Vec<Object<ParsedAnnotation>> {
        let mut projected = Vec::with_capacity(objects.len());

        for object in objects {
            match object {
                Object {
                    ann,
                    data: ObjectData::Plain(value),
                } => self.extend_radio_links_in_plain(&mut projected, ann, value),
                _ => projected.push(object),
            }
        }

        projected
    }

    pub(super) fn project_semantic_radio_links(
        &self,
        objects: Vec<Object<ParsedAnnotation>>,
    ) -> Vec<Object<ParsedAnnotation>> {
        let capacity = objects.len();
        objects
            .into_iter()
            .fold(
                SemanticRadioProjection::new(self, capacity),
                SemanticRadioProjection::push,
            )
            .finish()
    }

    pub(super) fn project_radio_links_in_object_run(
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
        let spans = object_run_spans(&objects);
        let mut projected = Vec::with_capacity(objects.len());
        let mut emitted_until = 0;
        let mut search_cursor = 0;

        while let Some((start, end, target)) =
            next_radio_link(&raw, search_cursor, &self.radio_targets)
        {
            if start < emitted_until {
                search_cursor = end;
                continue;
            }

            let Some(description) =
                self.slice_radio_link_objects(&objects, &spans, base, start, end)
            else {
                search_cursor = next_char_boundary(&raw, start);
                continue;
            };
            let Some(prefix) =
                self.slice_radio_link_objects(&objects, &spans, base, emitted_until, start)
            else {
                return objects;
            };

            projected.extend(prefix);

            let raw_description = raw[start..end].to_string();
            let link_ann = self.ann(text_range(base + start, base + end));
            projected.push(Object {
                ann: link_ann,
                data: ObjectData::Link(Box::new(Link {
                    path: LinkPath::new(target.to_string()),
                    target: LinkTarget::Internal(target.to_string()),
                    description,
                    default_description: Vec::new(),
                    raw_description,
                    description_state: LinkDescriptionState::Explicit,
                    media_kind: LinkMediaKind::Normal,
                    caption: None,
                    search: None,
                    attachment: None,
                    file: None,
                })),
            });

            emitted_until = end;
            search_cursor = end;
        }

        if emitted_until == 0 {
            return objects;
        }

        let Some(suffix) =
            self.slice_radio_link_objects(&objects, &spans, base, emitted_until, raw.len())
        else {
            return objects;
        };
        projected.extend(suffix);
        projected
    }

    pub(super) fn slice_radio_link_objects(
        &self,
        objects: &[Object<ParsedAnnotation>],
        spans: &[ObjectRunSpan],
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
            .map(|(span, object)| self.slice_radio_link_object(object, *span, base, start, end))
            .collect()
    }

    pub(super) fn slice_radio_link_object(
        &self,
        object: &Object<ParsedAnnotation>,
        span: ObjectRunSpan,
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
        let raw = value.get(relative_start..relative_end)?.to_string();
        Some(Object {
            ann: self.ann(text_range(base + slice_start, base + slice_end)),
            data: ObjectData::Plain(raw),
        })
    }

    pub(super) fn extend_radio_links_in_plain(
        &self,
        objects: &mut Vec<Object<ParsedAnnotation>>,
        ann: ParsedAnnotation,
        value: String,
    ) {
        let mut cursor = 0;
        let base = usize::from(ann.range.start());

        while let Some((start, end, target)) = next_radio_link(&value, cursor, &self.radio_targets)
        {
            if cursor < start {
                objects.push(Object {
                    ann: self.ann(text_range(base + cursor, base + start)),
                    data: ObjectData::Plain(value[cursor..start].to_string()),
                });
            }

            let raw = value[start..end].to_string();
            let link_ann = self.ann(text_range(base + start, base + end));
            objects.push(Object {
                ann: link_ann.clone(),
                data: ObjectData::Link(Box::new(Link {
                    path: LinkPath::new(target.to_string()),
                    target: LinkTarget::Internal(target.to_string()),
                    description: vec![Object {
                        ann: link_ann,
                        data: ObjectData::Plain(raw.clone()),
                    }],
                    default_description: Vec::new(),
                    raw_description: raw,
                    description_state: LinkDescriptionState::Explicit,
                    media_kind: LinkMediaKind::Normal,
                    caption: None,
                    search: None,
                    attachment: None,
                    file: None,
                })),
            });

            cursor = end;
        }

        if cursor == 0 {
            objects.push(Object {
                ann,
                data: ObjectData::Plain(value),
            });
            return;
        }

        if cursor < value.len() {
            objects.push(Object {
                ann: self.ann(text_range(base + cursor, base + value.len())),
                data: ObjectData::Plain(value[cursor..].to_string()),
            });
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct ObjectRunSpan {
    start: usize,
    end: usize,
}

pub(super) struct SemanticRadioProjection<'converter, 'source> {
    converter: &'converter Converter<'source>,
    projected: Vec<Object<ParsedAnnotation>>,
    run: Vec<Object<ParsedAnnotation>>,
}

impl<'converter, 'source> SemanticRadioProjection<'converter, 'source> {
    fn new(converter: &'converter Converter<'source>, capacity: usize) -> Self {
        Self {
            converter,
            projected: Vec::with_capacity(capacity),
            run: Vec::new(),
        }
    }

    fn push(mut self, object: Object<ParsedAnnotation>) -> Self {
        if is_semantic_radio_link_candidate(&object.data) {
            self.run.push(object);
        } else {
            self.flush();
            self.projected.push(object);
        }
        self
    }

    fn finish(mut self) -> Vec<Object<ParsedAnnotation>> {
        self.flush();
        self.projected
    }

    fn flush(&mut self) {
        if !self.run.is_empty() {
            self.projected.extend(
                self.converter
                    .project_radio_links_in_object_run(std::mem::take(&mut self.run)),
            );
        }
    }
}

pub(super) fn object_run_spans(objects: &[Object<ParsedAnnotation>]) -> Vec<ObjectRunSpan> {
    objects
        .iter()
        .scan(0, |cursor, object| {
            let start = *cursor;
            *cursor += object.ann.raw.len();
            Some(ObjectRunSpan {
                start,
                end: *cursor,
            })
        })
        .collect()
}
