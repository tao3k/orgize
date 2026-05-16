//! Non-executing Org Cite export planning.

use super::{
    Citation, CitationBibliography, CitationExportOption, CitationExportPlan,
    CitationExportWarning, CitationExportWarningKind, CitationProcessor, CitationUsage, Document,
    Element, ElementData, Keyword, Object, ObjectData, ParsedAnnotation, PrintBibliography,
    Section, SectionIndexSource,
};

impl Document<ParsedAnnotation> {
    /// Collects citation export intent without loading bibliography files or
    /// invoking a citation processor.
    pub fn citation_export_plan(&self) -> CitationExportPlan<ParsedAnnotation> {
        let mut plan = CitationExportPlan {
            bibliographies: Vec::new(),
            processors: Vec::new(),
            print_bibliographies: Vec::new(),
            citations: Vec::new(),
            warnings: Vec::new(),
        };
        for keyword in &self.metadata {
            collect_keyword(keyword, &mut plan);
        }
        collect_elements(&self.children, &mut plan);
        for section in &self.sections {
            collect_section(section, &mut plan);
        }
        collect_warnings(&mut plan);
        plan
    }
}

fn collect_section(
    section: &Section<ParsedAnnotation>,
    plan: &mut CitationExportPlan<ParsedAnnotation>,
) {
    collect_objects(
        &section.title,
        plan,
        Some(SectionIndexSource::from_annotation(&section.ann)),
    );
    collect_elements(&section.children, plan);
    for subsection in &section.subsections {
        collect_section(subsection, plan);
    }
}

fn collect_elements(
    elements: &[Element<ParsedAnnotation>],
    plan: &mut CitationExportPlan<ParsedAnnotation>,
) {
    for element in elements {
        for keyword in &element.affiliated_keywords {
            collect_keyword(keyword, plan);
        }
        match &element.data {
            ElementData::Keyword(keyword) | ElementData::BabelCall(keyword) => {
                collect_keyword(keyword, plan);
            }
            ElementData::Paragraph(objects) => collect_objects(objects, plan, None),
            ElementData::Drawer(drawer) => collect_elements(&drawer.children, plan),
            ElementData::List(list) => {
                for item in &list.items {
                    collect_objects(&item.tag, plan, None);
                    collect_elements(&item.children, plan);
                }
            }
            ElementData::Table(table) => {
                for cell in table.rows.iter().flat_map(|row| &row.cells) {
                    collect_objects(&cell.objects, plan, None);
                }
            }
            ElementData::Block(block) => collect_elements(&block.children, plan),
            ElementData::FootnoteDef(footnote) => collect_elements(&footnote.children, plan),
            ElementData::Inlinetask(task) => {
                collect_objects(&task.title, plan, None);
                collect_elements(&task.children, plan);
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
}

fn collect_keyword(
    keyword: &Keyword<ParsedAnnotation>,
    plan: &mut CitationExportPlan<ParsedAnnotation>,
) {
    let key = keyword.key.to_ascii_uppercase();
    match key.as_str() {
        "BIBLIOGRAPHY" => plan.bibliographies.push(CitationBibliography {
            ann: keyword.ann.clone(),
            files: split_keyword_values(keyword.value.as_str()),
            raw: keyword.value.clone(),
        }),
        "CITE_EXPORT" => plan.processors.push(citation_processor(keyword)),
        "PRINT_BIBLIOGRAPHY" => plan.print_bibliographies.push(PrintBibliography {
            ann: keyword.ann.clone(),
            options: plist_options(keyword.value.as_str()),
            raw: keyword.value.clone(),
        }),
        _ => {}
    }
}

fn citation_processor(keyword: &Keyword<ParsedAnnotation>) -> CitationProcessor<ParsedAnnotation> {
    let mut parts = split_keyword_values(keyword.value.as_str());
    let processor = parts.first().cloned().unwrap_or_default();
    let style = if parts.len() > 1 {
        Some(parts.split_off(1).join(" "))
    } else {
        None
    };
    CitationProcessor {
        ann: keyword.ann.clone(),
        processor,
        style,
        raw: keyword.value.clone(),
    }
}

fn collect_objects(
    objects: &[Object<ParsedAnnotation>],
    plan: &mut CitationExportPlan<ParsedAnnotation>,
    source: Option<SectionIndexSource>,
) {
    for object in objects {
        match &object.data {
            ObjectData::Citation(citation) => {
                plan.citations
                    .push(citation_usage(object, citation, source.clone()));
                collect_objects(&citation.prefix, plan, source.clone());
                collect_objects(&citation.suffix, plan, source.clone());
                for reference in &citation.references {
                    collect_objects(&reference.prefix, plan, source.clone());
                    collect_objects(&reference.suffix, plan, source.clone());
                }
            }
            ObjectData::Markup { children, .. } => collect_objects(children, plan, source.clone()),
            ObjectData::FootnoteRef { definition, .. } => {
                collect_objects(definition, plan, source.clone());
            }
            ObjectData::Link(link) => {
                collect_objects(&link.description, plan, source.clone());
                collect_objects(&link.default_description, plan, source.clone());
            }
            ObjectData::Cloze { text, .. } => collect_objects(text, plan, source.clone()),
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
}

fn citation_usage(
    object: &Object<ParsedAnnotation>,
    citation: &Citation<ParsedAnnotation>,
    source: Option<SectionIndexSource>,
) -> CitationUsage<ParsedAnnotation> {
    let keys = citation
        .references
        .iter()
        .map(|reference| reference.id.clone())
        .collect::<Vec<_>>();
    CitationUsage {
        ann: object.ann.clone(),
        style: citation.style.clone(),
        variant: citation.variant.clone(),
        nocite: citation.style.eq_ignore_ascii_case("nocite"),
        keys,
        raw: object.ann.raw.clone(),
        source,
    }
}

fn collect_warnings(plan: &mut CitationExportPlan<ParsedAnnotation>) {
    if !plan.citations.is_empty() && plan.bibliographies.is_empty() {
        plan.warnings.push(CitationExportWarning {
            kind: CitationExportWarningKind::MissingBibliography,
            message: "citation objects were found but no BIBLIOGRAPHY keyword was collected"
                .to_string(),
        });
    }
    if !plan.print_bibliographies.is_empty() && plan.processors.is_empty() {
        plan.warnings.push(CitationExportWarning {
            kind: CitationExportWarningKind::PrintBibliographyWithoutProcessor,
            message: "PRINT_BIBLIOGRAPHY appears without a CITE_EXPORT processor hint".to_string(),
        });
    }
}

fn split_keyword_values(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .map(|part| part.trim_matches('"').to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

fn plist_options(value: &str) -> Vec<CitationExportOption> {
    let parts = split_keyword_values(value);
    let mut options = Vec::new();
    let mut index = 0;
    while index < parts.len() {
        let raw = parts[index].clone();
        if raw.starts_with(':') {
            let key = raw.trim_start_matches(':').to_string();
            let value = parts
                .get(index + 1)
                .filter(|next| !next.starts_with(':'))
                .cloned();
            if value.is_some() {
                index += 1;
            }
            options.push(CitationExportOption { key, value, raw });
        }
        index += 1;
    }
    options
}
