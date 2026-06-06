//! Markdown exporter for the lossless syntax traversal API.

use rowan::ast::AstNode;
use std::cmp::min;
use std::fmt::Write as _;

use crate::syntax_ast::{OrgTable, OrgTableCell, OrgTableRow, PropertyDrawer};
use crate::{SyntaxElement, SyntaxNode};

use super::TraversalContext;
use super::Traverser;
use super::event::{Container, Event};

/// Traverser that renders Org syntax events to Markdown.
#[derive(Default)]
pub struct MarkdownExport {
    output: String,

    inside_blockquote: bool,
    table_stack: Vec<TableState>,
    options: MarkdownExportOptions,
}

/// Options for lossless syntax-tree Markdown export.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkdownExportOptions {
    /// Convert Org special strings such as `--`, `---`, and `...` while rendering text events.
    pub special_strings: bool,
    /// Expand entity events to UTF-8 characters when true, or preserve the source-backed raw entity.
    pub expand_entities: bool,
}

impl Default for MarkdownExportOptions {
    fn default() -> Self {
        Self {
            special_strings: false,
            expand_entities: true,
        }
    }
}

#[derive(Default)]
struct TableState {
    column_count: usize,
    has_header: bool,
    standard_rows_seen: usize,
}

impl MarkdownExport {
    pub fn with_options(options: MarkdownExportOptions) -> Self {
        Self {
            options,
            ..Self::default()
        }
    }

    pub fn push_str(&mut self, s: impl AsRef<str>) {
        self.output += s.as_ref();
    }

    /// Render syntax node to markdown string
    ///
    /// ```rust
    /// use orgize::{Org, export::MarkdownExport, rowan::ast::AstNode, syntax_ast::Bold};
    ///
    /// let org = Org::parse("* /hello/ *world*");
    /// let bold = org.first_node::<Bold>().unwrap();
    /// let mut markdown = MarkdownExport::default();
    /// markdown.render(bold.syntax());
    /// assert_eq!(markdown.finish(), "**world**");
    /// ```
    pub fn render(&mut self, node: &SyntaxNode) {
        let mut ctx = TraversalContext::default();
        self.element(SyntaxElement::Node(node.clone()), &mut ctx);
    }

    pub fn finish(self) -> String {
        self.output
    }

    fn follows_newline(&mut self) {
        if !self.output.is_empty() && !self.output.ends_with(['\n', '\r']) {
            self.output += "\n";
        }
    }
}

