//! Attachment link lint checks.

use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use crate::ast::{
    Element, ElementData, Link, Object, ObjectData, ParsedAnnotation, ParsedAst, Section,
};

use super::lint_model::{LintFinding, LintOptions, LintSeverity, location_for_range};

pub(crate) fn attachment_findings(
    document: &ParsedAst,
    source: &str,
    options: &LintOptions,
) -> Vec<LintFinding> {
    let Some(base_dir) = &options.attachment_base_dir else {
        return Vec::new();
    };

    let mut findings = Vec::new();
    collect_element_attachment_findings(&document.children, None, source, base_dir, &mut findings);
    for section in &document.sections {
        collect_section_attachment_findings(section, source, base_dir, &mut findings);
    }
    findings
}

fn collect_section_attachment_findings(
    section: &Section<ParsedAnnotation>,
    source: &str,
    base_dir: &Path,
    findings: &mut Vec<LintFinding>,
) {
    collect_object_attachment_findings(&section.title, section, source, base_dir, findings);
    collect_element_attachment_findings(
        &section.children,
        Some(section),
        source,
        base_dir,
        findings,
    );
    for subsection in &section.subsections {
        collect_section_attachment_findings(subsection, source, base_dir, findings);
    }
}

fn collect_element_attachment_findings(
    elements: &[Element<ParsedAnnotation>],
    section: Option<&Section<ParsedAnnotation>>,
    source: &str,
    base_dir: &Path,
    findings: &mut Vec<LintFinding>,
) {
    for element in elements {
        collect_one_element_attachment_finding(element, section, source, base_dir, findings);
    }
}

