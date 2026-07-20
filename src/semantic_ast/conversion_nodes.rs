//! Semantic node conversion for drawers, lists, tables, blocks, and paragraphs.

use super::block_metadata::{
    BlockLineOptions, block_code_refs, parse_block_header_args, parse_block_lines,
    parse_block_switches,
};
use super::block_syntax::{
    block_name_from_begin, block_parts, block_switches_from_begin, semantic_block_name,
};
use super::conversion::Converter;
use super::conversion_util::text_range;
use super::footnote_parts::FootnoteDefParts;
use super::table_metadata::{parsed_table_formulas, table_column_alignments};
use super::{
    Block, BlockKind, BlockSwitches, Checkbox, Drawer, FootnoteDef, List, ListItem, ListType,
    ParsedAnnotation, SemanticFixedWidth, Table, TableCell, TableRow,
};
use crate::{
    syntax::{SyntaxKind, SyntaxNode},
    syntax_ast,
};
use rowan::ast::AstNode;

use super::conversion_util::{block_content_line_ranges, source_line_ranges};

impl<'a> Converter<'a> {
    pub(super) fn drawer(&mut self, node: &SyntaxNode) -> Drawer<ParsedAnnotation> {
        let name = syntax_ast::SyntaxDrawer::cast(node.clone())
            .map(|drawer| drawer.name().to_string())
            .unwrap_or_default();
        let children = node
            .children()
            .find(|child| child.kind() == SyntaxKind::DRAWER_CONTENT)
            .map(|child| self.elements_from_container(&child))
            .unwrap_or_default();

        Drawer {
            name,
            children,
            raw: node.to_string(),
        }
    }

    pub(super) fn list(&mut self, node: &SyntaxNode) -> List<ParsedAnnotation> {
        let syntax = syntax_ast::SyntaxList::cast(node.clone()).expect("list node");
        let has_descriptive_item = node.children().any(|item| {
            item.kind() == SyntaxKind::LIST_ITEM
                && item
                    .children()
                    .any(|child| child.kind() == SyntaxKind::LIST_ITEM_TAG)
        });
        let list_type = if has_descriptive_item || syntax.is_descriptive() {
            ListType::Descriptive
        } else if syntax.is_ordered() {
            ListType::Ordered
        } else {
            ListType::Unordered
        };
        let items = node
            .children()
            .filter(|child| child.kind() == SyntaxKind::LIST_ITEM)
            .map(|child| self.list_item(&child))
            .collect();

        List { list_type, items }
    }

    pub(super) fn list_item(&mut self, node: &SyntaxNode) -> ListItem<ParsedAnnotation> {
        let syntax = syntax_ast::SyntaxListItem::cast(node.clone()).expect("list item node");
        let tag = syntax.tag().collect::<Vec<_>>();
        let children = node
            .children()
            .find(|child| child.kind() == SyntaxKind::LIST_ITEM_CONTENT)
            .map(|child| self.elements_from_container(&child))
            .unwrap_or_default();
        let checkbox = syntax.checkbox().and_then(|token| match token.as_ref() {
            "X" => Some(Checkbox::On),
            " " => Some(Checkbox::Off),
            "-" => Some(Checkbox::Trans),
            _ => None,
        });

        ListItem {
            ann: self.node_ann(node),
            bullet: syntax.bullet().to_string(),
            counter: syntax.counter().map(|x| x.to_string()),
            checkbox,
            tag: self.objects_from_elements(tag),
            children,
        }
    }

    pub(super) fn table(&mut self, node: &SyntaxNode) -> Table<ParsedAnnotation> {
        let row_nodes = node
            .children()
            .filter(|child| {
                matches!(
                    child.kind(),
                    SyntaxKind::ORG_TABLE_RULE_ROW | SyntaxKind::ORG_TABLE_STANDARD_ROW
                )
            })
            .collect::<Vec<_>>();
        let column_alignments = table_column_alignments(&row_nodes);
        let rows = row_nodes
            .iter()
            .map(|child| TableRow {
                ann: self.node_ann(child),
                is_rule: child.kind() == SyntaxKind::ORG_TABLE_RULE_ROW,
                cells: child
                    .children()
                    .filter(|cell| cell.kind() == SyntaxKind::ORG_TABLE_CELL)
                    .map(|cell| TableCell {
                        ann: self.node_ann(&cell),
                        objects: self.objects_from_elements(cell.children_with_tokens()),
                    })
                    .collect(),
            })
            .collect();
        let formulas = node
            .children()
            .filter(|child| child.kind() == SyntaxKind::KEYWORD)
            .map(|child| self.keyword(&child, false))
            .collect::<Vec<_>>();
        let parsed_formulas = parsed_table_formulas(&formulas);

        Table {
            rows,
            column_alignments,
            formulas,
            parsed_formulas,
        }
    }

    pub(super) fn table_el(&self, node: &SyntaxNode) -> String {
        syntax_ast::TableEl::cast(node.clone())
            .map(|table| table.raw())
            .unwrap_or_else(|| node.to_string())
    }

