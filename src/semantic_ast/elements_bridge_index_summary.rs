//! Element/object summary and text rendering helpers for bridge index records.

use super::{
    Checkbox, Element, ElementData, Keyword, MarkupKind, Object, ObjectData, OrgElementProperties,
    OrgElementsAffiliatedProperties, OrgElementsIndexSummary, OrgElementsIndexSummaryValue,
    ParsedAnnotation, TargetKind, TodoState,
};
use crate::ast::elements_bridge_model::{
    OrgElementPropertyProvenance, OrgElementPropertyProvenanceMap,
};
use std::collections::BTreeMap;

pub(super) fn element_kind(element: &Element<ParsedAnnotation>) -> &'static str {
    match &element.data {
        ElementData::Paragraph(_) => "paragraph",
        ElementData::Keyword(_) => "keyword",
        ElementData::BabelCall(_) => "babel-call",
        ElementData::Clock(_) => "clock",
        ElementData::Drawer(_) => "drawer",
        ElementData::PropertyDrawer(_) => "property-drawer",
        ElementData::List(_) => "plain-list",
        ElementData::Table(_) => "table",
        ElementData::TableEl { .. } => "table.el",
        ElementData::Block(block) => match &block.kind {
            super::BlockKind::Source => "src-block",
            super::BlockKind::Example => "example-block",
            super::BlockKind::Export => "export-block",
            super::BlockKind::Quote => "quote-block",
            super::BlockKind::Verse => "verse-block",
            super::BlockKind::Center => "center-block",
            super::BlockKind::Comment => "comment-block",
            super::BlockKind::Dynamic => "dynamic-block",
            super::BlockKind::Special(_) => "special-block",
        },
        ElementData::FootnoteDef(_) => "footnote-definition",
        ElementData::Inlinetask(_) => "inlinetask",
        ElementData::Comment(_) => "comment",
        ElementData::DiarySexp(_) => "diary-sexp",
        ElementData::FixedWidth(_) => "fixed-width",
        ElementData::Rule => "horizontal-rule",
        ElementData::LatexEnvironment(_) => "latex-environment",
        ElementData::Unknown { .. } => "unknown",
    }
}

pub(super) fn element_summary(element: &Element<ParsedAnnotation>) -> OrgElementsIndexSummary {
    match &element.data {
        ElementData::Keyword(keyword) | ElementData::BabelCall(keyword) => summary([
            ("key", keyword.key.clone().into()),
            ("value", keyword.value.clone().into()),
        ]),
        ElementData::Drawer(drawer) => summary([("name", drawer.name.clone().into())]),
        ElementData::List(list) => summary([("items", list.items.len().into())]),
        ElementData::Table(table) => summary([("rows", table.rows.len().into())]),
        ElementData::Block(block) => summary([
            ("name", optional_text(block.name.as_deref())),
            ("language", optional_text(block.language.as_deref())),
            ("value", block.value.clone().into()),
            ("valueBytes", block.value.len().into()),
        ]),
        ElementData::FootnoteDef(footnote) => summary([("label", footnote.label.clone().into())]),
        ElementData::Inlinetask(task) => summary([("title", task.raw_title.trim_end().into())]),
        ElementData::Clock(clock) => summary([
            ("raw", clock.raw.clone().into()),
            ("duration", optional_text(clock.duration.as_deref())),
        ]),
        ElementData::Paragraph(objects) => summary([("objects", objects.len().into())]),
        ElementData::PropertyDrawer(properties) => {
            summary([("properties", properties.len().into())])
        }
        ElementData::Comment(raw)
        | ElementData::DiarySexp(raw)
        | ElementData::LatexEnvironment(raw)
        | ElementData::Unknown { raw, .. } => summary([("raw", raw.clone().into())]),
        ElementData::FixedWidth(fixed) => summary([("valueBytes", fixed.value.len().into())]),
        ElementData::TableEl { raw } => summary([("raw", raw.clone().into())]),
        ElementData::Rule => empty_summary(),
    }
}

pub(super) fn element_affiliated_properties(
    element: &Element<ParsedAnnotation>,
) -> OrgElementsAffiliatedProperties {
    OrgElementsAffiliatedProperties {
        name: affiliated_keyword_value(&element.affiliated_keywords, "NAME"),
    }
}

