//! Flat org-element-map-style index for the Org elements bridge.

use std::collections::BTreeMap;

use super::{
    Checkbox, Citation, Document, Element, ElementData, Keyword, ListItem, MarkupKind, Object,
    ObjectData, OrgElementsIndexCategory, OrgElementsIndexKind, OrgElementsIndexRecord,
    OrgElementsIndexSummary, OrgElementsIndexSummaryValue, ParsedAnnotation, Section, TargetKind,
};

pub(super) fn index_records(
    document: &Document<ParsedAnnotation>,
) -> Vec<OrgElementsIndexRecord<ParsedAnnotation>> {
    let mut index = ElementIndex::default();
    index.push(
        OrgElementsIndexCategory::Document,
        "org-data",
        &document.ann,
        &[],
        "root",
        summary([
            ("sections", document.sections.len().into()),
            ("elements", document.children.len().into()),
            ("metadata", document.metadata.len().into()),
        ]),
    );
    for keyword in &document.metadata {
        index.push_keyword(keyword, &[], "metadata");
    }
    for target in &document.targets {
        index.push(
            OrgElementsIndexCategory::TargetDefinition,
            "target-definition",
            &target.ann,
            &[],
            "sideTable",
            summary([
                ("key", target.key.clone().into()),
                ("value", target.value.clone().into()),
                ("targetKind", target_kind(target.kind).into()),
            ]),
        );
        index.collect_objects(&target.alias, &[], "targetAlias");
    }
    for footnote in &document.footnotes {
        index.push(
            OrgElementsIndexCategory::FootnoteEntry,
            "footnote-entry",
            &footnote.ann,
            &[],
            "sideTable",
            summary([("label", footnote.label.clone().into())]),
        );
    }
    index.collect_elements(&document.children, &[], "document");
    index.collect_sections(&document.sections, Vec::new());
    index.records
}

#[derive(Default)]
struct ElementIndex {
    next_ordinal: usize,
    records: Vec<OrgElementsIndexRecord<ParsedAnnotation>>,
}

impl ElementIndex {
    fn push(
        &mut self,
        category: OrgElementsIndexCategory,
        kind: impl Into<OrgElementsIndexKind>,
        ann: &ParsedAnnotation,
        outline_path: &[String],
        context: impl Into<String>,
        summary: OrgElementsIndexSummary,
    ) {
        self.next_ordinal += 1;
        self.records.push(OrgElementsIndexRecord {
            ann: ann.clone(),
            ordinal: self.next_ordinal,
            category,
            kind: kind.into(),
            outline_path: outline_path.to_vec(),
            context: context.into(),
            summary,
        });
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
                OrgElementsIndexCategory::Section,
                "headline",
                &section.ann,
                &path,
                "outline",
                summary([
                    ("level", section.level.into()),
                    ("title", section.raw_title.trim_end().into()),
                    (
                        "todo",
                        optional_text(section.todo.as_ref().map(|todo| todo.name.as_str())),
                    ),
                    ("tags", section.tags.clone().into()),
                    ("anchor", optional_text(section.anchor.as_deref())),
                ]),
            );
            self.collect_objects(&section.title, &path, "headlineTitle");
            for property in &section.properties {
                self.push(
                    OrgElementsIndexCategory::Property,
                    "node-property",
                    &property.ann,
                    &path,
                    "propertyDrawer",
                    summary([
                        ("key", property.key.clone().into()),
                        ("value", property.value.clone().into()),
                    ]),
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
            OrgElementsIndexCategory::Element,
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
                        OrgElementsIndexCategory::Element,
                        "table-row",
                        &row.ann,
                        outline_path,
                        "table",
                        summary([
                            ("isRule", row.is_rule.into()),
                            ("cells", row.cells.len().into()),
                        ]),
                    );
                    for cell in &row.cells {
                        self.push(
                            OrgElementsIndexCategory::Object,
                            "table-cell",
                            &cell.ann,
                            outline_path,
                            "tableRow",
                            summary([("objects", cell.objects.len().into())]),
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
            OrgElementsIndexCategory::Element,
            "item",
            &item.ann,
            outline_path,
            "plainList",
            summary([
                ("bullet", item.bullet.clone().into()),
                ("counter", optional_text(item.counter.as_deref())),
                ("checkbox", optional_text(item.checkbox.map(checkbox))),
                ("tagObjectCount", item.tag.len().into()),
            ]),
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
            OrgElementsIndexCategory::Object,
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
            OrgElementsIndexCategory::Keyword,
            "keyword",
            &keyword.ann,
            outline_path,
            context,
            summary([
                ("key", keyword.key.clone().into()),
                ("value", keyword.value.clone().into()),
                ("optional", optional_text(keyword.optional.as_deref())),
            ]),
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

fn element_summary(element: &Element<ParsedAnnotation>) -> OrgElementsIndexSummary {
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
        | ElementData::LatexEnvironment(raw)
        | ElementData::Unknown { raw, .. } => summary([("raw", raw.clone().into())]),
        ElementData::FixedWidth(fixed) => summary([("valueBytes", fixed.value.len().into())]),
        ElementData::TableEl { raw } => summary([("raw", raw.clone().into())]),
        ElementData::Rule => empty_summary(),
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

fn object_summary(object: &Object<ParsedAnnotation>) -> OrgElementsIndexSummary {
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

fn summary<const N: usize>(
    entries: [(&'static str, OrgElementsIndexSummaryValue); N],
) -> OrgElementsIndexSummary {
    entries
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
}

fn empty_summary() -> OrgElementsIndexSummary {
    BTreeMap::new()
}

fn optional_text(value: Option<&str>) -> OrgElementsIndexSummaryValue {
    value
        .map(OrgElementsIndexSummaryValue::from)
        .unwrap_or(OrgElementsIndexSummaryValue::Null)
}
