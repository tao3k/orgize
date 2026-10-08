//! Raw source syntax for Contract and syntax lint consumers.
use super::{projection::SourceBlockProjection, traversal::affiliated_keyword_value};
use crate::ast::{
    Block, Element, Object, ObjectData, ParsedAnnotation, Property, SourceBlockRecordKind,
    SourceBlockSource,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SourceBlockSyntaxRecord {
    pub source: SourceBlockSource,
    pub kind: SourceBlockRecordKind,
    pub name: Option<String>,
    pub language: Option<String>,
    pub parameters: Option<String>,
    pub value: String,
}

impl SourceBlockProjection for Vec<SourceBlockSyntaxRecord> {
    fn block(
        &mut self,
        element: &Element<ParsedAnnotation>,
        block: &Block<ParsedAnnotation>,
        _properties: &[Property<ParsedAnnotation>],
        _next: Option<&Element<ParsedAnnotation>>,
    ) {
        self.push(SourceBlockSyntaxRecord {
            source: SourceBlockSource::from_annotation(&element.ann),
            kind: SourceBlockRecordKind::Block,
            name: affiliated_keyword_value(&element.affiliated_keywords, "NAME"),
            language: block.language.clone(),
            parameters: block.parameters.clone(),
            value: block.value.clone(),
        });
    }
    fn inline(
        &mut self,
        object: &Object<ParsedAnnotation>,
        _properties: &[Property<ParsedAnnotation>],
        _next: Option<&Object<ParsedAnnotation>>,
    ) {
        if let ObjectData::InlineSrc {
            language,
            parameters,
            value,
            ..
        } = &object.data
        {
            self.push(SourceBlockSyntaxRecord {
                source: SourceBlockSource::from_annotation(&object.ann),
                kind: SourceBlockRecordKind::InlineSource,
                name: None,
                language: Some(language.clone()),
                parameters: parameters.clone(),
                value: value.clone(),
            });
        }
    }
}