    pub(super) fn block(&mut self, node: &SyntaxNode) -> Block<ParsedAnnotation> {
        let parts = block_parts(node);
        let kind = match node.kind() {
            SyntaxKind::SOURCE_BLOCK => BlockKind::Source,
            SyntaxKind::EXAMPLE_BLOCK => BlockKind::Example,
            SyntaxKind::EXPORT_BLOCK => BlockKind::Export,
            SyntaxKind::QUOTE_BLOCK => BlockKind::Quote,
            SyntaxKind::VERSE_BLOCK => BlockKind::Verse,
            SyntaxKind::CENTER_BLOCK => BlockKind::Center,
            SyntaxKind::COMMENT_BLOCK => BlockKind::Comment,
            SyntaxKind::DYN_BLOCK => BlockKind::Dynamic,
            SyntaxKind::SPECIAL_BLOCK => BlockKind::Special(
                block_name_from_begin(parts.begin.as_ref()).unwrap_or_else(|| "special".into()),
            ),
            _ => BlockKind::Special(format!("{:?}", node.kind())),
        };

        let source = syntax_ast::SourceBlock::cast(node.clone());
        let export = syntax_ast::ExportBlock::cast(node.clone());
        let switches = block_switches_from_begin(parts.begin.as_ref());
        let switch_options = parse_block_switches(switches.as_deref());
        let preserve_indentation =
            self.config.src_preserve_indentation || switch_options.preserve_indentation;
        let value = parts
            .content
            .as_ref()
            .map(SyntaxNode::to_string)
            .unwrap_or_default();
        let children = parts
            .content
            .as_ref()
            .map(|content| self.elements_from_container(content))
            .unwrap_or_default();

        let value = source
            .as_ref()
            .map(|block| block.value())
            .or_else(|| export.as_ref().map(|block| block.value()))
            .unwrap_or(value);
        let lines = if matches!(kind, BlockKind::Source | BlockKind::Example) {
            let source = parts.content.as_ref().map(SyntaxNode::to_string);
            let line_ranges = parts
                .content
                .as_ref()
                .zip(source.as_deref())
                .map(|(content, source)| block_content_line_ranges(content, source))
                .unwrap_or_default();
            let fallback_range = parts
                .content
                .as_ref()
                .map(|content| {
                    let end = usize::from(content.text_range().end());
                    text_range(end, end)
                })
                .unwrap_or_else(|| node.text_range());

            parse_block_lines(
                &value,
                source.as_deref(),
                BlockLineOptions {
                    switches: &switch_options,
                    tab_width: self.config.src_tab_width,
                    preserve_indentation,
                },
                |index| self.ann(line_ranges.get(index).copied().unwrap_or(fallback_range)),
            )
        } else {
            Vec::new()
        };
        let code_refs = block_code_refs(&lines);
        let parameters = source
            .as_ref()
            .and_then(|block| block.parameters().map(|x| x.to_string()));

        Block {
            kind,
            name: semantic_block_name(node.kind(), parts.begin.as_ref()),
            language: source
                .as_ref()
                .and_then(|block| block.language().map(|x| x.to_string())),
            line_numbering: switch_options.line_numbering.clone(),
            switch_options,
            preserve_indentation,
            lines,
            code_refs,
            switches,
            header_args: parse_block_header_args(parameters.as_deref()),
            parameters,
            value,
            children,
        }
    }

    pub(super) fn fixed_width(
        &mut self,
        node: &SyntaxNode,
    ) -> SemanticFixedWidth<ParsedAnnotation> {
        let syntax = syntax_ast::FixedWidth::cast(node.clone());
        let value = syntax
            .as_ref()
            .map(syntax_ast::FixedWidth::value)
            .unwrap_or_else(|| node.to_string());
        let source = node.to_string();
        let line_ranges = source_line_ranges(usize::from(node.text_range().start()), &source);
        let switches = BlockSwitches::default();
        let fallback_range = {
            let end = usize::from(node.text_range().end());
            text_range(end, end)
        };
        let lines = parse_block_lines(
            &value,
            Some(&source),
            BlockLineOptions {
                switches: &switches,
                tab_width: self.config.src_tab_width,
                preserve_indentation: self.config.src_preserve_indentation,
            },
            |index| self.ann(line_ranges.get(index).copied().unwrap_or(fallback_range)),
        );

        SemanticFixedWidth { value, lines }
    }

    pub(super) fn footnote_def(&mut self, node: &SyntaxNode) -> FootnoteDef<ParsedAnnotation> {
        let parts = node
            .children_with_tokens()
            .fold(FootnoteDefParts::default(), FootnoteDefParts::push);
        let children = self
            .paragraph_from_elements(parts.content)
            .into_iter()
            .collect::<Vec<_>>();

        FootnoteDef {
            label: parts.label,
            children,
        }
    }
}
