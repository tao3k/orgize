//! Object-level JSON projection for the Org elements bridge.

use serde_json::{Value, json};

use super::{
    AttachmentLinkSearchKind, Citation, CiteReference, FileLinkPathKind, Link,
    LinkDescriptionState, LinkMediaKind, LinkSearchKind, LinkTarget, MarkupKind, Object,
    ObjectData, ParsedAnnotation, RepeaterKind, TimeUnit, Timestamp, TimestampKind, WarningKind,
};

pub(super) fn objects_json(objects: &[Object<ParsedAnnotation>]) -> Vec<Value> {
    objects.iter().map(object_json).collect()
}

fn object_json(object: &Object<ParsedAnnotation>) -> Value {
    let (kind, value) = match &object.data {
        ObjectData::Plain(value) => ("plain-text", json!({ "value": value })),
        ObjectData::LineBreak => ("line-break", json!({})),
        ObjectData::Markup { kind, children } => (
            markup_kind(*kind),
            json!({
                "markupKind": markup_kind(*kind),
                "children": objects_json(children),
            }),
        ),
        ObjectData::Code(value) => ("code", json!({ "value": value })),
        ObjectData::Verbatim(value) => ("verbatim", json!({ "value": value })),
        ObjectData::Timestamp(timestamp) => ("timestamp", timestamp_json(timestamp)),
        ObjectData::Entity(raw) => ("entity", json!({ "raw": raw })),
        ObjectData::LatexFragment(raw) => ("latex-fragment", json!({ "raw": raw })),
        ObjectData::ExportSnippet { backend, value } => (
            "export-snippet",
            json!({
                "backend": backend,
                "value": value,
            }),
        ),
        ObjectData::FootnoteRef {
            label,
            resolved_label,
            definition,
        } => (
            "footnote-reference",
            json!({
                "label": label,
                "resolvedLabel": resolved_label,
                "definition": objects_json(definition),
            }),
        ),
        ObjectData::Citation(citation) => ("citation", citation_json(citation)),
        ObjectData::Cloze {
            text,
            raw_text,
            hint,
            id,
            raw,
        } => (
            "cloze",
            json!({
                "text": objects_json(text),
                "rawText": raw_text,
                "hint": hint,
                "id": id,
                "raw": raw,
            }),
        ),
        ObjectData::InlineCall {
            name,
            arguments,
            header,
            end_header,
            raw,
        } => (
            "inline-babel-call",
            json!({
                "name": name,
                "arguments": arguments,
                "header": header,
                "endHeader": end_header,
                "raw": raw,
            }),
        ),
        ObjectData::InlineSrc {
            language,
            parameters,
            value,
            raw,
        } => (
            "inline-src-block",
            json!({
                "language": language,
                "parameters": parameters,
                "value": value,
                "raw": raw,
            }),
        ),
        ObjectData::Link(link) => ("link", link_json(link)),
        ObjectData::Target(value) => ("target", json!({ "value": value })),
        ObjectData::RadioTarget(value) => ("radio-target", json!({ "value": value })),
        ObjectData::Macro { name, arguments } => (
            "macro",
            json!({
                "name": name,
                "arguments": arguments,
            }),
        ),
        ObjectData::StatisticCookie(raw) => ("statistics-cookie", json!({ "raw": raw })),
        ObjectData::Unknown { kind, raw } => (
            "unknown",
            json!({
                "syntaxKind": kind.as_str(),
                "raw": raw,
            }),
        ),
    };
    with_object_base(object, kind, value)
}

fn with_object_base(object: &Object<ParsedAnnotation>, kind: &str, mut value: Value) -> Value {
    if let Value::Object(map) = &mut value {
        map.insert(
            "source".to_string(),
            super::elements_bridge_json::annotation_json(&object.ann),
        );
        map.insert("kind".to_string(), json!(kind));
    }
    value
}

fn citation_json(citation: &Citation<ParsedAnnotation>) -> Value {
    json!({
        "style": &citation.style,
        "variant": &citation.variant,
        "prefix": objects_json(&citation.prefix),
        "suffix": objects_json(&citation.suffix),
        "references": citation.references.iter().map(cite_reference_json).collect::<Vec<_>>(),
    })
}

fn cite_reference_json(reference: &CiteReference<ParsedAnnotation>) -> Value {
    json!({
        "id": &reference.id,
        "prefix": objects_json(&reference.prefix),
        "suffix": objects_json(&reference.suffix),
    })
}