pub(super) fn object_kind(object: &Object<ParsedAnnotation>) -> &'static str {
    match &object.data {
        ObjectData::Plain(_) => "plain-text",
        ObjectData::LineBreak => "line-break",
        ObjectData::Markup { kind, .. } => markup_kind(*kind),
        ObjectData::Code(_) => "code",
        ObjectData::Verbatim(_) => "verbatim",
        ObjectData::Timestamp(_) => "timestamp",
        ObjectData::Entity(_) => "entity",
        ObjectData::LatexFragment(_) => "latex-fragment",
        ObjectData::ExportSnippet { .. } => "export-snippet",
        ObjectData::FootnoteRef { .. } => "footnote-reference",
        ObjectData::Citation(_) => "citation",
        ObjectData::Cloze { .. } => "cloze",
        ObjectData::InlineCall { .. } => "inline-babel-call",
        ObjectData::InlineSrc { .. } => "inline-src-block",
        ObjectData::Link(_) => "link",
        ObjectData::Target(_) => "target",
        ObjectData::RadioTarget(_) => "radio-target",
        ObjectData::Macro { .. } => "macro",
        ObjectData::StatisticCookie(_) => "statistics-cookie",
        ObjectData::Unknown { .. } => "unknown",
    }
}

pub(super) fn object_summary(object: &Object<ParsedAnnotation>) -> OrgElementsIndexSummary {
    match &object.data {
        ObjectData::Plain(value)
        | ObjectData::Code(value)
        | ObjectData::Verbatim(value)
        | ObjectData::Entity(value)
        | ObjectData::LatexFragment(value)
        | ObjectData::Target(value)
        | ObjectData::RadioTarget(value)
        | ObjectData::StatisticCookie(value) => summary([("value", value.clone().into())]),
        ObjectData::Timestamp(timestamp) => summary([("raw", timestamp.raw.clone().into())]),
        ObjectData::Link(link) => summary([
            ("path", link.path().to_string().into()),
            ("hasDescription", link.has_description().into()),
            ("isImage", link.is_image().into()),
        ]),
        ObjectData::InlineSrc {
            language,
            parameters,
            value,
            ..
        } => summary([
            ("language", language.clone().into()),
            ("parameters", optional_text(parameters.as_deref())),
            ("value", value.clone().into()),
        ]),
        ObjectData::InlineCall {
            name, arguments, ..
        } => summary([
            ("name", name.clone().into()),
            ("arguments", arguments.clone().into()),
        ]),
        ObjectData::ExportSnippet { backend, value } => summary([
            ("backend", backend.clone().into()),
            ("value", value.clone().into()),
        ]),
        ObjectData::FootnoteRef {
            label,
            resolved_label,
            ..
        } => summary([
            ("label", optional_text(label.as_deref())),
            ("resolvedLabel", optional_text(resolved_label.as_deref())),
        ]),
        ObjectData::Citation(citation) => summary([
            ("style", citation.style.clone().into()),
            ("references", citation.references.len().into()),
        ]),
        ObjectData::Cloze { raw, .. } | ObjectData::Unknown { raw, .. } => {
            summary([("raw", raw.clone().into())])
        }
        ObjectData::Macro { name, arguments } => summary([
            ("name", name.clone().into()),
            ("arguments", arguments.clone().into()),
        ]),
        ObjectData::Markup { children, .. } => summary([("children", children.len().into())]),
        ObjectData::LineBreak => empty_summary(),
    }
}

pub(super) fn objects_text(objects: &[Object<ParsedAnnotation>]) -> String {
    objects.iter().map(object_text).collect::<Vec<_>>().join("")
}

