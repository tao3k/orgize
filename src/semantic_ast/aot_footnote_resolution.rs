//! Document-level footnote registry from Scheme-classified Elements and Objects.

use std::collections::{HashMap, HashSet};

use gerbil_parser_runtime::TextRange;

use super::{
    AstMut, Document, ElementData, FootnoteDefinition, FootnoteEntry, ObjectData, ParsedAnnotation,
};

pub(super) fn resolve_document_footnotes(document: &mut Document<ParsedAnnotation>) {
    let mut entries = Vec::new();
    let mut definitions = HashSet::new();
    let mut anonymous = HashMap::<TextRange, String>::new();
    document.visit_mut(|node| match node {
        AstMut::Element(element) => {
            if let ElementData::FootnoteDef(definition) = &element.data
                && definitions.insert(definition.label.clone())
            {
                entries.push(FootnoteEntry {
                    ann: element.ann.clone(),
                    label: definition.label.clone(),
                    definition: FootnoteDefinition::Standalone(definition.children.clone()),
                });
            }
        }
        AstMut::Object(object) => {
            let ObjectData::FootnoteRef {
                label,
                resolved_label,
                definition,
            } = &mut object.data
            else {
                return;
            };
            let resolved = label.as_ref().filter(|label| !label.is_empty()).cloned();
            let next_anonymous = anonymous.len() + 1;
            let resolved = resolved.unwrap_or_else(|| {
                anonymous
                    .entry(object.ann.range)
                    .or_insert_with(|| format!("fn-{next_anonymous}"))
                    .clone()
            });
            *resolved_label = Some(resolved.clone());
            if !definition.is_empty() && definitions.insert(resolved.clone()) {
                entries.push(FootnoteEntry {
                    ann: object.ann.clone(),
                    label: resolved,
                    definition: FootnoteDefinition::Inline(definition.clone()),
                });
            }
        }
        _ => {}
    });
    document.footnotes = entries;
}
