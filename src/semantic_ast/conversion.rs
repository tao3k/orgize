//! Conversion from the lossless syntax tree into the semantic AST.

use super::postprocess::finalize_document;
use super::source_position::LineIndex;
use super::targets::TargetIndex;
use super::{
    Diagnostic, Document, Element, ElementData, LinkAbbreviation, ParsedAnnotation, ParsedAst,
    Property, Section,
};
use crate::{
    config::ParseConfig,
    syntax::{SyntaxKind, SyntaxNode},
};

impl ParsedAst {
    pub fn from_syntax_tree(root: &SyntaxNode, source: &str) -> Self {
        let config = ParseConfig::default();
        Self::from_syntax_tree_with_config(root, source, &config)
    }

    pub fn from_syntax_tree_with_config(
        root: &SyntaxNode,
        source: &str,
        config: &ParseConfig,
    ) -> Self {
        Converter::new(source, config).document(root)
    }
}

pub(super) struct Converter<'a> {
    pub(super) source: &'a str,
    pub(super) config: &'a ParseConfig,
    pub(super) lines: LineIndex<'a>,
    pub(super) diagnostics: Vec<Diagnostic>,
    pub(super) radio_targets: Vec<String>,
    pub(super) target_index: TargetIndex,
    pub(super) link_abbreviations: Vec<LinkAbbreviation>,
}

#[derive(Default)]
pub(super) struct DocumentParts {
    properties: Vec<Property<ParsedAnnotation>>,
    pub(super) children: Vec<Element<ParsedAnnotation>>,
    sections: Vec<Section<ParsedAnnotation>>,
}

impl<'a> Converter<'a> {
    pub(super) fn new(source: &'a str, config: &'a ParseConfig) -> Self {
        Self {
            source,
            config,
            lines: LineIndex::new(source),
            diagnostics: Vec::new(),
            radio_targets: Vec::new(),
            target_index: TargetIndex::default(),
            link_abbreviations: Vec::new(),
        }
    }

    fn document(mut self, root: &SyntaxNode) -> ParsedAst {
        let prescan = self.semantic_prescan(root);
        let target_index = prescan.target_index;
        self.radio_targets = target_index.radio_targets();
        self.target_index = target_index;
        self.link_abbreviations = prescan.link_abbreviations.clone();
        self.diagnostics = prescan.diagnostics;
        let ann = self.node_ann(root);
        let parts = root
            .children()
            .fold(DocumentParts::default(), |mut parts, node| {
                self.push_document_child(&mut parts, &node);
                parts
            });
        let targets = std::mem::take(&mut self.target_index.definitions);

        let mut properties = prescan.properties;
        properties.extend(parts.properties);

        let mut document = Document {
            ann,
            properties,
            archive_locations: prescan.archive_locations,
            metadata: prescan.metadata,
            filetags: prescan.filetags,
            tag_definitions: prescan.tag_definitions,
            export_settings: prescan.export_settings,
            link_abbreviations: prescan.link_abbreviations,
            includes: prescan.includes,
            macro_definitions: prescan.macro_definitions,
            targets,
            footnotes: prescan.footnotes,
            children: parts.children,
            sections: parts.sections,
            diagnostics: self.diagnostics,
        };
        finalize_document(&mut document);
        document
    }

    fn push_document_child(&mut self, parts: &mut DocumentParts, node: &SyntaxNode) {
        match node.kind() {
            SyntaxKind::PROPERTY_DRAWER => {
                parts.properties.extend(self.properties(node));
            }
            SyntaxKind::SECTION => {
                let section_children = self.elements_from_container(node);
                parts.properties.extend(
                    section_children
                        .iter()
                        .filter_map(|child| match &child.data {
                            ElementData::PropertyDrawer(properties) => Some(properties),
                            _ => None,
                        })
                        .flatten()
                        .cloned(),
                );
                parts.children.extend(section_children);
            }
            SyntaxKind::HEADLINE => parts.sections.push(self.section(node)),
            _ => {}
        }
    }
}