pub(super) fn object_text(object: &Object<ParsedAnnotation>) -> String {
    match &object.data {
        ObjectData::Plain(value)
        | ObjectData::Code(value)
        | ObjectData::Verbatim(value)
        | ObjectData::Entity(value)
        | ObjectData::LatexFragment(value)
        | ObjectData::Target(value)
        | ObjectData::RadioTarget(value)
        | ObjectData::StatisticCookie(value) => value.clone(),
        ObjectData::Timestamp(timestamp) => timestamp.raw.clone(),
        ObjectData::LineBreak => "\n".to_string(),
        ObjectData::InlineSrc { value, .. } | ObjectData::ExportSnippet { value, .. } => {
            value.clone()
        }
        ObjectData::InlineCall {
            name, arguments, ..
        } => {
            if arguments.is_empty() {
                name.clone()
            } else {
                format!("{name}({arguments})")
            }
        }
        ObjectData::FootnoteRef {
            label,
            resolved_label,
            definition,
            ..
        } => label
            .as_ref()
            .or(resolved_label.as_ref())
            .cloned()
            .unwrap_or_else(|| objects_text(definition)),
        ObjectData::Citation(citation) => citation
            .references
            .iter()
            .map(|reference| reference.id.as_str())
            .collect::<Vec<_>>()
            .join(","),
        ObjectData::Cloze { text, .. } | ObjectData::Markup { children: text, .. } => {
            objects_text(text)
        }
        ObjectData::Link(link) => {
            if link.has_description() {
                objects_text(&link.description)
            } else {
                objects_text(&link.default_description)
            }
        }
        ObjectData::Macro { name, arguments } => {
            if arguments.is_empty() {
                name.clone()
            } else {
                format!("{name}({})", arguments.join(","))
            }
        }
        ObjectData::Unknown { raw, .. } => raw.clone(),
    }
}

pub(super) fn markup_kind(kind: MarkupKind) -> &'static str {
    match kind {
        MarkupKind::Bold => "bold",
        MarkupKind::Italic => "italic",
        MarkupKind::Underline => "underline",
        MarkupKind::Strike => "strike-through",
        MarkupKind::Superscript => "superscript",
        MarkupKind::Subscript => "subscript",
    }
}

pub(super) fn target_kind(kind: TargetKind) -> &'static str {
    match kind {
        TargetKind::Headline => "headline",
        TargetKind::CustomId => "customId",
        TargetKind::Id => "id",
        TargetKind::Target => "target",
        TargetKind::RadioTarget => "radioTarget",
        TargetKind::FootnoteDefinition => "footnoteDefinition",
        TargetKind::CodeRef => "codeRef",
    }
}

pub(super) fn checkbox(checkbox: Checkbox) -> &'static str {
    match checkbox {
        Checkbox::On => "on",
        Checkbox::Off => "off",
        Checkbox::Trans => "trans",
    }
}

pub(super) fn summary<const N: usize>(
    entries: [(&'static str, OrgElementsIndexSummaryValue); N],
) -> OrgElementsIndexSummary {
    entries
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
}

pub(super) fn empty_summary() -> OrgElementsIndexSummary {
    BTreeMap::new()
}

pub(super) fn properties_from_summary(summary: &OrgElementsIndexSummary) -> OrgElementProperties {
    summary
        .iter()
        .map(|(key, value)| (org_property_key(key), value.clone()))
        .collect()
}

pub(super) fn property_provenance_from_summary(
    summary: &OrgElementsIndexSummary,
) -> OrgElementPropertyProvenanceMap {
    summary
        .keys()
        .map(|key| (org_property_key(key), OrgElementPropertyProvenance::Summary))
        .collect()
}

pub(super) fn property_provenance_from_properties(
    properties: &OrgElementProperties,
    provenance: OrgElementPropertyProvenance,
) -> OrgElementPropertyProvenanceMap {
    properties
        .keys()
        .map(|key| (org_property_key(key), provenance))
        .collect()
}

pub(super) fn todo_state_label(state: TodoState) -> &'static str {
    match state {
        TodoState::Todo => "todo",
        TodoState::Done => "done",
    }
}

pub(super) fn affiliated_keyword_value(
    keywords: &[Keyword<ParsedAnnotation>],
    key: &str,
) -> Option<String> {
    keywords
        .iter()
        .find(|keyword| keyword.key.eq_ignore_ascii_case(key))
        .map(|keyword| keyword.value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(super) fn optional_text(value: Option<&str>) -> OrgElementsIndexSummaryValue {
    value
        .map(OrgElementsIndexSummaryValue::from)
        .unwrap_or(OrgElementsIndexSummaryValue::Null)
}

pub(super) fn optional_usize(value: Option<usize>) -> OrgElementsIndexSummaryValue {
    value
        .map(OrgElementsIndexSummaryValue::from)
        .unwrap_or(OrgElementsIndexSummaryValue::Null)
}
use super::elements_bridge_index_properties::org_property_key;
