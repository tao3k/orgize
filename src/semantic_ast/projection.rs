//! Opt-in semantic projection helpers for exporter/indexer consumers.

use std::collections::HashSet;

use super::{
    AstMut, Document, ExportProjectionOptions, LinkAbbreviation, LinkTarget, ObjectData, Section,
};

impl<A: Clone> Document<A> {
    /// Returns an exporter-oriented semantic projection without mutating the parsed AST.
    pub fn project_for_export(&self, options: &ExportProjectionOptions) -> Document<A> {
        let mut document = self.clone();
        let select_tags = export_select_tags(&document, options);
        let exclude_tags = export_exclude_tags(&document, options);

        if options.prune {
            let selected_any = !select_tags.is_empty()
                && document
                    .sections
                    .iter()
                    .any(|section| section_has_selected_tag(section, &select_tags));
            document.sections =
                project_sections(document.sections, &select_tags, &exclude_tags, selected_any);
            if selected_any {
                document.children.clear();
            }
        }

        if options.special_strings {
            apply_special_strings(&mut document);
        }

        if options.headline_level_shift != 0 {
            apply_headline_level_shift(&mut document, options.headline_level_shift);
        }

        if options.expand_link_abbreviations {
            expand_link_abbreviations(&mut document);
        }

        document
    }
}

fn export_select_tags<A: Clone>(
    document: &Document<A>,
    options: &ExportProjectionOptions,
) -> Vec<String> {
    if options.select_tags.is_empty() {
        document.export_settings.select_tags.clone()
    } else {
        options.select_tags.clone()
    }
}

fn export_exclude_tags<A: Clone>(
    document: &Document<A>,
    options: &ExportProjectionOptions,
) -> Vec<String> {
    if options.exclude_tags.is_empty() {
        document.export_settings.exclude_tags.clone()
    } else {
        options.exclude_tags.clone()
    }
}

fn project_sections<A: Clone>(
    sections: Vec<Section<A>>,
    select_tags: &[String],
    exclude_tags: &[String],
    selected_any: bool,
) -> Vec<Section<A>> {
    sections
        .into_iter()
        .filter_map(|mut section| {
            if should_exclude(&section, exclude_tags) {
                return None;
            }
            section.subsections =
                project_sections(section.subsections, select_tags, exclude_tags, selected_any);
            if selected_any
                && !has_any_tag(&section.effective_tags, select_tags)
                && section.subsections.is_empty()
            {
                None
            } else {
                Some(section)
            }
        })
        .collect()
}

fn should_exclude<A>(section: &Section<A>, exclude_tags: &[String]) -> bool {
    section.is_comment
        || has_tag(&section.effective_tags, "ARCHIVE")
        || has_any_tag(&section.effective_tags, exclude_tags)
}

fn section_has_selected_tag<A>(section: &Section<A>, select_tags: &[String]) -> bool {
    has_any_tag(&section.effective_tags, select_tags)
        || section
            .subsections
            .iter()
            .any(|child| section_has_selected_tag(child, select_tags))
}

fn has_tag(tags: &[String], expected: &str) -> bool {
    tags.iter().any(|tag| tag.eq_ignore_ascii_case(expected))
}

fn has_any_tag(tags: &[String], expected: &[String]) -> bool {
    let expected = expected
        .iter()
        .map(|tag| tag.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    tags.iter()
        .any(|tag| expected.contains(&tag.to_ascii_lowercase()))
}

fn apply_special_strings<A>(document: &mut Document<A>) {
    document.visit_mut(|node| {
        let AstMut::Object(object) = node else {
            return;
        };
        if let ObjectData::Plain(value) = &mut object.data {
            *value = special_strings(value);
        }
    });
}

fn apply_headline_level_shift<A>(document: &mut Document<A>, shift: isize) {
    document.visit_mut(|node| {
        let AstMut::Section(section) = node else {
            return;
        };
        section.level = section.level.saturating_add_signed(shift).max(1);
    });
}

fn special_strings(value: &str) -> String {
    value
        .replace("---", "\u{2014}")
        .replace("--", "\u{2013}")
        .replace("...", "\u{2026}")
        .replace("\\-", "\u{00AD}")
        .replace('\'', "\u{2019}")
}

fn expand_link_abbreviations<A>(document: &mut Document<A>) {
    let abbreviations = document.link_abbreviations.clone();
    document.visit_mut(|node| {
        let AstMut::Object(object) = node else {
            return;
        };
        let ObjectData::Link(link) = &mut object.data else {
            return;
        };
        let LinkTarget::Uri { protocol, path } = &link.target else {
            return;
        };
        let Some(expanded) = expand_abbreviated_target(protocol, path, &abbreviations) else {
            return;
        };
        if let Some((protocol, path)) = expanded.split_once(':') {
            link.target = LinkTarget::Uri {
                protocol: protocol.to_string(),
                path: path.to_string(),
            };
        }
    });
}

fn expand_abbreviated_target(
    protocol: &str,
    path: &str,
    abbreviations: &[LinkAbbreviation],
) -> Option<String> {
    let replacement = abbreviations
        .iter()
        .find(|abbreviation| abbreviation.name.eq_ignore_ascii_case(protocol))
        .map(|abbreviation| abbreviation.replacement.as_str())?;
    if replacement.contains("%s") || replacement.contains("%h") {
        Some(
            replacement
                .replace("%s", path)
                .replace("%h", &percent_encode(path)),
        )
    } else {
        Some(format!("{replacement}{path}"))
    }
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}
