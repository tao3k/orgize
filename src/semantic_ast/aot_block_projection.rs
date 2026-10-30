//! Source-backed block projection from Scheme-owned graph fields.

use gerbil_parser_runtime::TextRange;

use super::GraphProjector;
use crate::ast::aot_block_switches::{
    project_block_header_args, project_block_parameters, project_block_switches,
};
use crate::ast::block_metadata::project_block_lines;
use crate::ast::block_model::SemanticFixedWidth;
use crate::ast::model::{Block, BlockKind, Keyword, ParsedAnnotation};

impl GraphProjector<'_> {
    pub(super) fn block(
        &mut self,
        id: usize,
        affiliated_keywords: &[Keyword<ParsedAnnotation>],
    ) -> Block<ParsedAnnotation> {
        let rows = self
            .block_lines
            .remove(&id)
            .expect("native block document plan");
        let record = self.record(id);
        let kind = match record.kind {
            "src-block" => BlockKind::Source,
            "example-block" => BlockKind::Example,
            "export-block" => BlockKind::Export,
            "quote-block" => BlockKind::Quote,
            "verse-block" => BlockKind::Verse,
            "center-block" => BlockKind::Center,
            "comment-block" => BlockKind::Comment,
            "dynamic-block" => BlockKind::Dynamic,
            _ => BlockKind::Special(record.field("name").unwrap_or_default().to_owned()),
        };
        let children = record.child_ids.clone();
        let name = if kind == BlockKind::Dynamic {
            record.field("name").map(str::to_owned)
        } else {
            affiliated_keywords
                .iter()
                .rev()
                .find(|keyword| keyword.key.eq_ignore_ascii_case("NAME"))
                .map(|keyword| keyword.value.trim().to_owned())
                .or_else(|| record.field("name").map(str::to_owned))
        };
        let language = record.field("language").map(str::to_owned);
        let parameters = project_block_parameters(record);
        let header_args = project_block_header_args(record);
        let value = record.field("body").unwrap_or_default().to_owned();
        let body_range = record
            .field_range("raw-body")
            .or_else(|| record.field_range("body"));
        let body_start = body_range.map_or(usize::from(record.range.start()), |range| {
            usize::from(range.start())
        });
        let source = body_range.map(|range| self.raw(range));
        let (raw_switches, switches) = project_block_switches(record, self.source);
        let lines = project_block_lines(rows, source.unwrap_or(&value), |start, end| {
            let start = body_start + start;
            let end = body_start + end;
            self.annotation(TextRange::new((start as u32).into(), (end as u32).into()))
        });
        let code_refs = lines
            .iter()
            .filter_map(|line| line.code_ref.clone())
            .collect();
        Block {
            kind,
            name,
            language,
            switches: raw_switches,
            line_numbering: switches.line_numbering.clone(),
            preserve_indentation: switches.preserve_indentation,
            switch_options: switches,
            lines,
            code_refs,
            parameters,
            header_args,
            value,
            children: children
                .into_iter()
                .filter_map(|child| self.element(child))
                .collect(),
        }
    }

    pub(super) fn fixed_width(&mut self, id: usize) -> SemanticFixedWidth<ParsedAnnotation> {
        let rows = self
            .block_lines
            .remove(&id)
            .expect("native fixed width document plan");
        let record = self.record(id);
        let range = record.range;
        let source = self.raw(range);
        let value = record.values("value").collect::<String>();
        let lines = project_block_lines(rows, source, |start, end| {
            let start = usize::from(range.start()) + start;
            let end = usize::from(range.start()) + end;
            self.annotation(TextRange::new((start as u32).into(), (end as u32).into()))
        });
        SemanticFixedWidth { value, lines }
    }
}
