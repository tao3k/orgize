//! Flat org-element-map-style index for the Org elements bridge.

use serde_json::{json, Value};

use super::{
    Checkbox, Citation, Document, Element, ElementData, Keyword, ListItem, MarkupKind, Object,
    ObjectData, ParsedAnnotation, Section, TargetKind,
};

pub(super) fn index_json(document: &Document<ParsedAnnotation>) -> Vec<Value> {
    let mut index = ElementIndex::default();
    index.push(
        "document",
        "org-data",
        &document.ann,
        &[],
        "root",
        json!({
            "sections": document.sections.len(),
            "elements": document.children.len(),
            "metadata": document.metadata.len(),
        }),
    );
    for keyword in &document.metadata {
        index.push_keyword(keyword, &[], "metadata");
    }
    for target in &document.targets {
        index.push(
            "target-definition",
            "target-definition",
            &target.ann,
            &[],
            "sideTable",
            json!({
                "key": &target.key,
                "value": &target.value,
                "targetKind": target_kind(target.kind),
            }),
        );
        index.collect_objects(&target.alias, &[], "targetAlias");
    }
    for footnote in &document.footnotes {
        index.push(
            "footnote-entry",
            "footnote-entry",
            &footnote.ann,
            &[],
            "sideTable",
            json!({ "label": &footnote.label }),
        );
    }
    index.collect_elements(&document.children, &[], "document");
    index.collect_sections(&document.sections, Vec::new());
    index.records
}

#[derive(Default)]
struct ElementIndex {
    next_ordinal: usize,
    records: Vec<Value>,
}

impl ElementIndex {
    fn push(
        &mut self,
        category: &str,
        kind: &str,
        ann: &ParsedAnnotation,
        outline_path: &[String],
        context: &str,
        summary: Value,
    ) {
        self.next_ordinal += 1;
        self.records.push(json!({
            "ordinal": self.next_ordinal,
            "category": category,
            "kind": kind,
            "source": super::elements_bridge_json::annotation_json(ann),
            "outlinePath": outline_path,
            "context": context,
            "summary": summary,
        }));
    }

    fn collect_sections(
        &mut self,
        sections: &[Section<ParsedAnnotation>],
        outline_path: Vec<String>,
    ) {
        for section in sections {
            let mut path = outline_path.clone();
            path.push(section.raw_title.trim_end().to_string());
            self.push(
                "section",
                "headline",
                &section.ann,
                &path,
                "outline",
                json!({
                    "level": section.level,
                    "title": section.raw_title.trim_end(),
                    "todo": section.todo.as_ref().map(|todo| todo.name.as_str()),
                    "tags": &section.tags,
                    "anchor": &section.anchor,
                }),
            );
            self.collect_objects(&section.title, &path, "headlineTitle");
            for property in &section.properties {
                self.push(
                    "property",
                    "node-property",
                    &property.ann,
                    &path,
                    "propertyDrawer",
                    json!({
                        "key": &property.key,
                        "value": &property.value,
                    }),
                );
            }
            self.collect_elements(&section.children, &path, "section");
            self.collect_sections(&section.subsections, path);
        }
    }

    fn collect_elements(
        &mut self,
        elements: &[Element<ParsedAnnotation>],
        outline_path: &[String],
        context: &str,
    ) {
        for element in elements {
            self.collect_element(element, outline_path, context);
        }
    }

