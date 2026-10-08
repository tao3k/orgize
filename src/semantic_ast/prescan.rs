//! Document-level semantic prescan state and keyword routing.

use super::settings::{KeywordFacts, link_abbreviation_from_facts};
use super::{
    ArchiveLocation, Diagnostic, ExportSettings, IncludeDirective, Keyword, LinkAbbreviation,
    MacroDefinition, OrgDuration, ParsedAnnotation, Property, TagDefinition,
};

#[derive(Default)]
pub(super) struct SemanticPrescan {
    pub(super) metadata: Vec<Keyword<ParsedAnnotation>>,
    pub(super) filetags: Vec<String>,
    pub(super) tag_definitions: Vec<TagDefinition>,
    pub(super) properties: Vec<Property<ParsedAnnotation>>,
    pub(super) archive_locations: Vec<ArchiveLocation<ParsedAnnotation>>,
    pub(super) export_settings: ExportSettings,
    pub(super) link_abbreviations: Vec<LinkAbbreviation>,
    pub(super) includes: Vec<IncludeDirective<ParsedAnnotation>>,
    pub(super) macro_definitions: Vec<MacroDefinition<ParsedAnnotation>>,
    pub(super) diagnostics: Vec<Diagnostic>,
}

pub(super) fn collect_document_keyword(
    keyword: Keyword<ParsedAnnotation>,
    facts: KeywordFacts,
    prescan: &mut SemanticPrescan,
) {
    match facts.field("route").expect("native keyword route") {
        "TITLE" | "AUTHOR" | "DATE" | "CAPTION" | "PYTHON" | "PYTHON_FILE" | "PYTHON-FILE"
        | "READONLY" | "ALLPRIORITIES" | "CONTRACT_ORG" => {
            prescan.metadata.push(keyword);
        }
        "FILETAGS" => {
            for tag in facts.values("tag") {
                push_unique(&mut prescan.filetags, tag);
            }
            prescan.metadata.push(keyword);
        }
        "OPTIONS" => {
            facts.apply_options(&mut prescan.export_settings);
            prescan.metadata.push(keyword);
        }
        "PROPERTY" => {
            if let Some(property) = keyword_property(&keyword, &facts) {
                prescan.properties.push(property);
            }
            prescan.metadata.push(keyword);
        }
        "ARCHIVE" => {
            prescan.archive_locations.push(ArchiveLocation::from_value(
                keyword.ann.clone(),
                keyword.value.clone(),
            ));
            prescan.metadata.push(keyword);
        }
        "SELECT_TAGS" => {
            prescan.export_settings.select_tags = facts.values("word");
            prescan.metadata.push(keyword);
        }
        "EXCLUDE_TAGS" => {
            prescan.export_settings.exclude_tags = facts.values("word");
            prescan.metadata.push(keyword);
        }
        "LINK" => {
            if let Some(abbreviation) = link_abbreviation_from_facts(&keyword, &facts) {
                prescan.link_abbreviations.push(abbreviation);
            }
            prescan.metadata.push(keyword);
        }
        _ => {}
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn keyword_property(
    keyword: &Keyword<ParsedAnnotation>,
    facts: &KeywordFacts,
) -> Option<Property<ParsedAnnotation>> {
    let key = facts
        .field("first")
        .expect("native keyword name")
        .to_owned();
    let rest = facts.field("rest").expect("native keyword rest").to_owned();
    (!key.is_empty()).then(|| Property {
        ann: keyword.ann.clone(),
        key,
        value: rest.clone(),
        duration: OrgDuration::parse(rest),
    })
}
