//! Document-local target values from Scheme-classified Element graph records.

use super::{GraphProjector, ParsedAnnotation};
use crate::ast::{TargetDefinition, TargetKind};

impl GraphProjector<'_> {
    pub(super) fn targets(&self) -> Vec<TargetDefinition<ParsedAnnotation>> {
        self.document
            .records()
            .iter()
            .filter_map(|record| {
                let (kind, key, value, range) = match record.kind {
                    "headline" => {
                        let title = self.document.headline_display_title(record.id)?;
                        (!title.is_empty())
                            .then(|| (TargetKind::Headline, title.clone(), title, record.range))?
                    }
                    "node-property" => {
                        let value = record.field("value")?.to_owned();
                        let kind = match record.field("key")? {
                            key if key.eq_ignore_ascii_case("CUSTOM_ID") => TargetKind::CustomId,
                            key if key.eq_ignore_ascii_case("ID") => TargetKind::Id,
                            _ => return None,
                        };
                        let prefix = if kind == TargetKind::CustomId {
                            "#"
                        } else {
                            "id:"
                        };
                        (
                            kind,
                            format!("{prefix}{value}"),
                            value,
                            record.field_range("value")?,
                        )
                    }
                    "target" | "radio-target" => {
                        let value = record.field("value")?.to_owned();
                        let kind = if record.kind == "target" {
                            TargetKind::Target
                        } else {
                            TargetKind::RadioTarget
                        };
                        (kind, value.clone(), value, record.range)
                    }
                    "footnote-definition" => {
                        let value = record.field("label")?.to_owned();
                        (
                            TargetKind::FootnoteDefinition,
                            format!("fn:{value}"),
                            value,
                            record.range,
                        )
                    }
                    _ => return None,
                };
                let alias = match kind {
                    TargetKind::Headline => self.headline_aliases.get(&record.id),
                    TargetKind::CustomId | TargetKind::Id => self
                        .nearest_headline(record.id)
                        .and_then(|headline| self.headline_aliases.get(&headline)),
                    _ => None,
                }
                .cloned()
                .unwrap_or_default();
                let raw = if matches!(
                    kind,
                    TargetKind::Headline | TargetKind::CustomId | TargetKind::Id
                ) {
                    value.clone()
                } else {
                    self.raw(record.range).to_owned()
                };
                Some(TargetDefinition {
                    ann: self.annotation(range),
                    kind,
                    key,
                    value,
                    raw,
                    alias,
                })
            })
            .collect()
    }
}