    fn collect_element(
        &mut self,
        element: &Element<ParsedAnnotation>,
        outline_path: &[String],
        context: &str,
    ) {
        for keyword in &element.affiliated_keywords {
            self.push_keyword(keyword, outline_path, "affiliatedKeyword");
        }
        self.push(
            "element",
            element_kind(element),
            &element.ann,
            outline_path,
            context,
            element_summary(element),
        );
        match &element.data {
            ElementData::Paragraph(objects) => {
                self.collect_objects(objects, outline_path, "paragraph")
            }
            ElementData::Keyword(keyword) | ElementData::BabelCall(keyword) => {
                self.collect_objects(&keyword.parsed, outline_path, "keywordValue");
            }
            ElementData::Drawer(drawer) => {
                self.collect_elements(&drawer.children, outline_path, "drawer");
            }
            ElementData::List(list) => {
                for item in &list.items {
                    self.collect_list_item(item, outline_path);
                }
            }
            ElementData::Table(table) => {
                for row in &table.rows {
                    self.push(
                        "element",
                        "table-row",
                        &row.ann,
                        outline_path,
                        "table",
                        json!({ "isRule": row.is_rule, "cells": row.cells.len() }),
                    );
                    for cell in &row.cells {
                        self.push(
                            "object",
                            "table-cell",
                            &cell.ann,
                            outline_path,
                            "tableRow",
                            json!({ "objects": cell.objects.len() }),
                        );
                        self.collect_objects(&cell.objects, outline_path, "tableCell");
                    }
                }
            }
            ElementData::Block(block) => {
                self.collect_elements(&block.children, outline_path, "block")
            }
            ElementData::FootnoteDef(footnote) => {
                self.collect_elements(&footnote.children, outline_path, "footnoteDefinition");
            }
            ElementData::Inlinetask(task) => {
                self.collect_objects(&task.title, outline_path, "inlinetaskTitle");
                self.collect_elements(&task.children, outline_path, "inlinetask");
            }
            ElementData::Clock(_)
            | ElementData::PropertyDrawer(_)
            | ElementData::TableEl { .. }
            | ElementData::Comment(_)
            | ElementData::FixedWidth(_)
            | ElementData::Rule
            | ElementData::LatexEnvironment(_)
            | ElementData::Unknown { .. } => {}
        }
    }

    fn collect_list_item(&mut self, item: &ListItem<ParsedAnnotation>, outline_path: &[String]) {
        self.push(
            "element",
            "item",
            &item.ann,
            outline_path,
            "plainList",
            json!({
                "bullet": &item.bullet,
                "counter": &item.counter,
                "checkbox": item.checkbox.map(checkbox),
                "tagObjectCount": item.tag.len(),
            }),
        );
        self.collect_objects(&item.tag, outline_path, "listItemTag");
        self.collect_elements(&item.children, outline_path, "listItem");
    }

    fn collect_objects(
        &mut self,
        objects: &[Object<ParsedAnnotation>],
        outline_path: &[String],
        context: &str,
    ) {
        for object in objects {
            self.collect_object(object, outline_path, context);
        }
    }

    fn collect_object(
        &mut self,
        object: &Object<ParsedAnnotation>,
        outline_path: &[String],
        context: &str,
    ) {
        self.push(
            "object",
            object_kind(object),
            &object.ann,
            outline_path,
            context,
            object_summary(object),
        );
        match &object.data {
            ObjectData::Markup { children, .. } => {
                self.collect_objects(children, outline_path, "markup")
            }
            ObjectData::FootnoteRef { definition, .. } => {
                self.collect_objects(definition, outline_path, "footnoteReference")
            }
            ObjectData::Citation(citation) => self.collect_citation(citation, outline_path),
            ObjectData::Cloze { text, .. } => self.collect_objects(text, outline_path, "cloze"),
            ObjectData::Link(link) => {
                if link.has_description() {
                    self.collect_objects(&link.description, outline_path, "linkDescription");
                } else {
                    self.collect_objects(
                        &link.default_description,
                        outline_path,
                        "linkDefaultDescription",
                    );
                }
            }
            ObjectData::Plain(_)
            | ObjectData::LineBreak
            | ObjectData::Code(_)
            | ObjectData::Verbatim(_)
            | ObjectData::Timestamp(_)
            | ObjectData::Entity(_)
            | ObjectData::LatexFragment(_)
            | ObjectData::ExportSnippet { .. }
            | ObjectData::InlineCall { .. }
            | ObjectData::InlineSrc { .. }
            | ObjectData::Target(_)
            | ObjectData::RadioTarget(_)
            | ObjectData::Macro { .. }
            | ObjectData::StatisticCookie(_)
            | ObjectData::Unknown { .. } => {}
        }
    }

    fn collect_citation(&mut self, citation: &Citation<ParsedAnnotation>, outline_path: &[String]) {
        self.collect_objects(&citation.prefix, outline_path, "citationPrefix");
        self.collect_objects(&citation.suffix, outline_path, "citationSuffix");
        for reference in &citation.references {
            self.collect_objects(&reference.prefix, outline_path, "citationReferencePrefix");
            self.collect_objects(&reference.suffix, outline_path, "citationReferenceSuffix");
        }
    }