impl Traverser for MarkdownExport {
    fn event(&mut self, event: Event, ctx: &mut TraversalContext) {
        match event {
            Event::Enter(Container::Document(_)) => {}
            Event::Leave(Container::Document(_)) => {}

            Event::Enter(Container::Headline(headline)) => {
                self.follows_newline();
                let level = min(headline.level(), 6);
                let _ = write!(&mut self.output, "{} ", "#".repeat(level));
                for elem in headline.title() {
                    self.element(elem, ctx);
                }
            }
            Event::Leave(Container::Headline(_)) => {}

            Event::Enter(Container::Paragraph(_)) => {}
            Event::Leave(Container::Paragraph(_)) => self.output += "\n",

            Event::Enter(Container::Section(_)) => self.follows_newline(),
            Event::Leave(Container::Section(_)) => {}

            Event::Enter(Container::Italic(_)) => self.output += "*",
            Event::Leave(Container::Italic(_)) => self.output += "*",

            Event::Enter(Container::Bold(_)) => self.output += "**",
            Event::Leave(Container::Bold(_)) => self.output += "**",

            Event::Enter(Container::Strike(_)) => self.output += "~~",
            Event::Leave(Container::Strike(_)) => self.output += "~~",

            Event::Enter(Container::Underline(_)) => {}
            Event::Leave(Container::Underline(_)) => {}

            Event::Enter(Container::Verbatim(_))
            | Event::Leave(Container::Verbatim(_))
            | Event::Enter(Container::Code(_))
            | Event::Leave(Container::Code(_)) => self.output += "`",

            Event::Enter(Container::SourceBlock(block)) => {
                self.follows_newline();
                self.output += "```";
                if let Some(language) = block.language() {
                    self.output += &language;
                }
                self.output += "\n";
            }
            Event::Leave(Container::SourceBlock(_)) => {
                self.follows_newline();
                self.output += "```\n";
            }

            Event::Enter(Container::ExampleBlock(_)) | Event::Enter(Container::FixedWidth(_)) => {
                self.follows_newline();
                self.output += "```\n";
            }
            Event::Leave(Container::ExampleBlock(_)) | Event::Leave(Container::FixedWidth(_)) => {
                self.follows_newline();
                self.output += "```\n";
            }

            Event::Enter(Container::ExportBlock(block)) => {
                if block.ty().is_some_and(|ty| {
                    ty.eq_ignore_ascii_case("markdown") || ty.eq_ignore_ascii_case("md")
                }) {
                    self.follows_newline();
                    self.output += &block.value();
                    if !self.output.ends_with('\n') {
                        self.output += "\n";
                    }
                }
                ctx.skip();
            }
            Event::Leave(Container::ExportBlock(_)) => {}

            Event::Enter(Container::QuoteBlock(_)) => {
                self.inside_blockquote = true;
                self.follows_newline();
                self.output += "> ";
            }
            Event::Leave(Container::QuoteBlock(_)) => self.inside_blockquote = false,

            Event::Enter(Container::CommentBlock(_)) => self.output += "<!--",
            Event::Leave(Container::CommentBlock(_)) => self.output += "-->",

            Event::Enter(Container::Comment(_)) => self.output += "<!--",
            Event::Leave(Container::Comment(_)) => self.output += "-->",

            Event::Enter(Container::Subscript(_)) => self.output += "<sub>",
            Event::Leave(Container::Subscript(_)) => self.output += "</sub>",

            Event::Enter(Container::Superscript(_)) => self.output += "<sup>",
            Event::Leave(Container::Superscript(_)) => self.output += "</sup>",

            Event::Enter(Container::List(_list)) => {}
            Event::Leave(Container::List(_list)) => {}

            Event::Enter(Container::ListItem(list_item)) => {
                self.follows_newline();
                self.output += &" ".repeat(list_item.indent());
                self.output += &list_item.bullet();
            }
            Event::Leave(Container::ListItem(_)) => {}

            Event::Enter(Container::OrgTable(table)) => {
                self.follows_newline();
                self.table_stack.push(TableState {
                    column_count: table_column_count(&table),
                    has_header: table.has_header(),
                    standard_rows_seen: 0,
                });
            }
            Event::Leave(Container::OrgTable(_)) => {
                self.table_stack.pop();
                self.output += "\n";
            }
            Event::Enter(Container::OrgTableRow(row)) => {
                if row.is_rule() {
                    let column_count = self
                        .table_stack
                        .last()
                        .map(|table| table.column_count)
                        .unwrap_or(1);
                    self.output += "|";
                    for _ in 0..column_count {
                        self.push_table_separator_cell();
                    }
                    self.output += "\n";
                    return ctx.skip();
                }

                self.output += "|";
            }
            Event::Leave(Container::OrgTableRow(row)) if !row.is_rule() => {
                self.output += "\n";
                let delimiter_columns = self.table_stack.last_mut().and_then(|table| {
                    table.standard_rows_seen += 1;
                    (!table.has_header && table.standard_rows_seen == 1)
                        .then_some(table.column_count)
                });
                if let Some(column_count) = delimiter_columns {
                    self.output += "|";
                    for _ in 0..column_count {
                        self.push_table_separator_cell();
                    }
                    self.output += "\n";
                }
            }
            Event::Leave(Container::OrgTableRow(_)) => {}
            Event::Enter(Container::OrgTableCell(_)) => {
                self.output += " ";
            }
            Event::Leave(Container::OrgTableCell(_)) => self.output += " |",

            Event::Enter(Container::PropertyDrawer(drawer)) => {
                self.push_property_drawer_table(&drawer);
                ctx.skip();
            }
            Event::Leave(Container::PropertyDrawer(_)) => {}

            Event::Enter(Container::Link(link)) => {
                let path = link.path();
                let path = path.trim_start_matches("file:");

                if link.is_image() {
                    let _ = write!(&mut self.output, "![]({path})");
                    return ctx.skip();
                }

                if !link.has_description() {
                    let _ = write!(&mut self.output, r#"[{}]({})"#, &path, &path);
                    return ctx.skip();
                }

                self.output += "[";
            }
            Event::Leave(Container::Link(link)) => {
                let _ = write!(&mut self.output, r#"]({})"#, &*link.path());
            }

            Event::Text(text) => {
                let text = if self.options.special_strings {
                    special_strings(&text)
                } else {
                    text.to_string()
                };
                if self.inside_blockquote {
                    self.push_blockquote_text(&text);
                } else {
                    self.output += &text;
                }
            }

            Event::LineBreak(_) => self.output += "\\\n",

            Event::Snippet(snippet)
                if snippet.backend().eq_ignore_ascii_case("markdown")
                    || snippet.backend().eq_ignore_ascii_case("md") =>
            {
                self.output += &snippet.value();
            }
            Event::Snippet(_snippet) => {}

            Event::Citation(citation) => self.output += &citation.raw(),

            Event::Rule(_) => self.output += "\n-----\n",

            Event::Timestamp(timestamp) => self.output += &timestamp.raw(),

            Event::LatexFragment(latex) => {
                let _ = write!(&mut self.output, "{}", &latex.syntax);
            }
            Event::LatexEnvironment(latex) => {
                let _ = write!(&mut self.output, "{}", &latex.syntax);
            }

            Event::Entity(entity) => {
                if self.options.expand_entities {
                    self.output += entity.utf8();
                } else {
                    self.output += &entity.raw();
                }
            }

            _ => {}
        }
    }
}

fn special_strings(value: &str) -> String {
    value
        .replace("---", "\u{2014}")
        .replace("--", "\u{2013}")
        .replace("...", "\u{2026}")
        .replace("\\-", "\u{00AD}")
        .replace('\'', "\u{2019}")
}

impl MarkdownExport {
    fn push_blockquote_text(&mut self, text: &str) {
        let mut lines = text.split('\n').peekable();
        let mut first = true;

        while let Some(line) = lines.next() {
            if line.is_empty() && lines.peek().is_none() {
                break;
            }

            if !first {
                self.output += "\n> ";
            }
            self.output += line;
            first = false;
        }

        if text.ends_with('\n') {
            self.output += "\n";
        }
    }

    fn push_table_separator_cell(&mut self) {
        self.output += " --- |";
    }

    fn push_property_drawer_table(&mut self, drawer: &PropertyDrawer) {
        let mut properties = drawer.iter().peekable();
        if properties.peek().is_none() {
            return;
        }

        self.follows_newline();
        self.output += "| Key | Value |\n| --- | --- |\n";

        for (key, value) in properties {
            self.output += "| ";
            self.push_table_cell_text(key.0.text());
            self.output += " | ";
            self.push_table_cell_text(value.0.text());
            self.output += " |\n";
        }

        self.output += "\n";
    }

    fn push_table_cell_text(&mut self, text: &str) {
        for ch in text.chars() {
            match ch {
                '|' => self.output += "\\|",
                '\\' => self.output += "\\\\",
                '\n' | '\r' => self.output += " ",
                _ => self.output.push(ch),
            }
        }
    }
}

fn table_column_count(table: &OrgTable) -> usize {
    table
        .syntax()
        .children()
        .filter_map(OrgTableRow::cast)
        .filter(|row| !row.is_rule())
        .map(|row| {
            row.syntax()
                .children()
                .filter_map(OrgTableCell::cast)
                .count()
        })
        .max()
        .unwrap_or(1)
        .max(1)
}
