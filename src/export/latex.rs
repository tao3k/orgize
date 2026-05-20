//! LaTeX exporter for the lossless syntax traversal API.

use rowan::{NodeOrToken, ast::AstNode};
use std::cmp::min;
use std::fmt;
use std::fmt::Write as _;

use crate::syntax_ast::{OrgTable, OrgTableCell, OrgTableRow};
use crate::{SyntaxElement, SyntaxKind, SyntaxNode};

use super::TraversalContext;
use super::Traverser;
use super::event::{Container, Event};

/// A wrapper for escaping text in LaTeX output.
///
/// ```rust
/// use orgize::export::LatexEscape as Escape;
///
/// assert_eq!(format!("{}", Escape("a_b & 10%")), r"a\_b \& 10\%");
/// assert_eq!(
///     format!("{}", Escape(r"\path{a}")),
///     r"\textbackslash{}path\{a\}"
/// );
/// ```
pub struct LatexEscape<S: AsRef<str>>(pub S);

impl<S: AsRef<str>> fmt::Display for LatexEscape<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ch in self.0.as_ref().chars() {
            match ch {
                '\\' => f.write_str(r"\textbackslash{}")?,
                '{' => f.write_str(r"\{")?,
                '}' => f.write_str(r"\}")?,
                '$' => f.write_str(r"\$")?,
                '&' => f.write_str(r"\&")?,
                '#' => f.write_str(r"\#")?,
                '%' => f.write_str(r"\%")?,
                '_' => f.write_str(r"\_")?,
                '^' => f.write_str(r"\textasciicircum{}")?,
                '~' => f.write_str(r"\textasciitilde{}")?,
                _ => f.write_char(ch)?,
            }
        }
        Ok(())
    }
}

/// Traverser that renders Org syntax events to LaTeX.
#[derive(Default)]
pub struct LatexExport {
    output: String,
    list_stack: Vec<ListKind>,
    table_stack: Vec<TableState>,
    options: LatexExportOptions,
}

/// Options for lossless syntax-tree LaTeX export.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LatexExportOptions {
    /// Convert Org special strings such as `--`, `---`, and `...` while rendering text events.
    pub special_strings: bool,
    /// Expand entity events to backend LaTeX when true, or preserve the source-backed raw entity.
    pub expand_entities: bool,
}

impl Default for LatexExportOptions {
    fn default() -> Self {
        Self {
            special_strings: false,
            expand_entities: true,
        }
    }
}

#[derive(Clone, Copy)]
enum ListKind {
    Itemize,
    Enumerate,
    Description,
}

struct TableState {
    cell_index: usize,
}

impl LatexExport {
    pub fn with_options(options: LatexExportOptions) -> Self {
        Self {
            options,
            ..Self::default()
        }
    }

    pub fn push_str(&mut self, s: impl AsRef<str>) {
        self.output += s.as_ref();
    }

    pub fn finish(self) -> String {
        self.output
    }

    /// Render syntax node to LaTeX string.
    ///
    /// ```rust
    /// use orgize::{Org, export::LatexExport, rowan::ast::AstNode, syntax_ast::Bold};
    ///
    /// let org = Org::parse("* /hello/ *world*");
    /// let bold = org.first_node::<Bold>().unwrap();
    /// let mut latex = LatexExport::default();
    /// latex.render(bold.syntax());
    /// assert_eq!(latex.finish(), r"\textbf{world}");
    /// ```
    pub fn render(&mut self, node: &SyntaxNode) {
        let mut ctx = TraversalContext::default();
        self.element(SyntaxElement::Node(node.clone()), &mut ctx);
    }

    fn follows_newline(&mut self) {
        if !self.output.is_empty() && !self.output.ends_with(['\n', '\r']) {
            self.output += "\n";
        }
    }

    fn follows_blank_line(&mut self) {
        if self.output.is_empty() || self.output.ends_with("\n\n") {
            return;
        }
        if self.output.ends_with(['\n', '\r']) {
            self.output += "\n";
        } else {
            self.output += "\n\n";
        }
    }

    fn push_begin_environment(&mut self, name: &str) {
        self.follows_newline();
        let _ = writeln!(&mut self.output, "\\begin{{{name}}}");
    }

    fn push_end_environment(&mut self, name: &str) {
        self.follows_newline();
        let _ = writeln!(&mut self.output, "\\end{{{name}}}");
    }

    fn push_verbatim_environment(&mut self, name: &str, content: &str) {
        self.push_begin_environment(name);
        self.output += content;
        if !content.ends_with('\n') {
            self.output += "\n";
        }
        self.push_end_environment(name);
    }
}

