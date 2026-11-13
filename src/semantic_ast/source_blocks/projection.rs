//! Consumer-specific projections over the shared source-block traversal.
use super::traversal::{
    affiliated_keyword_value, inline_result_from_object, source_block_result_from_element,
};
use crate::ast::{
    Block, Element, Object, ObjectData, ParsedAnnotation, Property, SourceBlockRecord,
    SourceBlockRecordKind, SourceBlockSource,
    source_block_execution::source_block_execution_plan,
    source_block_headers::{
        explicit_inline_source_header_args, explicit_source_block_header_args,
        source_block_header_args, source_block_result_options, source_block_tangle,
    },
};

pub(super) trait SourceBlockProjection {
    fn block(
        &mut self,
        element: &Element<ParsedAnnotation>,
        block: &Block<ParsedAnnotation>,
        properties: &[Property<ParsedAnnotation>],
        next: Option<&Element<ParsedAnnotation>>,
    );
    fn inline(
        &mut self,
        object: &Object<ParsedAnnotation>,
        properties: &[Property<ParsedAnnotation>],
        next: Option<&Object<ParsedAnnotation>>,
    );
}

impl SourceBlockProjection for Vec<SourceBlockRecord> {
    fn block(
        &mut self,
        element: &Element<ParsedAnnotation>,
        block: &Block<ParsedAnnotation>,
        properties: &[Property<ParsedAnnotation>],
        next: Option<&Element<ParsedAnnotation>>,
    ) {
        let header_args = explicit_source_block_header_args(
            element,
            block.language.as_deref(),
            properties,
            &block.header_args,
        );
        let normalized_header_args =
            source_block_header_args(SourceBlockRecordKind::Block, &header_args);
        self.push(SourceBlockRecord {
            source: SourceBlockSource::from_annotation(&element.ann),
            kind: SourceBlockRecordKind::Block,
            name: affiliated_keyword_value(&element.affiliated_keywords, "NAME"),
            language: block.language.clone(),
            parameters: block.parameters.clone(),
            header_args: header_args.clone(),
            result_options: source_block_result_options(&normalized_header_args),
            execution: source_block_execution_plan(&normalized_header_args),
            normalized_header_args,
            code_refs: block.code_refs.clone(),
            tangle: source_block_tangle(&header_args),
            result: next.and_then(source_block_result_from_element),
            value: block.value.clone(),
        });
    }
    fn inline(
        &mut self,
        object: &Object<ParsedAnnotation>,
        properties: &[Property<ParsedAnnotation>],
        next: Option<&Object<ParsedAnnotation>>,
    ) {
        if let ObjectData::InlineSrc {
            language,
            parameters,
            value,
            ..
        } = &object.data
        {
            let header_args = explicit_inline_source_header_args(
                language.as_str(),
                properties,
                &object.ann.header_args,
            );
            let normalized_header_args =
                source_block_header_args(SourceBlockRecordKind::InlineSource, &header_args);
            self.push(SourceBlockRecord {
                source: SourceBlockSource::from_annotation(&object.ann),
                kind: SourceBlockRecordKind::InlineSource,
                name: None,
                language: Some(language.clone()),
                parameters: parameters.clone(),
                result_options: source_block_result_options(&normalized_header_args),
                execution: source_block_execution_plan(&normalized_header_args),
                normalized_header_args,
                header_args: header_args.clone(),
                code_refs: Vec::new(),
                tangle: source_block_tangle(&header_args),
                result: next.and_then(inline_result_from_object),
                value: value.clone(),
            });
        }
    }
}
