//! Document-level semantic prescan state and keyword routing.

use super::settings::{apply_options_keyword, link_abbreviation, parse_tags, split_words};
use super::targets::TargetIndex;
use super::{
    Diagnostic, ExportSettings, FootnoteEntry, IncludeDirective, Keyword, LinkAbbreviation,
    MacroDefinition, ParsedAnnotation,
};

#[derive(Default)]
pub(super) struct SemanticPrescan {
    pub(super) target_index: TargetIndex,
    pub(super) metadata: Vec<Keyword<ParsedAnnotation>>,
    pub(super) filetags: Vec<String>,
    pub(super) export_settings: ExportSettings,
    pub(super) link_abbreviations: Vec<LinkAbbreviation>,
    pub(super) includes: Vec<IncludeDirective<ParsedAnnotation>>,
    pub(super) macro_definitions: Vec<MacroDefinition<ParsedAnnotation>>,
    pub(super) footnotes: Vec<FootnoteEntry<ParsedAnnotation>>,
    pub(super) diagnostics: Vec<Diagnostic>,
}

pub(super) fn collect_document_keyword(
    keyword: Keyword<ParsedAnnotation>,
    prescan: &mut SemanticPrescan,
) {
    let key = keyword.key.to_ascii_uppercase();
    match key.as_str() {
        "TITLE" | "AUTHOR" | "DATE" | "CAPTION" => prescan.metadata.push(keyword),
        "FILETAGS" => {
            prescan.filetags.extend(parse_tags(keyword.value.trim()));
            prescan.metadata.push(keyword);
        }
        "OPTIONS" => {
            apply_options_keyword(keyword.value.trim(), &mut prescan.export_settings);
            prescan.metadata.push(keyword);
        }
        "SELECT_TAGS" => {
            prescan.export_settings.select_tags = split_words(keyword.value.trim());
            prescan.metadata.push(keyword);
        }
        "EXCLUDE_TAGS" => {
            prescan.export_settings.exclude_tags = split_words(keyword.value.trim());
            prescan.metadata.push(keyword);
        }
        "LINK" => {
            if let Some(abbreviation) = link_abbreviation(&keyword) {
                prescan.link_abbreviations.push(abbreviation);
            }
            prescan.metadata.push(keyword);
        }
        _ => {}
    }
}