fn link_json(link: &Link<ParsedAnnotation>) -> Value {
    json!({
        "path": link.path(),
        "target": link_target_json(&link.target),
        "hasDescription": link.has_description(),
        "descriptionState": link_description_state(link.description_state),
        "description": objects_json(&link.description),
        "defaultDescription": objects_json(&link.default_description),
        "descriptionOrDefault": objects_json(link.description_or_default()),
        "rawDescription": &link.raw_description,
        "mediaKind": link_media_kind(link.media_kind),
        "isImage": link.is_image(),
        "caption": link.caption.as_ref().map(super::elements_bridge_json::keyword_json),
        "search": link.search.as_ref().map(|search| json!({
            "raw": &search.raw,
            "kind": link_search_kind(search.kind),
            "normalized": &search.normalized,
        })),
        "attachment": link.attachment.as_ref().map(|attachment| json!({
            "path": &attachment.path,
            "search": attachment.search.as_ref().map(|search| json!({
                "raw": &search.raw,
                "kind": attachment_link_search_kind(search.kind),
            })),
        })),
        "file": link.file.as_ref().map(|file| json!({
            "protocol": &file.protocol,
            "path": &file.path,
            "pathKind": file_link_path_kind(file.path_kind),
            "search": file.search.as_ref().map(|search| json!({
                "raw": &search.raw,
                "kind": link_search_kind(search.kind),
                "normalized": &search.normalized,
            })),
        })),
    })
}

pub(super) fn timestamp_json(timestamp: &Timestamp) -> Value {
    json!({
        "kind": timestamp_kind(timestamp.kind),
        "raw": &timestamp.raw,
        "isRange": timestamp.is_range,
        "start": timestamp.start.as_ref().map(|moment| json!({
            "year": moment.year,
            "month": moment.month,
            "day": moment.day,
            "dayName": &moment.day_name,
            "hour": moment.hour,
            "minute": moment.minute,
        })),
        "end": timestamp.end.as_ref().map(|moment| json!({
            "year": moment.year,
            "month": moment.month,
            "day": moment.day,
            "dayName": &moment.day_name,
            "hour": moment.hour,
            "minute": moment.minute,
        })),
        "repeater": timestamp.repeater.as_ref().map(|repeater| json!({
            "kind": repeater_kind(repeater.kind),
            "value": repeater.value,
            "unit": time_unit(repeater.unit),
        })),
        "warning": timestamp.warning.as_ref().map(|warning| json!({
            "kind": warning_kind(warning.kind),
            "value": warning.value,
            "unit": time_unit(warning.unit),
        })),
    })
}

fn markup_kind(kind: MarkupKind) -> &'static str {
    match kind {
        MarkupKind::Bold => "bold",
        MarkupKind::Italic => "italic",
        MarkupKind::Underline => "underline",
        MarkupKind::Strike => "strike-through",
        MarkupKind::Superscript => "superscript",
        MarkupKind::Subscript => "subscript",
    }
}

fn link_target_json(target: &LinkTarget) -> Value {
    match target {
        LinkTarget::Uri { protocol, path } => json!({
            "kind": "uri",
            "protocol": protocol,
            "path": path,
        }),
        LinkTarget::Internal(value) => json!({
            "kind": "internal",
            "value": value,
        }),
        LinkTarget::Unresolved(value) => json!({
            "kind": "unresolved",
            "value": value,
        }),
    }
}

fn link_description_state(state: LinkDescriptionState) -> &'static str {
    match state {
        LinkDescriptionState::None => "none",
        LinkDescriptionState::Explicit => "explicit",
    }
}

fn link_media_kind(kind: LinkMediaKind) -> &'static str {
    match kind {
        LinkMediaKind::Normal => "normal",
        LinkMediaKind::Image => "image",
    }
}

fn link_search_kind(kind: LinkSearchKind) -> &'static str {
    match kind {
        LinkSearchKind::Headline => "headline",
        LinkSearchKind::LineNumber => "lineNumber",
        LinkSearchKind::CustomId => "customId",
        LinkSearchKind::Regexp => "regexp",
        LinkSearchKind::Text => "text",
    }
}

fn attachment_link_search_kind(kind: AttachmentLinkSearchKind) -> &'static str {
    match kind {
        AttachmentLinkSearchKind::Headline => "headline",
        AttachmentLinkSearchKind::LineNumber => "lineNumber",
        AttachmentLinkSearchKind::CustomId => "customId",
        AttachmentLinkSearchKind::Regexp => "regexp",
        AttachmentLinkSearchKind::Text => "text",
    }
}

fn file_link_path_kind(kind: FileLinkPathKind) -> &'static str {
    match kind {
        FileLinkPathKind::Empty => "empty",
        FileLinkPathKind::Absolute => "absolute",
        FileLinkPathKind::HomeRelative => "homeRelative",
        FileLinkPathKind::Relative => "relative",
        FileLinkPathKind::Remote => "remote",
    }
}

fn timestamp_kind(kind: TimestampKind) -> &'static str {
    match kind {
        TimestampKind::Active => "active",
        TimestampKind::Inactive => "inactive",
        TimestampKind::Diary => "diary",
    }
}

fn repeater_kind(kind: RepeaterKind) -> &'static str {
    match kind {
        RepeaterKind::Cumulate => "cumulate",
        RepeaterKind::CatchUp => "catchUp",
        RepeaterKind::Restart => "restart",
    }
}

fn warning_kind(kind: WarningKind) -> &'static str {
    match kind {
        WarningKind::All => "all",
        WarningKind::First => "first",
    }
}

fn time_unit(unit: TimeUnit) -> &'static str {
    match unit {
        TimeUnit::Hour => "hour",
        TimeUnit::Day => "day",
        TimeUnit::Week => "week",
        TimeUnit::Month => "month",
        TimeUnit::Year => "year",
    }
}
