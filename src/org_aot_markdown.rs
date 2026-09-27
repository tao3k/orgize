//! Markdown projection from the Scheme-generated Org Element graph.
//!
//! Unmapped Element kinds fail explicitly; the old syntax parser is never used.

use gerbil_parser_rowan::GraphRecord;

use crate::org_aot::OrgAotDocument;

impl OrgAotDocument {
    /// Render supported AOT Elements as Markdown, reporting unmapped kinds.
    ///
    /// # Errors
    /// Returns the first Element kind not yet projected by this exporter.
    pub fn try_to_markdown(&self) -> Result<String, String> {
        let source = self.to_org();
        let mut renderer = MarkdownRenderer {
            document: self,
            source: &source,
            output: String::new(),
        };
        renderer.render(0)?;
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
}

struct MarkdownRenderer<'a> {
    document: &'a OrgAotDocument,
    source: &'a str,
    output: String,
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
                self.newline();
                self.output.push_str(&"#".repeat(level));
                self.output.push(' ');
                self.output.push_str(&title);
                self.output.push('\n');
                for child in child_ids {
                    self.render(child)?;
                }
            }
            "paragraph" => {
                self.render_paragraph(id)?;
                self.newline();
            }
            "src-block" | "example-block" => {
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
                self.output.push_str(&body);
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
                }
            }
            "keyword" | "property-drawer" | "node-property" | "planning" | "clock" => {}
            "horizontal-rule" => {
                self.newline();
                self.output.push_str("-----\n");
            }
            "table" => self.render_table(id),
            "comment" | "comment-block" => {}
            _ => {
                return Err(format!(
                    "Scheme AOT Markdown projection does not support {kind}"
                ));
            }
        }
        Ok(())
    }

    fn render_paragraph(&mut self, id: usize) -> Result<(), String> {
        let mut cursor = usize::from(self.record(id).range.start());
        let end = usize::from(self.record(id).range.end());
        let children = self.record(id).child_ids.clone();
        for child in children {
            let start = usize::from(self.record(child).range.start());
            let object_end = usize::from(self.record(child).range.end());
            if cursor < start {
                self.output.push_str(&self.source[cursor..start]);
            }
            let object = self.record(child);
            let rendered = match object.kind {
                "bold" => format!("**{}**", object.field("value").unwrap_or_default()),
                "italic" => format!("*{}*", object.field("value").unwrap_or_default()),
                "strike-through" => format!("~~{}~~", object.field("value").unwrap_or_default()),
                "code" | "verbatim" => format!("`{}`", object.field("value").unwrap_or_default()),
                "line-break" => "\\\n".to_owned(),
                "underline" | "subscript" | "superscript" | "entity" | "latex-fragment"
                | "statistics-cookie" | "timestamp" => self.source(object).to_owned(),
                kind => return Err(format!("Scheme AOT Markdown object is unsupported: {kind}")),
            };
            self.output.push_str(&rendered);
            cursor = object_end;
        }
        if cursor < end {
            self.output.push_str(&self.source[cursor..end]);
        }
        Ok(())
    }

    fn render_table(&mut self, id: usize) {
        self.newline();
        let rows = self.record(id).child_ids.clone();
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