    fn push_keyword(
        &mut self,
        keyword: &Keyword<ParsedAnnotation>,
        outline_path: &[String],
        context: &str,
    ) {
        self.push(
            "keyword",
            "keyword",
            &keyword.ann,
            outline_path,
            context,
            json!({
                "key": &keyword.key,
                "value": &keyword.value,
                "optional": &keyword.optional,
            }),
        );
        self.collect_objects(&keyword.parsed, outline_path, "keywordValue");
    }
}

fn element_kind(element: &Element<ParsedAnnotation>) -> &'static str {
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
        ElementData::FixedWidth(_) => "fixed-width",
        ElementData::Rule => "horizontal-rule",
        ElementData::LatexEnvironment(_) => "latex-environment",
        ElementData::Unknown { .. } => "unknown",
    }
}

fn element_summary(element: &Element<ParsedAnnotation>) -> Value {
    match &element.data {
        ElementData::Keyword(keyword) | ElementData::BabelCall(keyword) => {
            json!({ "key": &keyword.key, "value": &keyword.value })
        }
        ElementData::Drawer(drawer) => json!({ "name": &drawer.name }),
        ElementData::List(list) => json!({ "items": list.items.len() }),
        ElementData::Table(table) => json!({ "rows": table.rows.len() }),
        ElementData::Block(block) => json!({
            "name": &block.name,
            "language": &block.language,
            "valueBytes": block.value.len(),
        }),
        ElementData::FootnoteDef(footnote) => json!({ "label": &footnote.label }),
        ElementData::Inlinetask(task) => json!({ "title": task.raw_title.trim_end() }),
        ElementData::Clock(clock) => json!({ "raw": &clock.raw, "duration": &clock.duration }),
        ElementData::Paragraph(objects) => json!({ "objects": objects.len() }),
        ElementData::PropertyDrawer(properties) => json!({ "properties": properties.len() }),
        ElementData::Comment(raw)
        | ElementData::LatexEnvironment(raw)
        | ElementData::Unknown { raw, .. } => json!({ "raw": raw }),
        ElementData::FixedWidth(fixed) => json!({ "valueBytes": fixed.value.len() }),
        ElementData::TableEl { raw } => json!({ "raw": raw }),
        ElementData::Rule => json!({}),
    }
}

fn object_kind(object: &Object<ParsedAnnotation>) -> &'static str {
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

fn object_summary(object: &Object<ParsedAnnotation>) -> Value {
    match &object.data {
        ObjectData::Plain(value)
        | ObjectData::Code(value)
        | ObjectData::Verbatim(value)
        | ObjectData::Entity(value)
        | ObjectData::LatexFragment(value)
        | ObjectData::Target(value)
        | ObjectData::RadioTarget(value)
        | ObjectData::StatisticCookie(value) => json!({ "value": value }),
        ObjectData::Timestamp(timestamp) => json!({ "raw": &timestamp.raw }),
        ObjectData::Link(link) => json!({
            "path": link.path(),
            "hasDescription": link.has_description(),
            "isImage": link.is_image(),
        }),
        ObjectData::InlineSrc {
            language,
            parameters,
            value,
            ..
        } => json!({ "language": language, "parameters": parameters, "value": value }),
        ObjectData::InlineCall {
            name, arguments, ..
        } => json!({ "name": name, "arguments": arguments }),
        ObjectData::ExportSnippet { backend, value } => {
            json!({ "backend": backend, "value": value })
        }
        ObjectData::FootnoteRef {
            label,
            resolved_label,
            ..
        } => json!({ "label": label, "resolvedLabel": resolved_label }),
        ObjectData::Citation(citation) => {
            json!({ "style": &citation.style, "references": citation.references.len() })
        }
        ObjectData::Cloze { raw, .. } | ObjectData::Unknown { raw, .. } => json!({ "raw": raw }),
        ObjectData::Macro { name, arguments } => {
            json!({ "name": name, "arguments": arguments })
        }
        ObjectData::Markup { children, .. } => json!({ "children": children.len() }),
        ObjectData::LineBreak => json!({}),
    }
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

fn target_kind(kind: TargetKind) -> &'static str {
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

fn checkbox(checkbox: Checkbox) -> &'static str {
    match checkbox {
        Checkbox::On => "on",
        Checkbox::Off => "off",
        Checkbox::Trans => "trans",
    }
}
