//! Materialize Scheme-AOT TAGS syntax from graph fields; no source lexer here.

use gerbil_parser_rowan::{GraphFieldValue, GraphRecord};
use rowan::{TextRange, TextSize};

use super::GraphProjector;
use crate::ast::{TagDefinition, TagDefinitionGroup};

enum TagEntry<'a> {
    Field(&'a GraphFieldValue),
    Group(&'a GraphRecord),
}

impl TagEntry<'_> {
    fn start(&self) -> TextSize {
        match self {
            Self::Field(field) => field.range.start(),
            Self::Group(group) => group.range.start(),
        }
    }
}

impl GraphProjector<'_> {
    pub(super) fn tag_definitions(&self, keyword_id: usize) -> Vec<TagDefinition> {
        let mut definitions = Vec::new();
        for &id in &self.record(keyword_id).child_ids {
            let vocabulary = self.record(id);
            if vocabulary.kind == "tag-vocabulary" {
                self.append_tag_definitions(vocabulary, None, &mut definitions);
            }
        }
        definitions
            .into_iter()
            .map(|(definition, _)| definition)
            .collect()
    }

    fn append_tag_definitions(
        &self,
        record: &GraphRecord,
        exclusive: Option<bool>,
        definitions: &mut Vec<(TagDefinition, TextRange)>,
    ) {
        let mut entries = record
            .fields
            .iter()
            .filter(|field| matches!(field.name, "name" | "shortcut" | "separator"))
            .map(TagEntry::Field)
            .collect::<Vec<_>>();
        entries.extend(
            record
                .child_ids
                .iter()
                .map(|&id| self.record(id))
                .filter(|child| matches!(child.kind, "tag-exclusive-group" | "tag-inclusive-group"))
                .map(TagEntry::Group),
        );
        entries.sort_unstable_by_key(TagEntry::start);

        let labeled = exclusive.is_some()
            && entries
                .iter()
                .any(|entry| matches!(entry, TagEntry::Field(field) if field.name == "separator"));
        let mut after_separator = false;
        let mut parent = None;
        for entry in entries {
            match entry {
                TagEntry::Field(field) if field.name == "separator" => {
                    after_separator = true;
                }
                TagEntry::Field(field) if field.name == "shortcut" => {
                    if !field.value.is_empty()
                        && let Some((previous, range)) = definitions.last_mut()
                        && previous.shortcut.is_none()
                    {
                        previous.shortcut = Some(field.value.clone());
                        *range =
                            TextRange::new(range.start(), field.range.end() + TextSize::from(1));
                        previous.raw = self.raw(*range).to_owned();
                    }
                }
                TagEntry::Field(field) if field.name == "name" && !field.value.is_empty() => {
                    let is_group = labeled && !after_separator;
                    let group =
                        exclusive
                            .filter(|_| !is_group)
                            .map(|exclusive| TagDefinitionGroup {
                                name: parent.clone(),
                                exclusive,
                            });
                    definitions.push((
                        TagDefinition {
                            name: field.value.clone(),
                            shortcut: None,
                            raw: field.value.clone(),
                            is_group,
                            group,
                        },
                        field.range,
                    ));
                    if is_group {
                        parent = Some(field.value.clone());
                    }
                }
                TagEntry::Group(child) => {
                    self.append_tag_definitions(
                        child,
                        Some(child.kind == "tag-exclusive-group"),
                        definitions,
                    );
                }
                TagEntry::Field(_) => {}
            }
        }
    }
}
