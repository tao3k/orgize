//! Source-backed block projection from Scheme-owned graph fields.

use rowan::TextRange;

use super::GraphProjector;
use crate::ast::aot_block_switches::{
    project_block_header_args, project_block_parameters, project_block_switches,
};
use crate::ast::block_metadata::{BlockLineOptions, parse_block_lines, split_block_lines};
use crate::ast::block_model::{BlockSwitches, SemanticFixedWidth};
use crate::ast::model::{Block, BlockKind, Keyword, ParsedAnnotation};

impl GraphProjector<'_> {
    pub(super) fn block(
        &mut self,
        id: usize,
        affiliated_keywords: &[Keyword<ParsedAnnotation>],
    ) -> Block<ParsedAnnotation> {
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
        let parameters = project_block_parameters(record, self.source);
        let header_args = project_block_header_args(record, self.source);
        let value = record.field("body").unwrap_or_default().to_owned();
        let body_range = record
            .field_range("raw-body")
            .or_else(|| record.field_range("body"));
        let body_start = body_range.map_or(usize::from(record.range.start()), |range| {
            usize::from(range.start())
        });
        let source = body_range.map(|range| self.raw(range));
        let source_lines = split_block_lines(source.unwrap_or(&value));
        let (raw_switches, switches) = project_block_switches(record, self.source);
        let lines = parse_block_lines(
            &value,
            source,
            BlockLineOptions {
                switches: &switches,
                tab_width: self.document.config().src_tab_width,
                preserve_indentation: switches.preserve_indentation,
            },
            |index| {
                let line = &source_lines[index];
                let start = body_start + line.start;
                let end = body_start + line.end;
                self.annotation(TextRange::new((start as u32).into(), (end as u32).into()))
            },
        );
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

    pub(super) fn fixed_width(&self, range: TextRange) -> SemanticFixedWidth<ParsedAnnotation> {
        let source = self.raw(range);
        let source_lines = split_block_lines(source);
        let mut value = String::with_capacity(source.len());
        for line in &source_lines {
            let content = line
                .text
                .split_once(':')
                .map_or(line.text, |(_, text)| text);
            let content = content.strip_prefix(' ').unwrap_or(content);
            value.push_str(content);
            if let Some(ending) = line.ending {
                value.push_str(ending);
            }
        }
        let switches = BlockSwitches::default();
        let lines = parse_block_lines(
            &value,
            Some(source),
            BlockLineOptions {
                switches: &switches,
                tab_width: self.document.config().src_tab_width,
                preserve_indentation: false,
            },
            |index| {
                let line = &source_lines[index];
                let start = usize::from(range.start()) + line.start;
                let end = usize::from(range.start()) + line.end;
                self.annotation(TextRange::new((start as u32).into(), (end as u32).into()))
            },
        );
        SemanticFixedWidth { value, lines }
    }
}
