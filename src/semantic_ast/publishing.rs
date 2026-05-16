//! Publishing metadata projection over ordinary Org keywords.

use super::{
    Document, Element, ElementData, Keyword, PublishingAttribute, PublishingBind,
    PublishingKeyword, PublishingOption, PublishingOptionKind, PublishingSettings, Section,
};

impl<A: Clone> Document<A> {
    /// Projects publishing/export settings without executing export behavior.
    ///
    /// This API keeps publishing out of core parsing while making document
    /// intent visible to lint, site generation, and frontend consumers.
    pub fn publishing_settings(&self) -> PublishingSettings<A> {
        let mut settings = PublishingSettings {
            includes: self.includes.clone(),
            ..PublishingSettings::default()
        };
        for element in &self.children {
            collect_publishing_from_element(element, &mut settings);
        }
        for section in &self.sections {
            collect_publishing_from_section(section, &mut settings);
        }
        settings
    }
}

fn collect_publishing_from_section<A: Clone>(
    section: &Section<A>,
    settings: &mut PublishingSettings<A>,
) {
    for element in &section.children {
        collect_publishing_from_element(element, settings);
    }
    for subsection in &section.subsections {
        collect_publishing_from_section(subsection, settings);
    }
}

fn collect_publishing_from_element<A: Clone>(
    element: &Element<A>,
    settings: &mut PublishingSettings<A>,
) {
    for keyword in &element.affiliated_keywords {
        collect_publishing_keyword(keyword, settings);
    }
    match &element.data {
        ElementData::Keyword(keyword) | ElementData::BabelCall(keyword) => {
            collect_publishing_keyword(keyword, settings);
        }
        ElementData::Drawer(drawer) => {
            for child in &drawer.children {
                collect_publishing_from_element(child, settings);
            }
        }
        ElementData::List(list) => {
            for item in &list.items {
                for child in &item.children {
                    collect_publishing_from_element(child, settings);
                }
            }
        }
        ElementData::Block(block) => {
            for child in &block.children {
                collect_publishing_from_element(child, settings);
            }
        }
        ElementData::FootnoteDef(footnote) => {
            for child in &footnote.children {
                collect_publishing_from_element(child, settings);
            }
        }
        ElementData::Inlinetask(task) => {
            for child in &task.children {
                collect_publishing_from_element(child, settings);
            }
        }
        ElementData::Paragraph(_)
        | ElementData::Clock(_)
        | ElementData::PropertyDrawer(_)
        | ElementData::Table(_)
        | ElementData::TableEl { .. }
        | ElementData::Comment(_)
        | ElementData::FixedWidth(_)
        | ElementData::Rule
        | ElementData::LatexEnvironment(_)
        | ElementData::Unknown { .. } => {}
    }
}

fn collect_publishing_keyword<A: Clone>(
    keyword: &Keyword<A>,
    settings: &mut PublishingSettings<A>,
) {
    let key = keyword.key.to_ascii_uppercase();
    match key.as_str() {
        "EXPORT_FILE_NAME" => {
            settings.export_file_name = Some(publishing_keyword(keyword));
        }
        "SETUPFILE" => settings.setup_files.push(publishing_keyword(keyword)),
        "BIND" => {
            if let Some(bind) = publishing_bind(keyword) {
                settings.binds.push(bind);
            }
        }
        "OPTIONS" => {
            settings.options.extend(publishing_options(keyword));
        }
        key if key.starts_with("ATTR_") => settings.attributes.push(publishing_attribute(
            keyword,
            key.trim_start_matches("ATTR_"),
        )),
        key if is_backend_export_keyword(key) => {
            settings.backend_keywords.push(publishing_keyword(keyword));
        }
        _ => {}
    }
}

fn publishing_keyword<A: Clone>(keyword: &Keyword<A>) -> PublishingKeyword<A> {
    PublishingKeyword {
        ann: keyword.ann.clone(),
        key: keyword.key.clone(),
        value: keyword.value.trim().to_string(),
    }
}

fn publishing_bind<A: Clone>(keyword: &Keyword<A>) -> Option<PublishingBind<A>> {
    let raw = keyword.value.trim();
    let (name, value) = raw
        .split_once(char::is_whitespace)
        .map(|(name, value)| (name.trim(), value.trim()))
        .unwrap_or((raw, ""));
    (!name.is_empty()).then(|| PublishingBind {
        ann: keyword.ann.clone(),
        name: name.to_string(),
        value: value.to_string(),
        raw: keyword.value.clone(),
    })
}

fn publishing_options<A: Clone>(keyword: &Keyword<A>) -> Vec<PublishingOption<A>> {
    keyword
        .value
        .split_whitespace()
        .filter_map(|token| {
            let (key, value) = token.split_once(':')?;
            Some(PublishingOption {
                ann: keyword.ann.clone(),
                key: key.to_string(),
                value: value.to_string(),
                raw: token.to_string(),
                kind: publishing_option_kind(key),
            })
        })
        .collect()
}

fn publishing_option_kind(key: &str) -> PublishingOptionKind {
    match key {
        "H" => PublishingOptionKind::HeadlineLevels,
        "num" => PublishingOptionKind::SectionNumbering,
        "-" => PublishingOptionKind::SpecialStrings,
        "e" => PublishingOptionKind::Entities,
        "todo" => PublishingOptionKind::TodoKeywords,
        "tags" => PublishingOptionKind::Tags,
        "<" => PublishingOptionKind::Timestamps,
        "author" => PublishingOptionKind::Author,
        "creator" => PublishingOptionKind::Creator,
        "date" => PublishingOptionKind::Date,
        "email" => PublishingOptionKind::Email,
        "title" => PublishingOptionKind::Title,
        "d" => PublishingOptionKind::Drawers,
        "p" => PublishingOptionKind::Planning,
        "pri" => PublishingOptionKind::Priorities,
        "broken-links" => PublishingOptionKind::BrokenLinks,
        _ => PublishingOptionKind::Other,
    }
}

fn publishing_attribute<A: Clone>(keyword: &Keyword<A>, backend: &str) -> PublishingAttribute<A> {
    PublishingAttribute {
        ann: keyword.ann.clone(),
        backend: backend.to_ascii_lowercase(),
        optional: keyword.optional.clone(),
        attributes: keyword.attributes.clone(),
        raw: keyword.value.clone(),
    }
}

fn is_backend_export_keyword(key: &str) -> bool {
    key.starts_with("HTML_")
        || key.starts_with("LATEX_")
        || key.starts_with("MD_")
        || key.starts_with("BEAMER_")
        || key.starts_with("ODT_")
        || matches!(key, "EXPORT_TITLE" | "EXPORT_AUTHOR" | "EXPORT_DATE")
}
