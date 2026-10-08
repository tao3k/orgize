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
        let mut keywords = Vec::new();
        for element in &self.children {
            collect_publishing_from_element(element, &mut keywords);
        }
        for section in &self.sections {
            collect_publishing_from_section(section, &mut keywords);
        }
        project_publishing_keywords(&keywords, &mut settings);
        settings
    }
}

fn collect_publishing_from_section<'a, A>(
    section: &'a Section<A>,
    keywords: &mut Vec<&'a Keyword<A>>,
) {
    for element in &section.children {
        collect_publishing_from_element(element, keywords);
    }
    for subsection in &section.subsections {
        collect_publishing_from_section(subsection, keywords);
    }
}

fn collect_publishing_from_element<'a, A>(
    element: &'a Element<A>,
    keywords: &mut Vec<&'a Keyword<A>>,
) {
    for keyword in &element.affiliated_keywords {
        keywords.push(keyword);
    }
    match &element.data {
        ElementData::Keyword(keyword) | ElementData::BabelCall(keyword) => {
            keywords.push(keyword);
        }
        ElementData::Drawer(drawer) => {
            for child in &drawer.children {
                collect_publishing_from_element(child, keywords);
            }
        }
        ElementData::List(list) => {
            for item in &list.items {
                for child in &item.children {
                    collect_publishing_from_element(child, keywords);
                }
            }
        }
        ElementData::Block(block) => {
            for child in &block.children {
                collect_publishing_from_element(child, keywords);
            }
        }
        ElementData::FootnoteDef(footnote) => {
            for child in &footnote.children {
                collect_publishing_from_element(child, keywords);
            }
        }
        ElementData::Inlinetask(task) => {
            for child in &task.children {
                collect_publishing_from_element(child, keywords);
            }
        }
        ElementData::Paragraph(_)
        | ElementData::Clock(_)
        | ElementData::PropertyDrawer(_)
        | ElementData::Table(_)
        | ElementData::TableEl { .. }
        | ElementData::Comment(_)
        | ElementData::DiarySexp(_)
        | ElementData::FixedWidth(_)
        | ElementData::Rule
        | ElementData::LatexEnvironment(_)
        | ElementData::Unknown { .. } => {}
    }
}

fn project_publishing_keywords<A: Clone>(
    keywords: &[&Keyword<A>],
    settings: &mut PublishingSettings<A>,
) {
    // Scheme returns closed tagged rows in source order. Rust admits their shape
    // and attaches graph annotations; it never reinterprets the keyword text.
    if keywords.is_empty() {
        return;
    }
    let request = keywords
        .iter()
        .flat_map(|keyword| [keyword.key.as_str(), keyword.value.as_str()])
        .collect::<Vec<_>>();
    let mut previous = 0;
    for row in super::org_native_values::rows("publishing-keywords", &request) {
        let (index, row) = row.split_first().expect("native publishing index");
        let index = index
            .parse::<usize>()
            .expect("native publishing index integer");
        assert!(index >= previous, "native publishing source order");
        previous = index;
        let keyword = keywords.get(index).expect("native publishing index bounds");
        let fields = row.iter().map(String::as_str).collect::<Vec<_>>();
        match fields.as_slice() {
            [
                tag @ ("export-file-name" | "setup-file" | "backend-keyword"),
                value,
            ] => {
                let projected = PublishingKeyword {
                    ann: keyword.ann.clone(),
                    key: keyword.key.clone(),
                    value: (*value).to_owned(),
                };
                match *tag {
                    "export-file-name" => settings.export_file_name = Some(projected),
                    "setup-file" => settings.setup_files.push(projected),
                    _ => settings.backend_keywords.push(projected),
                }
            }
            ["bind", name, value] => settings.binds.push(PublishingBind {
                ann: keyword.ann.clone(),
                name: (*name).to_owned(),
                value: (*value).to_owned(),
                raw: keyword.value.clone(),
            }),
            ["option", key, value, raw, kind] => settings.options.push(PublishingOption {
                ann: keyword.ann.clone(),
                key: (*key).to_owned(),
                value: (*value).to_owned(),
                raw: (*raw).to_owned(),
                kind: admit_option_kind(kind),
            }),
            ["attribute", backend] => settings.attributes.push(PublishingAttribute {
                ann: keyword.ann.clone(),
                backend: (*backend).to_owned(),
                optional: keyword.optional.clone(),
                attributes: keyword.attributes.clone(),
                raw: keyword.value.clone(),
            }),
            _ => panic!("invalid native publishing row"),
        }
    }
}

// Closed ABI tags only; source classification belongs to Scheme.
fn admit_option_kind(tag: &str) -> PublishingOptionKind {
    match tag {
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
        "other" => PublishingOptionKind::Other,
        _ => panic!("invalid native publishing option tag"),
    }
}