impl Traverser for LatexExport {
    fn event(&mut self, event: Event, ctx: &mut TraversalContext) {
        match event {
            Event::Enter(Container::Document(_)) | Event::Leave(Container::Document(_)) => {}

            Event::Enter(Container::Headline(headline)) => {
                self.follows_blank_line();
                self.output += headline_command(headline.level());
                self.output += "{";
                for elem in headline.title() {
                    self.element(elem, ctx);
                }
                self.output += "}\n";
            }
            Event::Leave(Container::Headline(_)) => {}

            Event::Enter(Container::Paragraph(_)) => {}
            Event::Leave(Container::Paragraph(_)) => self.follows_blank_line(),

            Event::Enter(Container::Section(_)) | Event::Leave(Container::Section(_)) => {}

            Event::Enter(Container::Italic(_)) => self.output += r"\emph{",
            Event::Leave(Container::Italic(_)) => self.output += "}",

            Event::Enter(Container::Bold(_)) => self.output += r"\textbf{",
            Event::Leave(Container::Bold(_)) => self.output += "}",

            Event::Enter(Container::Strike(_)) => self.output += r"\sout{",
            Event::Leave(Container::Strike(_)) => self.output += "}",

            Event::Enter(Container::Underline(_)) => self.output += r"\underline{",
            Event::Leave(Container::Underline(_)) => self.output += "}",

            Event::Enter(Container::Verbatim(_)) | Event::Enter(Container::Code(_)) => {
                self.output += r"\texttt{"
            }
            Event::Leave(Container::Verbatim(_)) | Event::Leave(Container::Code(_)) => {
                self.output += "}"
            }

            Event::Enter(Container::Superscript(_)) => self.output += r"\textsuperscript{",
            Event::Leave(Container::Superscript(_)) => self.output += "}",

            Event::Enter(Container::Subscript(_)) => self.output += r"\textsubscript{",
            Event::Leave(Container::Subscript(_)) => self.output += "}",

            Event::Enter(Container::SourceBlock(block)) => {
                self.push_verbatim_environment("verbatim", &block.value());
                ctx.skip();
            }
            Event::Leave(Container::SourceBlock(_)) => {}

            Event::Enter(Container::ExampleBlock(block)) => {
                self.push_verbatim_environment("verbatim", &block_content(block.syntax()));
                ctx.skip();
            }
            Event::Leave(Container::ExampleBlock(_)) => {}

            Event::Enter(Container::FixedWidth(fixed)) => {
                self.push_verbatim_environment("verbatim", &fixed.value());
                ctx.skip();
            }
            Event::Leave(Container::FixedWidth(_)) => {}

            Event::Enter(Container::QuoteBlock(_)) => self.push_begin_environment("quote"),
            Event::Leave(Container::QuoteBlock(_)) => self.push_end_environment("quote"),

            Event::Enter(Container::VerseBlock(_)) => self.push_begin_environment("verse"),
            Event::Leave(Container::VerseBlock(_)) => self.push_end_environment("verse"),

            Event::Enter(Container::CenterBlock(_)) => self.push_begin_environment("center"),
            Event::Leave(Container::CenterBlock(_)) => self.push_end_environment("center"),

            Event::Enter(Container::CommentBlock(_))
            | Event::Enter(Container::Comment(_))
            | Event::Enter(Container::PropertyDrawer(_))
            | Event::Enter(Container::Keyword(_)) => ctx.skip(),
            Event::Leave(Container::CommentBlock(_))
            | Event::Leave(Container::Comment(_))
            | Event::Leave(Container::PropertyDrawer(_))
            | Event::Leave(Container::Keyword(_)) => {}

            Event::Enter(Container::ExportBlock(block)) => {
                if block
                    .ty()
                    .is_some_and(|ty| ty.eq_ignore_ascii_case("latex"))
                {
                    self.follows_newline();
                    self.output += &block.value();
                    if !self.output.ends_with('\n') {
                        self.output += "\n";
                    }
                }
                ctx.skip();
            }
            Event::Leave(Container::ExportBlock(_)) => {}

            Event::Enter(Container::SpecialBlock(_)) | Event::Leave(Container::SpecialBlock(_)) => {
            }

            Event::Enter(Container::List(list)) => {
                let kind = if list.is_ordered() {
                    ListKind::Enumerate
                } else if list.is_descriptive() {
                    ListKind::Description
                } else {
                    ListKind::Itemize
                };
                self.list_stack.push(kind);
                self.push_begin_environment(list_environment(kind));
            }
            Event::Leave(Container::List(_)) => {
                if let Some(kind) = self.list_stack.pop() {
                    self.push_end_environment(list_environment(kind));
                }
            }
            Event::Enter(Container::ListItem(list_item)) => {
                self.follows_newline();
                if matches!(self.list_stack.last(), Some(ListKind::Description)) {
                    self.output += r"\item[";
                    for elem in list_item.tag() {
                        self.element(elem, ctx);
                    }
                    self.output += "] ";
                } else {
                    self.output += r"\item ";
                }
            }
            Event::Leave(Container::ListItem(_)) => self.follows_newline(),

            Event::Enter(Container::OrgTable(table)) => {
                self.follows_newline();
                let column_count = table_column_count(&table);
                let _ = writeln!(
                    &mut self.output,
                    "\\begin{{tabular}}{{{}}}",
                    "l".repeat(column_count)
                );
                self.table_stack.push(TableState { cell_index: 0 });
            }
            Event::Leave(Container::OrgTable(_)) => {
                self.output += "\\end{tabular}\n";
                self.table_stack.pop();
            }
            Event::Enter(Container::OrgTableRow(row)) => {
                if row.is_rule() {
                    self.output += "\\hline\n";
                    ctx.skip();
                } else if let Some(table) = self.table_stack.last_mut() {
                    table.cell_index = 0;
                }
            }
            Event::Leave(Container::OrgTableRow(row)) => {
                if !row.is_rule() {
                    self.output += r" \\";
                    self.output += "\n";
                }
            }
            Event::Enter(Container::OrgTableCell(_)) => {
                if let Some(table) = self.table_stack.last_mut() {
                    if table.cell_index > 0 {
                        self.output += " & ";
                    }
                    table.cell_index += 1;
                }
            }
            Event::Leave(Container::OrgTableCell(_)) => {}
            Event::Enter(Container::TableEl(_)) | Event::Leave(Container::TableEl(_)) => {}

            Event::Enter(Container::Link(link)) => {
                let path = link.path();
                let path = path.trim_start_matches("file:");

                if link.is_image() {
                    let _ = write!(
                        &mut self.output,
                        "\\includegraphics{{{}}}",
                        LatexEscape(path)
                    );
                    return ctx.skip();
                }

                if !link.has_description() {
                    let _ = write!(&mut self.output, "\\url{{{}}}", LatexEscape(path));
                    return ctx.skip();
                }

                let _ = write!(&mut self.output, "\\href{{{}}}{{", LatexEscape(path));
            }
            Event::Leave(Container::Link(_)) => self.output += "}",

            Event::Enter(Container::Target(target)) => {
                if let Some(label) = target_label(&target.raw(), "<<", ">>") {
                    let _ = write!(&mut self.output, "\\label{{{}}}", label);
                }
                ctx.skip();
            }
            Event::Leave(Container::Target(_)) => {}

            Event::Enter(Container::RadioTarget(target)) => {
                if let Some(label) = target_label(&target.raw(), "<<<", ">>>") {
                    let _ = write!(&mut self.output, "\\label{{{}}}", label);
                }
            }
            Event::Leave(Container::RadioTarget(_)) => {}

            Event::Enter(Container::FnRef(reference)) => {
                let _ = write!(
                    &mut self.output,
                    "\\textsuperscript{{{}}}",
                    LatexEscape(reference.raw())
                );
                ctx.skip();
            }
            Event::Leave(Container::FnRef(_)) => {}

            Event::Enter(Container::FnDef(definition)) => {
                let _ = write!(
                    &mut self.output,
                    "\\footnotetext{{{}}}",
                    LatexEscape(definition.raw())
                );
                ctx.skip();
            }
            Event::Leave(Container::FnDef(_)) => {}

            Event::Enter(Container::Drawer(_)) | Event::Leave(Container::Drawer(_)) => {}
            Event::Enter(Container::DynBlock(_)) | Event::Leave(Container::DynBlock(_)) => {}
            Event::Enter(Container::BabelCall(_)) | Event::Leave(Container::BabelCall(_)) => {}
            Event::Enter(Container::AffiliatedKeyword(_))
            | Event::Leave(Container::AffiliatedKeyword(_)) => {}

            Event::Text(text) => {
                let text = if self.options.special_strings {
                    special_strings(&text)
                } else {
                    text.to_string()
                };
                let _ = write!(&mut self.output, "{}", LatexEscape(text));
            }

            Event::Macros(macros) => {
                let _ = write!(&mut self.output, "{}", LatexEscape(macros.raw()));
            }

            Event::Cookie(cookie) => {
                let _ = write!(&mut self.output, "{}", LatexEscape(cookie.raw()));
            }

            Event::Citation(citation) => {
                if let Some(latex) = citation_command(&citation.raw()) {
                    self.output += &latex;
                } else {
                    let _ = write!(&mut self.output, "{}", LatexEscape(citation.raw()));
                }
            }

            Event::InlineCall(call) => {
                let _ = write!(&mut self.output, "{}", LatexEscape(call.raw()));
            }

            Event::InlineSrc(src) => {
                let _ = write!(&mut self.output, "\\texttt{{{}}}", LatexEscape(src.value()));
            }

            Event::Clock(clock) => {
                let _ = write!(&mut self.output, "{}", LatexEscape(clock.raw()));
            }

            Event::LineBreak(_) => self.output += "\\\\\n",

            Event::Snippet(snippet) if snippet.backend().eq_ignore_ascii_case("latex") => {
                self.output += &snippet.value();
            }
            Event::Snippet(_) => {}

            Event::Rule(_) => {
                self.follows_newline();
                self.output += "\\noindent\\rule{\\linewidth}{0.4pt}\n";
            }

            Event::Timestamp(timestamp) => {
                let _ = write!(
                    &mut self.output,
                    "\\textit{{{}}}",
                    LatexEscape(timestamp.raw())
                );
            }

            Event::LatexFragment(latex) => self.output += &latex.raw(),
            Event::LatexEnvironment(latex) => {
                self.follows_newline();
                self.output += &latex.raw();
                if !self.output.ends_with('\n') {
                    self.output += "\n";
                }
            }

            Event::Entity(entity) => {
                if !self.options.expand_entities {
                    let _ = write!(&mut self.output, "{}", LatexEscape(entity.raw()));
                } else if entity.is_latex_math() {
                    let _ = write!(&mut self.output, "${}$", entity.latex());
                } else {
                    self.output += entity.latex();
                }
            }

            #[cfg(feature = "syntax-org-fc")]
            Event::Cloze(cloze) => {
                let _ = write!(&mut self.output, "{}", LatexEscape(cloze.raw()));
            }
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

fn headline_command(level: usize) -> &'static str {
    match min(level, 6) {
        1 => r"\section",
        2 => r"\subsection",
        3 => r"\subsubsection",
        4 => r"\paragraph",
        5 => r"\subparagraph",
        _ => r"\textbf",
    }
}

fn list_environment(kind: ListKind) -> &'static str {
    match kind {
        ListKind::Itemize => "itemize",
        ListKind::Enumerate => "enumerate",
        ListKind::Description => "description",
    }
}

fn block_content(node: &SyntaxNode) -> String {
    node.children()
        .find(|n| n.kind() == SyntaxKind::BLOCK_CONTENT)
        .into_iter()
        .flat_map(|n| n.children_with_tokens())
        .filter_map(|elem| match elem {
            NodeOrToken::Token(token) if token.kind() == SyntaxKind::TEXT => {
                Some(token.text().to_owned())
            }
            _ => None,
        })
        .collect()
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

fn target_label(raw: &str, prefix: &str, suffix: &str) -> Option<String> {
    let label = raw.strip_prefix(prefix)?.strip_suffix(suffix)?.trim();
    if label.is_empty() {
        return None;
    }
    Some(sanitize_label(label))
}

fn sanitize_label(label: &str) -> String {
    let mut sanitized = String::with_capacity(label.len());
    let mut previous_was_separator = false;

    for ch in label.chars() {
        let next = if ch.is_ascii_alphanumeric() || matches!(ch, ':' | '_' | '-' | '.') {
            previous_was_separator = false;
            Some(ch)
        } else if ch.is_whitespace() {
            if previous_was_separator {
                None
            } else {
                previous_was_separator = true;
                Some('-')
            }
        } else if previous_was_separator {
            None
        } else {
            previous_was_separator = true;
            Some('-')
        };

        if let Some(ch) = next {
            sanitized.push(ch);
        }
    }

    sanitized.trim_matches('-').to_owned()
}

fn citation_command(raw: &str) -> Option<String> {
    let mut keys = Vec::new();
    let mut chars = raw.char_indices().peekable();

    while let Some((_, ch)) = chars.next() {
        if ch != '@' {
            continue;
        }

        let start = chars.peek().map(|(idx, _)| *idx)?;
        let mut end = start;
        while let Some((idx, ch)) = chars.peek().copied() {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':' | '.' | '/' | '#') {
                end = idx + ch.len_utf8();
                chars.next();
            } else {
                break;
            }
        }

        if end > start {
            keys.push(&raw[start..end]);
        }
    }

    if keys.is_empty() {
        None
    } else {
        Some(format!(r"\cite{{{}}}", keys.join(",")))
    }
}
