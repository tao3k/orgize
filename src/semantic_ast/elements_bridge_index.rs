//! Flat org-element-map-style index for the Org elements bridge.

use std::collections::BTreeMap;

use super::{
    Checkbox, Citation, Document, Element, ElementData, Keyword, ListItem, MarkupKind, Object,
    ObjectData, OrgElementId, OrgElementProperties, OrgElementsAffiliatedProperties,
    OrgElementsIndexCategory, OrgElementsIndexKind, OrgElementsIndexRecord,
    OrgElementsIndexSummary, OrgElementsIndexSummaryValue, ParsedAnnotation, Section, TargetKind,
    TodoState,
};

pub(super) fn index_records(
    document: &Document<ParsedAnnotation>,
) -> Vec<OrgElementsIndexRecord<ParsedAnnotation>> {
    let mut index = ElementIndex::default();
    let root_id = index.push(
        None,
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
        index.push_keyword(keyword, &[], "metadata", root_id);
    }
    index.collect_property_drawer(&document.properties, &[], "document", root_id);
    for target in &document.targets {
        index.push(
            Some(root_id),
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
        index.collect_objects(&target.alias, &[], "targetAlias", root_id);
    }
    for footnote in &document.footnotes {
        index.push(
            Some(root_id),
            OrgElementsIndexCategory::FootnoteEntry,
            "footnote-entry",
            &footnote.ann,
            &[],
            "sideTable",
            summary([("label", footnote.label.clone().into())]),
        );
    }
    index.collect_elements(&document.children, &[], "document", root_id);
    index.collect_sections(&document.sections, Vec::new(), root_id);
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
        parent_id: Option<OrgElementId>,
        category: OrgElementsIndexCategory,
        kind: impl Into<OrgElementsIndexKind>,
        ann: &ParsedAnnotation,
        outline_path: &[String],
        context: impl Into<String>,
        summary: OrgElementsIndexSummary,
    ) -> OrgElementId {
        self.push_with_properties(
            parent_id,
            category,
            kind,
            ann,
            outline_path,
            context,
            properties_from_summary(&summary),
            summary,
        )
    }

    fn push_with_properties(
        &mut self,
        parent_id: Option<OrgElementId>,
        category: OrgElementsIndexCategory,
        kind: impl Into<OrgElementsIndexKind>,
        ann: &ParsedAnnotation,
        outline_path: &[String],
        context: impl Into<String>,
        properties: OrgElementProperties,
        summary: OrgElementsIndexSummary,
    ) -> OrgElementId {
        self.push_with_standard_properties(
            parent_id,
            category,
            kind,
            ann,
            outline_path,
            context,
            properties,
            summary,
            StandardProperties::default(),
        )
    }

    fn push_with_standard_properties(
        &mut self,
        parent_id: Option<OrgElementId>,
        category: OrgElementsIndexCategory,
        kind: impl Into<OrgElementsIndexKind>,
        ann: &ParsedAnnotation,
        outline_path: &[String],
        context: impl Into<String>,
        properties: OrgElementProperties,
        summary: OrgElementsIndexSummary,
        standard: StandardProperties,
    ) -> OrgElementId {
        self.next_ordinal += 1;
        let id = OrgElementId::new(self.next_ordinal);
        let properties = properties_with_standard_properties(properties, ann, parent_id, standard);
        self.records.push(OrgElementsIndexRecord {
            id,
            parent_id,
            child_ids: Vec::new(),
            ann: ann.clone(),
            ordinal: self.next_ordinal,
            category,
            kind: kind.into(),
            affiliated: OrgElementsAffiliatedProperties::default(),
            outline_path: outline_path.to_vec(),
            context: context.into(),
            properties,
            summary,
        });
        if let Some(parent_id) = parent_id
            && let Some(parent) = self
                .records
                .iter_mut()
                .find(|record| record.id == parent_id)
        {
            parent.child_ids.push(id);
        }
        id
    }

    fn push_element(
        &mut self,
        parent_id: OrgElementId,
        element: &Element<ParsedAnnotation>,
        outline_path: &[String],
        context: impl Into<String>,
        summary: OrgElementsIndexSummary,
    ) -> OrgElementId {
        self.next_ordinal += 1;
        let id = OrgElementId::new(self.next_ordinal);
        let properties = properties_with_standard_properties(
            properties_from_summary(&summary),
            &element.ann,
            Some(parent_id),
            StandardProperties::default(),
        );
        self.records.push(OrgElementsIndexRecord {
            id,
            parent_id: Some(parent_id),
            child_ids: Vec::new(),
            ann: element.ann.clone(),
            ordinal: self.next_ordinal,
            category: OrgElementsIndexCategory::Element,
            kind: element_kind(element).into(),
            affiliated: element_affiliated_properties(element),
            outline_path: outline_path.to_vec(),
            context: context.into(),
            properties,
            summary,
        });
        if let Some(parent) = self
            .records
            .iter_mut()
            .find(|record| record.id == parent_id)
        {
            parent.child_ids.push(id);
        }
        id
    }

    fn collect_sections(
        &mut self,
        sections: &[Section<ParsedAnnotation>],
        outline_path: Vec<String>,
        parent_id: OrgElementId,
    ) {
        for section in sections {
            let mut path = outline_path.clone();
            path.push(section.raw_title.trim_end().to_string());
            let headline_summary = summary([
                ("level", section.level.into()),
                ("title", section.raw_title.trim_end().into()),
                (
                    "todo",
                    optional_text(section.todo.as_ref().map(|todo| todo.name.as_str())),
                ),
                ("tags", section.tags.clone().into()),
                ("anchor", optional_text(section.anchor.as_deref())),
            ]);
            let headline_standard = section
                .body_ann
                .as_ref()
                .map(StandardProperties::from_content_ann)
                .unwrap_or_default();
            let section_id = self.push_with_standard_properties(
                Some(parent_id),
                OrgElementsIndexCategory::Section,
                "headline",
                &section.ann,
                &path,
                "outline",
                headline_properties(section, &headline_summary),
                headline_summary,
                headline_standard,
            );
            if has_planning(&section.planning) {
                self.push(
                    Some(section_id),
                    OrgElementsIndexCategory::Element,
                    "planning",
                    &section.ann,
                    &path,
                    "headline",
                    planning_summary(&section.planning),
                );
            }
            self.collect_objects(&section.title, &path, "headlineTitle", section_id);
            self.collect_property_drawer(&section.properties, &path, "headline", section_id);
            let body_parent_id = if let Some(body_ann) = &section.body_ann {
                let body_summary = summary([("elements", section.children.len().into())]);
                self.push_with_standard_properties(
                    Some(section_id),
                    OrgElementsIndexCategory::Element,
                    "section",
                    body_ann,
                    &path,
                    "headline",
                    properties_from_summary(&body_summary),
                    body_summary,
                    StandardProperties::from_content_ann(body_ann),
                )
            } else {
                section_id
            };
            self.collect_elements(&section.children, &path, "section", body_parent_id);
            self.collect_sections(&section.subsections, path, section_id);
        }
    }

    fn collect_property_drawer(
        &mut self,
        properties: &[super::Property<ParsedAnnotation>],
        outline_path: &[String],
        context: &str,
        parent_id: OrgElementId,
    ) {
        let Some(first_property) = properties.first() else {
            return;
        };
        let drawer_id = self.push(
            Some(parent_id),
            OrgElementsIndexCategory::Element,
            "property-drawer",
            &first_property.ann,
            outline_path,
            context,
            summary([("properties", properties.len().into())]),
        );
        for property in properties {
            self.push(
                Some(drawer_id),
                OrgElementsIndexCategory::Property,
                "node-property",
                &property.ann,
                outline_path,
                "propertyDrawer",
                summary([
                    ("key", property.key.clone().into()),
                    ("value", property.value.clone().into()),
                ]),
            );
        }
    }

    fn collect_elements(
        &mut self,
        elements: &[Element<ParsedAnnotation>],
        outline_path: &[String],
        context: &str,
        parent_id: OrgElementId,
    ) {
        for element in elements {
            self.collect_element(element, outline_path, context, parent_id);
        }
    }

    fn collect_element(
        &mut self,
        element: &Element<ParsedAnnotation>,
        outline_path: &[String],
        context: &str,
        parent_id: OrgElementId,
    ) {
        for keyword in &element.affiliated_keywords {
            self.push_keyword(keyword, outline_path, "affiliatedKeyword", parent_id);
        }
        let element_id = self.push_element(
            parent_id,
            element,
            outline_path,
            context,
            element_summary(element),
        );
        match &element.data {
            ElementData::Paragraph(objects) => {
                self.collect_objects(objects, outline_path, "paragraph", element_id)
            }
            ElementData::Keyword(keyword) | ElementData::BabelCall(keyword) => {
                self.collect_objects(&keyword.parsed, outline_path, "keywordValue", element_id);
            }
            ElementData::Drawer(drawer) => {
                self.collect_elements(&drawer.children, outline_path, "drawer", element_id);
            }
            ElementData::List(list) => {
                for item in &list.items {
                    self.collect_list_item(item, outline_path, element_id);
                }
            }
            ElementData::Table(table) => {
                for row in &table.rows {
                    let row_id = self.push(
                        Some(element_id),
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
                        let cell_id = self.push(
                            Some(row_id),
                            OrgElementsIndexCategory::Object,
                            "table-cell",
                            &cell.ann,
                            outline_path,
                            "tableRow",
                            summary([("objects", cell.objects.len().into())]),
                        );
                        self.collect_objects(&cell.objects, outline_path, "tableCell", cell_id);
                    }
                }
            }
            ElementData::Block(block) => {
                self.collect_elements(&block.children, outline_path, "block", element_id)
            }
            ElementData::FootnoteDef(footnote) => {
                self.collect_elements(
                    &footnote.children,
                    outline_path,
                    "footnoteDefinition",
                    element_id,
                );
            }
            ElementData::Inlinetask(task) => {
                self.collect_objects(&task.title, outline_path, "inlinetaskTitle", element_id);
                self.collect_elements(&task.children, outline_path, "inlinetask", element_id);
            }
            ElementData::Clock(_)
            | ElementData::PropertyDrawer(_)
            | ElementData::TableEl { .. }
            | ElementData::Comment(_)
            | ElementData::DiarySexp(_)
            | ElementData::FixedWidth(_)
            | ElementData::Rule
            | ElementData::LatexEnvironment(_)
            | ElementData::Unknown { .. } => {}
        }
    }

    fn collect_list_item(
        &mut self,
        item: &ListItem<ParsedAnnotation>,
        outline_path: &[String],
        parent_id: OrgElementId,
    ) {
        let item_id = self.push(
            Some(parent_id),
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
        self.collect_objects(&item.tag, outline_path, "listItemTag", item_id);
        self.collect_elements(&item.children, outline_path, "listItem", item_id);
    }

    fn collect_objects(
        &mut self,
        objects: &[Object<ParsedAnnotation>],
        outline_path: &[String],
        context: &str,
        parent_id: OrgElementId,
    ) {
        for object in objects {
            self.collect_object(object, outline_path, context, parent_id);
        }
    }

    fn collect_object(
        &mut self,
        object: &Object<ParsedAnnotation>,
        outline_path: &[String],
        context: &str,
        parent_id: OrgElementId,
    ) {
        let object_id = self.push(
            Some(parent_id),
            OrgElementsIndexCategory::Object,
            object_kind(object),
            &object.ann,
            outline_path,
            context,
            object_summary(object),
        );
        match &object.data {
            ObjectData::Markup { children, .. } => {
                self.collect_objects(children, outline_path, "markup", object_id)
            }
            ObjectData::FootnoteRef { definition, .. } => {
                self.collect_objects(definition, outline_path, "footnoteReference", object_id)
            }
            ObjectData::Citation(citation) => {
                self.collect_citation(citation, outline_path, object_id)
            }
            ObjectData::Cloze { text, .. } => {
                self.collect_objects(text, outline_path, "cloze", object_id)
            }
            ObjectData::Link(link) => {
                if link.has_description() {
                    self.collect_objects(
                        &link.description,
                        outline_path,
                        "linkDescription",
                        object_id,
                    );
                } else {
                    self.collect_objects(
                        &link.default_description,
                        outline_path,
                        "linkDefaultDescription",
                        object_id,
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

    fn collect_citation(
        &mut self,
        citation: &Citation<ParsedAnnotation>,
        outline_path: &[String],
        parent_id: OrgElementId,
    ) {
        self.collect_objects(&citation.prefix, outline_path, "citationPrefix", parent_id);
        self.collect_objects(&citation.suffix, outline_path, "citationSuffix", parent_id);
        for reference in &citation.references {
            let reference_id = self.push(
                Some(parent_id),
                OrgElementsIndexCategory::Object,
                "citation-reference",
                &reference.ann,
                outline_path,
                "citation",
                summary([("key", reference.id.clone().into())]),
            );
            self.collect_objects(
                &reference.prefix,
                outline_path,
                "citationReferencePrefix",
                reference_id,
            );
            self.collect_objects(
                &reference.suffix,
                outline_path,
                "citationReferenceSuffix",
                reference_id,
            );
        }
    }

    fn push_keyword(
        &mut self,
        keyword: &Keyword<ParsedAnnotation>,
        outline_path: &[String],
        context: &str,
        parent_id: OrgElementId,
    ) {
        let keyword_id = self.push(
            Some(parent_id),
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
        self.collect_objects(&keyword.parsed, outline_path, "keywordValue", keyword_id);
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
        ElementData::DiarySexp(_) => "diary-sexp",
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
        | ElementData::DiarySexp(raw)
        | ElementData::LatexEnvironment(raw)
        | ElementData::Unknown { raw, .. } => summary([("raw", raw.clone().into())]),
        ElementData::FixedWidth(fixed) => summary([("valueBytes", fixed.value.len().into())]),
        ElementData::TableEl { raw } => summary([("raw", raw.clone().into())]),
        ElementData::Rule => empty_summary(),
    }
}

fn element_affiliated_properties(
    element: &Element<ParsedAnnotation>,
) -> OrgElementsAffiliatedProperties {
    OrgElementsAffiliatedProperties {
        name: affiliated_keyword_value(&element.affiliated_keywords, "NAME"),
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

fn properties_from_summary(summary: &OrgElementsIndexSummary) -> OrgElementProperties {
    summary
        .iter()
        .map(|(key, value)| (org_property_key(key), value.clone()))
        .collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StandardProperties {
    contents_begin: Option<usize>,
    contents_end: Option<usize>,
    post_blank: Option<usize>,
}

impl StandardProperties {
    fn from_content_ann(ann: &ParsedAnnotation) -> Self {
        Self {
            contents_begin: Some(range_start(ann)),
            contents_end: Some(range_end(ann)),
            post_blank: None,
        }
    }
}

fn properties_with_standard_properties(
    mut properties: OrgElementProperties,
    ann: &ParsedAnnotation,
    parent_id: Option<OrgElementId>,
    standard: StandardProperties,
) -> OrgElementProperties {
    let begin = range_start(ann);
    let end = range_end(ann);
    insert_property(&mut properties, ":begin", begin.into());
    insert_property(&mut properties, ":end", end.into());
    insert_property(&mut properties, ":post-affiliated", begin.into());
    insert_property(
        &mut properties,
        ":contents-begin",
        optional_usize(standard.contents_begin),
    );
    insert_property(
        &mut properties,
        ":contents-end",
        optional_usize(standard.contents_end),
    );
    insert_property(
        &mut properties,
        ":post-blank",
        optional_usize(standard.post_blank),
    );
    insert_property(
        &mut properties,
        ":parent",
        optional_usize(parent_id.map(OrgElementId::as_usize)),
    );
    properties
}

fn range_start(ann: &ParsedAnnotation) -> usize {
    u32::from(ann.range.start()) as usize
}

fn range_end(ann: &ParsedAnnotation) -> usize {
    u32::from(ann.range.end()) as usize
}

fn has_planning(planning: &super::Planning) -> bool {
    planning.scheduled.is_some() || planning.deadline.is_some() || planning.closed.is_some()
}

fn planning_summary(planning: &super::Planning) -> OrgElementsIndexSummary {
    summary([
        (
            "scheduled",
            optional_text(
                planning
                    .scheduled
                    .as_ref()
                    .map(|timestamp| timestamp.raw.as_str()),
            ),
        ),
        (
            "deadline",
            optional_text(
                planning
                    .deadline
                    .as_ref()
                    .map(|timestamp| timestamp.raw.as_str()),
            ),
        ),
        (
            "closed",
            optional_text(
                planning
                    .closed
                    .as_ref()
                    .map(|timestamp| timestamp.raw.as_str()),
            ),
        ),
    ])
}

fn headline_properties(
    section: &Section<ParsedAnnotation>,
    summary: &OrgElementsIndexSummary,
) -> OrgElementProperties {
    let mut properties = properties_from_summary(summary);
    insert_property(
        &mut properties,
        ":raw-value",
        section.raw_title.trim_end().into(),
    );
    insert_property(
        &mut properties,
        ":title",
        section.raw_title.trim_end().into(),
    );
    insert_property(&mut properties, ":level", section.level.into());
    insert_property(&mut properties, ":true-level", section.level.into());
    insert_property(
        &mut properties,
        ":todo-keyword",
        optional_text(section.todo.as_ref().map(|todo| todo.name.as_str())),
    );
    insert_property(
        &mut properties,
        ":todo-type",
        optional_text(
            section
                .todo
                .as_ref()
                .map(|todo| todo_state_label(todo.state)),
        ),
    );
    insert_property(
        &mut properties,
        ":priority",
        optional_text(section.priority.raw_cookie()),
    );
    insert_property(&mut properties, ":tags", section.tags.clone().into());
    insert_property(
        &mut properties,
        ":scheduled",
        optional_text(
            section
                .planning
                .scheduled
                .as_ref()
                .map(|timestamp| timestamp.raw.as_str()),
        ),
    );
    insert_property(
        &mut properties,
        ":deadline",
        optional_text(
            section
                .planning
                .deadline
                .as_ref()
                .map(|timestamp| timestamp.raw.as_str()),
        ),
    );
    insert_property(
        &mut properties,
        ":closed",
        optional_text(
            section
                .planning
                .closed
                .as_ref()
                .map(|timestamp| timestamp.raw.as_str()),
        ),
    );
    for property in &section.effective_properties {
        insert_property(
            &mut properties,
            format!(":{}", property.key),
            property.value.clone().into(),
        );
        insert_property(
            &mut properties,
            format!(":{}", property.key.to_ascii_uppercase()),
            property.value.clone().into(),
        );
    }
    properties
}

fn insert_property(
    properties: &mut OrgElementProperties,
    key: impl AsRef<str>,
    value: OrgElementsIndexSummaryValue,
) {
    properties.insert(org_property_key(key.as_ref()), value);
}

fn org_property_key(key: &str) -> String {
    if key.starts_with(':') {
        key.to_string()
    } else {
        format!(":{key}")
    }
}

fn todo_state_label(state: TodoState) -> &'static str {
    match state {
        TodoState::Todo => "todo",
        TodoState::Done => "done",
    }
}

fn affiliated_keyword_value(keywords: &[Keyword<ParsedAnnotation>], key: &str) -> Option<String> {
    keywords
        .iter()
        .find(|keyword| keyword.key.eq_ignore_ascii_case(key))
        .map(|keyword| keyword.value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn optional_text(value: Option<&str>) -> OrgElementsIndexSummaryValue {
    value
        .map(OrgElementsIndexSummaryValue::from)
        .unwrap_or(OrgElementsIndexSummaryValue::Null)
}

fn optional_usize(value: Option<usize>) -> OrgElementsIndexSummaryValue {
    value
        .map(OrgElementsIndexSummaryValue::from)
        .unwrap_or(OrgElementsIndexSummaryValue::Null)
}
