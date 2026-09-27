//! Markdown projection from the Scheme-generated Org Element graph.
//!
//! Unmapped Element kinds fail explicitly; the old syntax parser is never used.

use gerbil_parser_rowan::GraphRecord;

use crate::entities::ENTITIES;
use crate::export::{MarkdownExportOptions, special_strings};
use crate::org_aot::{OrgAotDocument, org_image_link};

impl OrgAotDocument {
    /// Render supported AOT Elements as Markdown, reporting unmapped kinds.
    ///
    /// # Errors
    /// Returns the first Element kind not yet projected by this exporter.
    pub fn try_to_markdown(&self) -> Result<String, String> {
        self.try_to_markdown_with_options(MarkdownExportOptions::default())
    }

    /// Render the AOT graph using Markdown presentation options.
    ///
    /// # Errors
    /// Returns the first Element or Object kind not yet projected.
    pub fn try_to_markdown_with_options(
        &self,
        options: MarkdownExportOptions,
    ) -> Result<String, String> {
        let source = self.to_org();
        let mut renderer = MarkdownRenderer {
            document: self,
            source: &source,
            options,
            output: String::new(),
            suppress_next_blank: false,
        };
        renderer.render(0)?;
        Ok(renderer.output)
    }

    /// Render one graph record and its children as Markdown.
    ///
    /// # Errors
    /// Returns an error for an unknown record identity or unsupported kind.
    pub fn try_markdown_record(&self, id: usize) -> Result<String, String> {
        if id >= self.records().len() {
            return Err(format!("invalid AOT graph record identity: {id}"));
        }
        let source = self.to_org();
        let mut renderer = MarkdownRenderer {
            document: self,
            source: &source,
            options: MarkdownExportOptions::default(),
            output: String::new(),
            suppress_next_blank: false,
        };
        renderer.render(id)?;
        Ok(renderer.output)
    }

    /// Render supported AOT Elements as Markdown.
    ///
    /// Use [`Self::try_to_markdown`] to handle an unsupported Element kind.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        self.try_to_markdown()
            .expect("Scheme AOT Markdown projection does not cover this Element")
    }

    /// Render the AOT graph with Markdown presentation options.
    #[must_use]
    pub fn to_markdown_with_options(&self, options: MarkdownExportOptions) -> String {
        self.try_to_markdown_with_options(options)
            .expect("Scheme AOT Markdown projection does not cover this Element")
    }
}

struct MarkdownRenderer<'a> {
    document: &'a OrgAotDocument,
    source: &'a str,
    options: MarkdownExportOptions,
    output: String,
    suppress_next_blank: bool,
}