fn collect_one_element_attachment_finding(
    element: &Element<ParsedAnnotation>,
    section: Option<&Section<ParsedAnnotation>>,
    source: &str,
    base_dir: &Path,
    findings: &mut Vec<LintFinding>,
) {
    match &element.data {
        ElementData::Paragraph(objects) => {
            collect_context_object_attachment_findings(
                objects, section, source, base_dir, findings,
            );
        }
        ElementData::Drawer(drawer) => {
            collect_element_attachment_findings(
                &drawer.children,
                section,
                source,
                base_dir,
                findings,
            );
        }
        ElementData::List(list) => {
            collect_list_attachment_findings(&list.items, section, source, base_dir, findings);
        }
        ElementData::Table(table) => {
            collect_table_attachment_findings(table, section, source, base_dir, findings);
        }
        ElementData::Block(block) => {
            collect_element_attachment_findings(
                &block.children,
                section,
                source,
                base_dir,
                findings,
            );
        }
        ElementData::FootnoteDef(footnote) => {
            collect_element_attachment_findings(
                &footnote.children,
                section,
                source,
                base_dir,
                findings,
            );
        }
        ElementData::Inlinetask(task) => {
            collect_context_object_attachment_findings(
                &task.title,
                section,
                source,
                base_dir,
                findings,
            );
            collect_element_attachment_findings(
                &task.children,
                section,
                source,
                base_dir,
                findings,
            );
        }
        ElementData::Keyword(_)
        | ElementData::BabelCall(_)
        | ElementData::Clock(_)
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

fn collect_context_object_attachment_findings(
    objects: &[Object<ParsedAnnotation>],
    section: Option<&Section<ParsedAnnotation>>,
    source: &str,
    base_dir: &Path,
    findings: &mut Vec<LintFinding>,
) {
    if let Some(section) = section {
        collect_object_attachment_findings(objects, section, source, base_dir, findings);
    } else {
        collect_root_object_attachment_findings(objects, source, findings);
    }
}

fn collect_list_attachment_findings(
    items: &[crate::ast::ListItem<ParsedAnnotation>],
    section: Option<&Section<ParsedAnnotation>>,
    source: &str,
    base_dir: &Path,
    findings: &mut Vec<LintFinding>,
) {
    for item in items {
        collect_context_object_attachment_findings(&item.tag, section, source, base_dir, findings);
        collect_element_attachment_findings(&item.children, section, source, base_dir, findings);
    }
}

fn collect_table_attachment_findings(
    table: &crate::ast::Table<ParsedAnnotation>,
    section: Option<&Section<ParsedAnnotation>>,
    source: &str,
    base_dir: &Path,
    findings: &mut Vec<LintFinding>,
) {
    for cell in table.rows.iter().flat_map(|row| &row.cells) {
        collect_context_object_attachment_findings(
            &cell.objects,
            section,
            source,
            base_dir,
            findings,
        );
    }
}

fn collect_root_object_attachment_findings(
    objects: &[Object<ParsedAnnotation>],
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    for object in objects {
        collect_one_root_object_attachment_finding(object, source, findings);
    }
}

fn collect_one_root_object_attachment_finding(
    object: &Object<ParsedAnnotation>,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    match &object.data {
        ObjectData::Link(link) if link.attachment.is_some() => {
            findings.push(missing_directory_finding(object, link, source));
        }
        ObjectData::Markup { children, .. } => {
            collect_root_object_attachment_findings(children, source, findings);
        }
        ObjectData::FootnoteRef { definition, .. } => {
            collect_root_object_attachment_findings(definition, source, findings);
        }
        ObjectData::Citation(citation) => {
            collect_root_citation_attachment_findings(citation, source, findings);
        }
        ObjectData::Cloze { text, .. } => {
            collect_root_object_attachment_findings(text, source, findings);
        }
        _ => {}
    }
}

fn collect_root_citation_attachment_findings(
    citation: &crate::ast::Citation<ParsedAnnotation>,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    collect_root_object_attachment_findings(&citation.prefix, source, findings);
    collect_root_object_attachment_findings(&citation.suffix, source, findings);
    for reference in &citation.references {
        collect_root_object_attachment_findings(&reference.prefix, source, findings);
        collect_root_object_attachment_findings(&reference.suffix, source, findings);
    }
}

fn collect_object_attachment_findings(
    objects: &[Object<ParsedAnnotation>],
    section: &Section<ParsedAnnotation>,
    source: &str,
    base_dir: &Path,
    findings: &mut Vec<LintFinding>,
) {
    for object in objects {
        collect_one_object_attachment_finding(object, section, source, base_dir, findings);
    }
}

fn collect_one_object_attachment_finding(
    object: &Object<ParsedAnnotation>,
    section: &Section<ParsedAnnotation>,
    source: &str,
    base_dir: &Path,
    findings: &mut Vec<LintFinding>,
) {
    match &object.data {
        ObjectData::Link(link) => {
            push_attachment_link_finding(object, link, section, source, base_dir, findings);
            collect_object_attachment_findings(
                link.description_or_default(),
                section,
                source,
                base_dir,
                findings,
            );
        }
        ObjectData::Markup { children, .. } => {
            collect_object_attachment_findings(children, section, source, base_dir, findings);
        }
        ObjectData::FootnoteRef { definition, .. } => {
            collect_object_attachment_findings(definition, section, source, base_dir, findings);
        }
        ObjectData::Citation(citation) => {
            collect_citation_attachment_findings(citation, section, source, base_dir, findings);
        }
        ObjectData::Cloze { text, .. } => {
            collect_object_attachment_findings(text, section, source, base_dir, findings);
        }
        _ => {}
    }
}

fn push_attachment_link_finding(
    object: &Object<ParsedAnnotation>,
    link: &Link<ParsedAnnotation>,
    section: &Section<ParsedAnnotation>,
    source: &str,
    base_dir: &Path,
    findings: &mut Vec<LintFinding>,
) {
    if link.attachment.is_none() {
        return;
    }
    if let Some(finding) = attachment_link_finding(object, link, section, source, base_dir) {
        findings.push(finding);
    }
}

fn collect_citation_attachment_findings(
    citation: &crate::ast::Citation<ParsedAnnotation>,
    section: &Section<ParsedAnnotation>,
    source: &str,
    base_dir: &Path,
    findings: &mut Vec<LintFinding>,
) {
    collect_object_attachment_findings(&citation.prefix, section, source, base_dir, findings);
    collect_object_attachment_findings(&citation.suffix, section, source, base_dir, findings);
    for reference in &citation.references {
        collect_object_attachment_findings(&reference.prefix, section, source, base_dir, findings);
        collect_object_attachment_findings(&reference.suffix, section, source, base_dir, findings);
    }
}

fn attachment_link_finding(
    object: &Object<ParsedAnnotation>,
    link: &Link<ParsedAnnotation>,
    section: &Section<ParsedAnnotation>,
    source: &str,
    base_dir: &Path,
) -> Option<LintFinding> {
    let attachment = link.attachment.as_ref()?;
    let Some(directory) = &section.attachment.directory else {
        return Some(missing_directory_finding(object, link, source));
    };
    if attachment.path.trim().is_empty() {
        return Some(LintFinding {
            code: "ORG016",
            severity: LintSeverity::Warning,
            message: "attachment link has no file path".to_string(),
            location: location_for_range(source, object.ann.range),
        });
    }

    let resolved =
        resolve_attachment_path(base_dir, directory.path.as_str(), attachment.path.as_str());
    let message = match fs::metadata(&resolved) {
        Ok(_) => return None,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            format!("attachment target `{}` was not found", link.path())
        }
        Err(error) => format!(
            "attachment target `{}` could not be read: {error}",
            link.path()
        ),
    };

    Some(LintFinding {
        code: "ORG016",
        severity: LintSeverity::Warning,
        message,
        location: location_for_range(source, object.ann.range),
    })
}

fn missing_directory_finding(
    object: &Object<ParsedAnnotation>,
    link: &Link<ParsedAnnotation>,
    source: &str,
) -> LintFinding {
    LintFinding {
        code: "ORG016",
        severity: LintSeverity::Warning,
        message: format!(
            "attachment link `{}` has no DIR, ATTACH_DIR, or ID attachment directory",
            link.path()
        ),
        location: location_for_range(source, object.ann.range),
    }
}

fn resolve_attachment_path(base_dir: &Path, directory: &str, attachment: &str) -> PathBuf {
    let directory = Path::new(directory);
    let root = if directory.is_absolute() {
        directory.to_path_buf()
    } else {
        base_dir.join(directory)
    };
    root.join(attachment)
}
