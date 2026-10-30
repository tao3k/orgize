//! Non-executing Org Cite export planning.

use super::{
    Citation, CitationExportPlan, CitationExportWarning, CitationExportWarningKind, CitationUsage,
    Document, Element, ElementData, Keyword, Object, ObjectData, ParsedAnnotation, Section,
    SectionIndexSource,
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
        let mut keywords: Vec<_> = self.metadata.iter().collect();
        collect_elements(&self.children, &mut plan, &mut keywords);
        for section in &self.sections {
            collect_section(section, &mut plan, &mut keywords);
        }
        super::citation_export_owner::project(&keywords, &mut plan);
        collect_warnings(&mut plan);
        plan
    }
}

fn collect_section<'a>(
    section: &'a Section<ParsedAnnotation>,
    plan: &mut CitationExportPlan<ParsedAnnotation>,
    keywords: &mut Vec<&'a Keyword<ParsedAnnotation>>,
) {
    collect_objects(
        &section.title,
        plan,
        Some(SectionIndexSource::from_annotation(&section.ann)),
    );
    collect_elements(&section.children, plan, keywords);
    for subsection in &section.subsections {
        collect_section(subsection, plan, keywords);
    }
}

fn collect_elements<'a>(
    elements: &'a [Element<ParsedAnnotation>],
    plan: &mut CitationExportPlan<ParsedAnnotation>,
    keywords: &mut Vec<&'a Keyword<ParsedAnnotation>>,
) {
    for element in elements {
        for keyword in &element.affiliated_keywords {
            keywords.push(keyword);
        }
        match &element.data {
            ElementData::Keyword(keyword) | ElementData::BabelCall(keyword) => {
                keywords.push(keyword);
            }
            ElementData::Paragraph(objects) => collect_objects(objects, plan, None),
            ElementData::Drawer(drawer) => collect_elements(&drawer.children, plan, keywords),
            ElementData::List(list) => {
                for item in &list.items {
                    collect_objects(&item.tag, plan, None);
                    collect_elements(&item.children, plan, keywords);
                }
            }
            ElementData::Table(table) => {
                for cell in table.rows.iter().flat_map(|row| &row.cells) {
                    collect_objects(&cell.objects, plan, None);
                }
            }
            ElementData::Block(block) => collect_elements(&block.children, plan, keywords),
            ElementData::FootnoteDef(footnote) => {
                collect_elements(&footnote.children, plan, keywords)
            }
            ElementData::Inlinetask(task) => {
                collect_objects(&task.title, plan, None);
                collect_elements(&task.children, plan, keywords);
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
        // Filled by the batched native style policy after graph traversal.
        nocite: false,
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
