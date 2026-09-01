//! Org element and object traversal for bridge index construction.

use super::{
    Citation, Element, ElementData, Keyword, ListItem, Object, ObjectData, OrgElementId,
    OrgElementsIndexCategory, ParsedAnnotation, Section,
};

use super::elements_bridge_index::{ElementIndex, ElementIndexRecordSpec};

impl ElementIndex {
    pub(super) fn collect_sections(
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
            let (headline_properties, headline_property_provenance) =
                headline_properties(section, &headline_summary);
            let section_id = self.push(
                ElementIndexRecordSpec::new(
                    Some(parent_id),
                    OrgElementsIndexCategory::Section,
                    "headline",
                    &section.ann,
                    &path,
                    "outline",
                    headline_summary,
                )
                .with_properties(headline_properties)
                .with_property_provenance(headline_property_provenance)
                .with_standard_properties(headline_standard),
            );
            if has_planning(&section.planning) {
                self.push(ElementIndexRecordSpec::new(
                    Some(section_id),
                    OrgElementsIndexCategory::Element,
                    "planning",
                    &section.ann,
                    &path,
                    "headline",
                    planning_summary(&section.planning),
                ));
            }
            self.collect_objects(&section.title, &path, "headlineTitle", section_id);
            self.collect_property_drawer(&section.properties, &path, "headline", section_id);
            let body_parent_id = if let Some(body_ann) = &section.body_ann {
                let body_summary = summary([("elements", section.children.len().into())]);
                self.push(
                    ElementIndexRecordSpec::new(
                        Some(section_id),
                        OrgElementsIndexCategory::Element,
                        "section",
                        body_ann,
                        &path,
                        "headline",
                        body_summary,
                    )
                    .with_standard_properties(StandardProperties::from_content_ann(body_ann)),
                )
            } else {
                section_id
            };
            self.collect_elements(&section.children, &path, "section", body_parent_id);
            self.collect_sections(&section.subsections, path, section_id);
        }
    }

    pub(super) fn collect_property_drawer(
        &mut self,
        properties: &[super::Property<ParsedAnnotation>],
        outline_path: &[String],
        context: &str,
        parent_id: OrgElementId,
    ) {
        let Some(first_property) = properties.first() else {
            return;
        };
        let drawer_id = self.push(ElementIndexRecordSpec::new(
            Some(parent_id),
            OrgElementsIndexCategory::Element,
            "property-drawer",
            &first_property.ann,
            outline_path,
            context,
            summary([("properties", properties.len().into())]),
        ));
        for property in properties {
            self.push(ElementIndexRecordSpec::new(
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
            ));
        }
    }

    pub(super) fn collect_elements(
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

    pub(super) fn collect_element(
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
                let header_row_index = table.rows.iter().position(|row| !row.is_rule);
                let header_cells = header_row_index
                    .and_then(|index| table.rows.get(index))
                    .map(|row| {
                        row.cells
                            .iter()
                            .map(|cell| objects_text(&cell.objects))
                            .collect()
                    })
                    .unwrap_or_else(Vec::new);
                for (row_index, row) in table.rows.iter().enumerate() {
                    let is_header = Some(row_index) == header_row_index;
                    let row_id = self.push(ElementIndexRecordSpec::new(
                        Some(element_id),
                        OrgElementsIndexCategory::Element,
                        "table-row",
                        &row.ann,
                        outline_path,
                        "table",
                        summary([
                            ("rowIndex", (row_index + 1).into()),
                            ("isRule", row.is_rule.into()),
                            ("isHeader", is_header.into()),
                            ("cells", row.cells.len().into()),
                        ]),
                    ));
                    for (column_index, cell) in row.cells.iter().enumerate() {
                        let text = objects_text(&cell.objects);
                        let cell_id = self.push(ElementIndexRecordSpec::new(
                            Some(row_id),
                            OrgElementsIndexCategory::Object,
                            "table-cell",
                            &cell.ann,
                            outline_path,
                            "tableRow",
                            summary([
                                ("rowIndex", (row_index + 1).into()),
                                ("columnIndex", (column_index + 1).into()),
                                (
                                    "columnName",
                                    optional_text(
                                        header_cells.get(column_index).map(|value| value.as_str()),
                                    ),
                                ),
                                ("isHeader", is_header.into()),
                                ("objects", cell.objects.len().into()),
                                ("text", text.clone().into()),
                                ("hasText", (!text.trim().is_empty()).into()),
                            ]),
                        ));
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

    pub(super) fn collect_list_item(
        &mut self,
        item: &ListItem<ParsedAnnotation>,
        outline_path: &[String],
        parent_id: OrgElementId,
    ) {
        let item_id = self.push(ElementIndexRecordSpec::new(
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
        ));
        self.collect_objects(&item.tag, outline_path, "listItemTag", item_id);
        self.collect_elements(&item.children, outline_path, "listItem", item_id);
    }

    pub(super) fn collect_objects(
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

    pub(super) fn collect_object(
        &mut self,
        object: &Object<ParsedAnnotation>,
        outline_path: &[String],
        context: &str,
        parent_id: OrgElementId,
    ) {
        let object_id = self.push(ElementIndexRecordSpec::new(
            Some(parent_id),
            OrgElementsIndexCategory::Object,
            object_kind(object),
            &object.ann,
            outline_path,
            context,
            object_summary(object),
        ));
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

    pub(super) fn collect_citation(
        &mut self,
        citation: &Citation<ParsedAnnotation>,
        outline_path: &[String],
        parent_id: OrgElementId,
    ) {
        self.collect_objects(&citation.prefix, outline_path, "citationPrefix", parent_id);
        self.collect_objects(&citation.suffix, outline_path, "citationSuffix", parent_id);
        for reference in &citation.references {
            let reference_id = self.push(ElementIndexRecordSpec::new(
                Some(parent_id),
                OrgElementsIndexCategory::Object,
                "citation-reference",
                &reference.ann,
                outline_path,
                "citation",
                summary([("key", reference.id.clone().into())]),
            ));
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

    pub(super) fn push_keyword(
        &mut self,
        keyword: &Keyword<ParsedAnnotation>,
        outline_path: &[String],
        context: &str,
        parent_id: OrgElementId,
    ) {
        let keyword_id = self.push(ElementIndexRecordSpec::new(
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
        ));
        self.collect_objects(&keyword.parsed, outline_path, "keywordValue", keyword_id);
    }
}
use super::elements_bridge_index_properties::{
    StandardProperties, has_planning, headline_properties, planning_summary,
};
use super::elements_bridge_index_summary::{
    checkbox, element_summary, object_kind, object_summary, objects_text, optional_text, summary,
};