impl MarkdownRenderer<'_> {
    fn record(&self, id: usize) -> &GraphRecord {
        &self.document.records()[id]
    }

    fn source(&self, record: &GraphRecord) -> &str {
        self.source
            .get(usize::from(record.range.start())..usize::from(record.range.end()))
            .expect("AOT graph range is source-bound")
    }

    fn newline(&mut self) {
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }

    fn source_blank_line(&mut self, id: usize) {
        if self.suppress_next_blank {
            self.suppress_next_blank = false;
            return;
        }
        let start = usize::from(self.record(id).range.start());
        if self.source[..start].ends_with("\n\n") && !self.output.ends_with("\n\n") {
            self.newline();
            self.output.push('\n');
        }
    }

    fn text(&mut self, value: &str) {
        if self.options.special_strings {
            self.output.push_str(&special_strings(value));
        } else {
            self.output.push_str(value);
        }
    }

    fn bounds(&self, id: usize) -> (usize, usize) {
        let range = self.record(id).range;
        (usize::from(range.start()), usize::from(range.end()))
    }

    fn render(&mut self, id: usize) -> Result<(), String> {
        let kind = self.record(id).kind;
        let child_ids = self.record(id).child_ids.clone();
        match kind {
            "org-data" | "section" => {
                for child in child_ids {
                    self.render(child)?;
                }
            }
            "headline" => {
                let level = self.record(id).field("markers").map_or(1, str::len).min(6);
                let title = self
                    .record(id)
                    .field("title")
                    .unwrap_or_default()
                    .trim()
                    .to_owned();
                let (start, end) = self.bounds(id);
                let line_end = self.source[start..end]
                    .find('\n')
                    .map_or(end, |offset| start + offset);
                let title_start = self.source[start..line_end]
                    .find(&title)
                    .map_or(line_end, |offset| start + offset);
                self.newline();
                self.output.push_str(&"#".repeat(level));
                self.output.push(' ');
                self.render_inline_span(title_start, title_start + title.len(), &child_ids)?;
                self.output.push('\n');
                for child in child_ids {
                    if usize::from(self.record(child).range.start()) >= line_end {
                        self.render(child)?;
                    }
                }
            }
            "paragraph" => {
                self.source_blank_line(id);
                self.render_paragraph(id)?;
                self.newline();
            }
            "src-block" | "example-block" | "fixed-width" => {
                let language = self
                    .record(id)
                    .field("language")
                    .unwrap_or_default()
                    .to_owned();
                let body = self.record(id).field("body").unwrap_or_default().to_owned();
                self.newline();
                self.output.push_str("```");
                if kind == "src-block" {
                    self.output.push_str(&language);
                }
                self.output.push('\n');
                if kind == "example-block" {
                    self.output.push_str(&body.replace(",*", "*"));
                } else {
                    self.output.push_str(&body);
                }
                self.newline();
                self.output.push_str("```\n");
            }
            "export-block" => {
                let markdown_backend = self.record(id).field("backend").is_some_and(|backend| {
                    backend.eq_ignore_ascii_case("markdown") || backend.eq_ignore_ascii_case("md")
                });
                if markdown_backend {
                    let body = self.record(id).field("body").unwrap_or_default().to_owned();
                    self.newline();
                    self.output.push_str(&body);
                    self.newline();
                    self.suppress_next_blank = true;
                }
            }
            "quote-block" | "verse-block" | "center-block" => self.render_container(id)?,
            "plain-list" => {
                self.source_blank_line(id);
                for (index, child) in child_ids.into_iter().enumerate() {
                    if index > 0 && !self.output.ends_with("\n\n") {
                        self.newline();
                        self.output.push('\n');
                    }
                    self.render(child)?;
                }
            }
            "item" => self.render_item(id)?,
            "property-drawer" => self.render_properties(id),
            "keyword" | "node-property" | "planning" | "clock" | "babel-call" | "target"
            | "radio-target" => {}
            "horizontal-rule" => {
                self.newline();
                self.output.push_str("-----\n");
            }
            "table" => self.render_table(id),
            "comment" | "comment-block" => {}
            "bold" | "italic" | "strike-through" | "underline" | "code" | "verbatim"
            | "subscript" | "superscript" => self.render_markup(id),
            "link" => self.render_link(id)?,
            "entity" => self.render_entity(id),
            "line-break" => self.output.push_str("\\\n"),
            "export-snippet" => {
                if self.record(id).field("backend").is_some_and(|backend| {
                    backend.eq_ignore_ascii_case("markdown") || backend.eq_ignore_ascii_case("md")
                }) {
                    let value = self
                        .record(id)
                        .field("value")
                        .unwrap_or_default()
                        .to_owned();
                    self.output.push_str(&value);
                }
            }
            "citation" | "timestamp" | "latex-fragment" | "latex-environment"
            | "statistics-cookie" | "macro" | "inline-babel-call" | "footnote-reference" => {
                let value = self.source(self.record(id)).to_owned();
                self.text(&value);
            }
            "inline-src-block" => {
                let value = self
                    .record(id)
                    .field("value")
                    .unwrap_or_default()
                    .to_owned();
                self.output.push('`');
                self.text(&value);
                self.output.push('`');
            }
            _ => {
                return Err(format!(
                    "Scheme AOT Markdown projection does not support {kind}"
                ));
            }
        }
        Ok(())
    }

    fn render_paragraph(&mut self, id: usize) -> Result<(), String> {
        let (start, end) = self.bounds(id);
        let trailing = self.source[start..end]
            .bytes()
            .rev()
            .take_while(|byte| *byte == b'\n')
            .count();
        let end = end.saturating_sub(trailing.saturating_sub(1));
        self.render_inline_span(start, end, &self.record(id).child_ids.clone())
    }

    fn render_inline_span(
        &mut self,
        start: usize,
        end: usize,
        children: &[usize],
    ) -> Result<(), String> {
        let mut cursor = start;
        for &child in children {
            let (child_start, child_end) = self.bounds(child);
            if child_start < cursor || child_end > end {
                continue;
            }
            let gap = self.source[cursor..child_start].to_owned();
            self.text(&gap);
            self.render(child)?;
            cursor = child_end;
        }
        let tail = self.source[cursor..end].to_owned();
        self.text(&tail);
        Ok(())
    }

    fn render_markup(&mut self, id: usize) {
        let marker = match self.record(id).kind {
            "bold" => "**",
            "italic" => "*",
            "strike-through" => "~~",
            "code" | "verbatim" => "`",
            "subscript" => "<sub>",
            "superscript" => "<sup>",
            _ => "",
        };
        let close = match self.record(id).kind {
            "subscript" => "</sub>",
            "superscript" => "</sup>",
            _ => marker,
        };
        let value = self
            .record(id)
            .field("value")
            .unwrap_or_default()
            .to_owned();
        self.output.push_str(marker);
        self.text(&value);
        self.output.push_str(close);
    }

    fn render_link(&mut self, id: usize) -> Result<(), String> {
        let path = self.record(id).field("path").unwrap_or_default().to_owned();
        let href = path.trim_start_matches("file:");
        if org_image_link(href) {
            self.output.push_str("![](");
            self.output.push_str(href);
            self.output.push(')');
        } else if let Some(description) = self.record(id).field("description") {
            let description_len = description.len();
            let (_, end) = self.bounds(id);
            let description_end = end.saturating_sub(2);
            let description_start = description_end.saturating_sub(description_len);
            self.output.push('[');
            self.render_inline_span(
                description_start,
                description_end,
                &self.record(id).child_ids.clone(),
            )?;
            self.output.push_str("](");
            self.output.push_str(&path);
            self.output.push(')');
        } else {
            self.output.push('[');
            self.output.push_str(href);
            self.output.push_str("](");
            self.output.push_str(href);
            self.output.push(')');
        }
        Ok(())
    }

    fn render_entity(&mut self, id: usize) {
        let name = self.record(id).field("name").unwrap_or_default();
        if self.options.expand_entities
            && let Some(entity) = ENTITIES.iter().find(|entity| entity.0 == name)
        {
            self.output.push_str(entity.6);
        } else {
            let raw = self.source(self.record(id)).to_owned();
            self.text(&raw);
        }
    }

    fn render_container(&mut self, id: usize) -> Result<(), String> {
        self.source_blank_line(id);
        self.newline();
        let original = std::mem::take(&mut self.output);
        for child in self.record(id).child_ids.clone() {
            self.render(child)?;
        }
        let content = std::mem::replace(&mut self.output, original);
        if self.record(id).kind == "quote-block" {
            for line in content.trim_end_matches('\n').lines() {
                self.output.push_str("> ");
                self.output.push_str(line);
                self.output.push('\n');
            }
        } else {
            self.output.push_str(&content);
        }
        Ok(())
    }

    fn render_item(&mut self, id: usize) -> Result<(), String> {
        self.newline();
        let bullet = self.record(id).field("bullet").unwrap_or("+").to_owned();
        self.output.push_str(&bullet);
        self.output.push(' ');
        for child in self.record(id).child_ids.clone() {
            self.render(child)?;
        }
        Ok(())
    }

    fn render_properties(&mut self, id: usize) {
        let properties = self.record(id).child_ids.clone();
        if properties.is_empty() {
            return;
        }
        self.newline();
        self.output.push_str("| Key | Value |\n| --- | --- |\n");
        for child in properties {
            if self.record(child).kind != "node-property" {
                continue;
            }
            let key = self
                .record(child)
                .field("key")
                .unwrap_or_default()
                .to_owned();
            let value = self
                .record(child)
                .field("value")
                .unwrap_or_default()
                .to_owned();
            self.output.push_str("| ");
            push_table_cell_text(&mut self.output, &key);
            self.output.push_str(" | ");
            push_table_cell_text(&mut self.output, &value);
            self.output.push_str(" |\n");
        }
        self.output.push('\n');
    }

    fn render_table(&mut self, id: usize) {
        self.source_blank_line(id);
        self.newline();
        let rows = self.record(id).child_ids.clone();
        let has_rule = rows
            .iter()
            .any(|&row| self.record(row).kind == "table-rule-row");
        let mut ordinary_rows = 0;
        for row_id in rows {
            let row_kind = self.record(row_id).kind;
            if row_kind == "table-rule-row" {
                let columns = rows_column_count(self.document.records(), id).max(1);
                self.output.push('|');
                for _ in 0..columns {
                    self.output.push_str(" --- |");
                }
                self.output.push('\n');
                continue;
            }
            if row_kind != "table-row" {
                continue;
            }
            ordinary_rows += 1;
            let cells = self.record(row_id).child_ids.clone();
            self.output.push('|');
            for cell_id in cells {
                let text = self
                    .record(cell_id)
                    .field("text")
                    .unwrap_or_default()
                    .trim()
                    .to_owned();
                self.output.push(' ');
                self.output.push_str(&text);
                self.output.push_str(" |");
            }
            self.output.push('\n');
            if ordinary_rows == 1 && !has_rule {
                self.output.push('|');
                for _ in 0..rows_column_count(self.document.records(), id).max(1) {
                    self.output.push_str(" --- |");
                }
                self.output.push('\n');
            }
        }
    }
}

fn push_table_cell_text(output: &mut String, text: &str) {
    for character in text.chars() {
        match character {
            '|' => output.push_str("\\|"),
            '\\' => output.push_str("\\\\"),
            '\n' | '\r' => output.push(' '),
            _ => output.push(character),
        }
    }
}

fn rows_column_count(records: &[GraphRecord], table_id: usize) -> usize {
    records[table_id]
        .child_ids
        .iter()
        .map(|&id| records[id].child_ids.len())
        .max()
        .unwrap_or(0)
}
