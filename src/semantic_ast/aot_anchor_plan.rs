//! Typed application of the native document anchor owner, before link lookup.
use super::{ParsedAnnotation, Section};
use std::collections::HashMap;

pub(super) fn apply_document_anchors(
    sections: &mut [Section<ParsedAnnotation>],
    inputs: Vec<(u32, String)>,
) {
    if inputs.is_empty() {
        return;
    }
    let count = inputs.len().to_string();
    let mut fields = vec!["headline-document-anchors", &count];
    fields.extend(inputs.iter().map(|(_, title)| title.as_str()));
    let mut rows =
        crate::org_aot::native_semantic_rows(16, &fields).expect("native document anchors");
    assert_eq!(rows.len(), 1, "native document anchor row count");
    let anchors = rows.pop().unwrap();
    assert_eq!(anchors.len(), inputs.len(), "native document anchor count");
    let mut admitted = inputs
        .into_iter()
        .zip(anchors)
        .map(|((start, _), anchor)| (start, anchor))
        .collect::<HashMap<_, _>>();
    fn apply(sections: &mut [Section<ParsedAnnotation>], admitted: &mut HashMap<u32, String>) {
        for section in sections {
            if let Some(anchor) = admitted.remove(&u32::from(section.ann.range.start())) {
                section.anchor = (!anchor.is_empty()).then_some(anchor);
            }
            apply(&mut section.subsections, admitted);
        }
    }
    apply(sections, &mut admitted);
    assert!(
        admitted.is_empty(),
        "native anchors belong to projected sections"
    );
}
