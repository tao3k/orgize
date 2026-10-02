//! LaTeX projection over Orgize's Scheme-generated Element graph.

use std::fmt::Write as _;

use gerbil_parser_rowan::GraphRecord;

use crate::entities::ENTITIES;
use crate::export::{LatexEscape, LatexExportOptions};
use crate::org_aot::{OrgAotDocument, org_image_link};

impl OrgAotDocument {
    /// Render a document through its Scheme AOT Element graph.
    ///
    /// # Errors
    /// Returns the first Element or Object without a LaTeX projection.
    pub fn try_to_latex_with_options(&self, options: LatexExportOptions) -> Result<String, String> {
        let source = self.to_org();
        let mut renderer = LatexRenderer {
            document: self,
            source: &source,
            options,
            output: String::new(),
        };
        renderer.render(0)?;
        Ok(renderer.output)
    }

    /// Render with default LaTeX options.
    ///
    /// # Errors
    /// Returns the first unsupported graph kind.
    pub fn try_to_latex(&self) -> Result<String, String> {
        self.try_to_latex_with_options(LatexExportOptions::default())
    }

    /// Render LaTeX or fail explicitly on an unmapped graph kind.
    #[must_use]
    pub fn to_latex_with_options(&self, options: LatexExportOptions) -> String {
        self.try_to_latex_with_options(options)
            .expect("Scheme AOT LaTeX projection does not cover this Org kind")
    }

    /// Render LaTeX with default options.
    #[must_use]
    pub fn to_latex(&self) -> String {
        self.to_latex_with_options(LatexExportOptions::default())
    }

    /// Render one source-backed Element or Object without reparsing it.
    ///
    /// # Errors
    /// Returns an error for an invalid record identity or unsupported kind.
    pub fn try_latex_record(&self, id: usize) -> Result<String, String> {
        if self.records().get(id).is_none() {
            return Err(format!("invalid Org Element record {id}"));
        }
        let source = self.to_org();
        let mut renderer = LatexRenderer {
            document: self,
            source: &source,
            options: LatexExportOptions::default(),
            output: String::new(),
        };
        renderer.render(id)?;
        Ok(renderer.output)
    }
}

struct LatexRenderer<'a> {
    document: &'a OrgAotDocument,
    source: &'a str,
    options: LatexExportOptions,
    output: String,
}

impl LatexRenderer<'_> {
    fn record(&self, id: usize) -> &GraphRecord {
        &self.document.records()[id]
    }

    fn bounds(&self, id: usize) -> (usize, usize) {
        let range = self.record(id).range;
        (usize::from(range.start()), usize::from(range.end()))
    }

    fn newline(&mut self) {
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }

    fn blank_line(&mut self) {
        if !self.output.is_empty() && !self.output.ends_with("\n\n") {
            self.newline();
            self.output.push('\n');
        }
    }

    fn text(output: &mut String, value: &str, special: bool) {
        if special {
            let converted = value
                .replace("---", "\u{2014}")
                .replace("--", "\u{2013}")
                .replace("...", "\u{2026}")
                .replace("\\-", "\u{00AD}")
                .replace('\'', "\u{2019}");
            let _ = write!(output, "{}", LatexEscape(converted));
        } else {
            let _ = write!(output, "{}", LatexEscape(value));
        }
    }

    fn span(&mut self, start: usize, end: usize, children: &[usize]) -> Result<(), String> {
        let mut cursor = start;
        for &child in children {
            let (child_start, child_end) = self.bounds(child);
            if child_start < cursor || child_end > end {
                continue;
            }
            Self::text(
                &mut self.output,
                &self.source[cursor..child_start],
                self.options.special_strings,
            );
            self.render(child)?;
            cursor = child_end;
        }
        Self::text(
            &mut self.output,
            &self.source[cursor..end],
            self.options.special_strings,
        );
        Ok(())
    }

    fn render(&mut self, id: usize) -> Result<(), String> {
        let kind = self.record(id).kind;
        match kind {
            "org-data" | "section" => {
                for child in self.record(id).child_ids.clone() {
                    self.render(child)?;
                }
            }
            "headline" => self.headline(id)?,
            "paragraph" => self.paragraph(id)?,
            "bold" | "italic" | "underline" | "strike-through" | "code" | "verbatim"
            | "subscript" | "superscript" => self.markup(id),
            "link" => self.link(id)?,
            "entity" => self.entity(id),
            "latex-fragment" => self.output.push_str(
                self.document.records()[id]
                    .field("value")
                    .unwrap_or_default(),
            ),
            "latex-environment" => {
                self.newline();
                let (start, end) = self.bounds(id);
                self.output.push_str(&self.source[start..end]);
                self.newline();
            }
            "timestamp" => {
                let (start, end) = self.bounds(id);
                let _ = write!(
                    self.output,
                    "\\textit{{{}}}",
                    LatexEscape(&self.source[start..end])
                );
            }
            "citation" => self.citation(id),
            "line-break" => self.output.push_str("\\\\\n"),
            "export-snippet" => {
                if self
                    .record(id)
                    .field("backend")
                    .is_some_and(|backend| backend.eq_ignore_ascii_case("latex"))
                {
                    self.output.push_str(
                        self.document.records()[id]
                            .field("value")
                            .unwrap_or_default(),
                    );
                }
            }
            "src-block" | "example-block" | "fixed-width" => self.verbatim(id),
            "export-block" => {
                if self
                    .record(id)
                    .field("backend")
                    .is_some_and(|backend| backend.eq_ignore_ascii_case("latex"))
                {
                    self.newline();
                    self.output.push_str(
                        self.document.records()[id]
                            .field("body")
                            .unwrap_or_default(),
                    );
                    self.newline();
                }
            }
            "quote-block" | "verse-block" | "center-block" => self.container(id)?,
            "plain-list" => self.list(id)?,
            "item" => self.item(id)?,
            "footnote-definition" => {
                let label = self
                    .record(id)
                    .field("label")
                    .unwrap_or_default()
                    .to_owned();
                self.blank_line();
                let _ = write!(
                    self.output,
                    "\\begin{{quote}}\\textsuperscript{{{}}}",
                    LatexEscape(&label)
                );
                for child in self.record(id).child_ids.clone() {
                    self.render(child)?;
                }
                self.newline();
                self.output.push_str("\\end{quote}\n");
            }
            "table" => self.table(id)?,
            "horizontal-rule" => {
                self.newline();
                self.output
                    .push_str("\\noindent\\rule{\\linewidth}{0.4pt}\n");
            }
            "target" | "radio-target" => self.target(id),
            "footnote-reference" => {
                let (start, end) = self.bounds(id);
                let _ = write!(
                    self.output,
                    "\\textsuperscript{{{}}}",
                    LatexEscape(&self.source[start..end])
                );
            }
            "keyword" | "planning" | "clock" | "property-drawer" | "node-property" | "comment"
            | "comment-block" => {}
            "statistics-cookie" | "macro" | "inline-babel-call" => {
                let (start, end) = self.bounds(id);
                Self::text(
                    &mut self.output,
                    &self.source[start..end],
                    self.options.special_strings,
                );
            }
            "inline-src-block" => {
                let value = self.document.records()[id]
                    .field("value")
                    .unwrap_or_default();
                let _ = write!(self.output, "\\texttt{{{}}}", LatexEscape(value));
            }
            _ => {
                return Err(format!(
                    "Scheme AOT LaTeX projection does not support {kind}"
                ));
            }
        }
        Ok(())
    }

    fn headline(&mut self, id: usize) -> Result<(), String> {
        self.blank_line();
        let level = self.record(id).field("markers").map_or(1, str::len).min(6);
        let command = match level {
            1 => "\\section",
            2 => "\\subsection",
            3 => "\\subsubsection",
            4 => "\\paragraph",
            5 => "\\subparagraph",
            _ => "\\textbf",
        };
        let title = self
            .record(id)
            .field("title")
            .unwrap_or_default()
            .to_owned();
        let (start, end) = self.bounds(id);
        let line_end = self.source[start..end]
            .find('\n')
            .map_or(end, |offset| start + offset);
        let after_markers = start + self.record(id).field("markers").map_or(0, str::len);
        let title_start = self.source[after_markers..line_end]
            .find(&title)
            .map_or(line_end, |offset| after_markers + offset);
        let title_end = title_start + title.len();
        let _ = write!(self.output, "{command}{{");
        self.span(title_start, title_end, &self.record(id).child_ids.clone())?;
        self.output.push_str("}\n");
        let body: Vec<_> = self
            .record(id)
            .child_ids
            .iter()
            .copied()
            .filter(|&child| usize::from(self.record(child).range.start()) >= line_end)
            .collect();
        for child in body {
            self.render(child)?;
        }
        Ok(())
    }

    fn paragraph(&mut self, id: usize) -> Result<(), String> {
        let (start, end) = self.bounds(id);
        if self.source[..start].ends_with("\n\n") {
            self.blank_line();
        }
        let trailing = self.source[start..end]
            .bytes()
            .rev()
            .take_while(|byte| *byte == b'\n')
            .count();
        let end = end.saturating_sub(trailing.saturating_sub(1));
        self.span(start, end, &self.record(id).child_ids.clone())?;
        self.blank_line();
        Ok(())
    }

    fn markup(&mut self, id: usize) {
        let command = match self.record(id).kind {
            "bold" => "\\textbf",
            "italic" => "\\emph",
            "underline" => "\\underline",
            "strike-through" => "\\sout",
            "subscript" => "\\textsubscript",
            "superscript" => "\\textsuperscript",
            _ => "\\texttt",
        };
        let value = self.document.records()[id]
            .field("value")
            .unwrap_or_default();
        let _ = write!(self.output, "{command}{{{}}}", LatexEscape(value));
    }

    fn link(&mut self, id: usize) -> Result<(), String> {
        let path = self.record(id).field("path").unwrap_or_default().to_owned();
        let href = path.trim_start_matches("file:");
        if org_image_link(href) {
            let _ = write!(self.output, "\\includegraphics{{{}}}", LatexEscape(href));
        } else if let Some(description) = self.record(id).field("description") {
            let description_len = description.len();
            let (_, end) = self.bounds(id);
            let description_end = end.saturating_sub(2);
            let description_start = description_end.saturating_sub(description_len);
            let _ = write!(self.output, "\\href{{{}}}{{", LatexEscape(href));
            self.span(
                description_start,
                description_end,
                &self.record(id).child_ids.clone(),
            )?;
            self.output.push('}');
        } else {
            let _ = write!(self.output, "\\url{{{}}}", LatexEscape(href));
        }
        Ok(())
    }

    fn entity(&mut self, id: usize) {
        let name = self.record(id).field("name").unwrap_or_default();
        if self.options.expand_entities
            && let Some(entity) = ENTITIES.iter().find(|entity| entity.0 == name)
        {
            if entity.2 {
                let _ = write!(self.output, "${}$", entity.1);
            } else {
                self.output.push_str(entity.1);
            }
        } else {
            let (start, end) = self.bounds(id);
            Self::text(
                &mut self.output,
                &self.source[start..end],
                self.options.special_strings,
            );
        }
    }

    fn citation(&mut self, id: usize) {
        let keys: Vec<_> = self
            .record(id)
            .child_ids
            .iter()
            .filter_map(|&child| self.record(child).field("key"))
            .collect();
        if keys.is_empty() {
            let (start, end) = self.bounds(id);
            Self::text(
                &mut self.output,
                &self.source[start..end],
                self.options.special_strings,
            );
        } else {
            let _ = write!(self.output, "\\cite{{{}}}", keys.join(","));
        }
    }

    fn verbatim(&mut self, id: usize) {
        self.environment_begin("verbatim");
        let body = self.document.records()[id]
            .field("body")
            .unwrap_or_default();
        self.output.push_str(body);
        self.newline();
        self.environment_end("verbatim");
    }

    fn environment_begin(&mut self, name: &str) {
        self.newline();
        let _ = writeln!(self.output, "\\begin{{{name}}}");
    }

    fn environment_end(&mut self, name: &str) {
        self.newline();
        let _ = writeln!(self.output, "\\end{{{name}}}");
    }

    fn container(&mut self, id: usize) -> Result<(), String> {
        let name = match self.record(id).kind {
            "quote-block" => "quote",
            "verse-block" => "verse",
            _ => "center",
        };
        self.environment_begin(name);
        for child in self.record(id).child_ids.clone() {
            self.render(child)?;
        }
        self.environment_end(name);
        Ok(())
    }

    fn list(&mut self, id: usize) -> Result<(), String> {
        let children = self.record(id).child_ids.clone();
        let first = children.first().ok_or("Scheme AOT list has no items")?;
        let bullet = self.record(*first).field("bullet").unwrap_or_default();
        let name = if self.record(*first).field("tag").is_some() {
            "description"
        } else if bullet
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_alphanumeric())
        {
            "enumerate"
        } else {
            "itemize"
        };
        self.environment_begin(name);
        for child in children {
            self.item(child)?;
        }
        self.environment_end(name);
        Ok(())
    }

    fn item(&mut self, id: usize) -> Result<(), String> {
        self.newline();
        if let Some(tag) = self.document.records()[id].field("tag") {
            let _ = write!(self.output, "\\item[{}] ", LatexEscape(tag));
        } else {
            self.output.push_str("\\item ");
        }
        for child in self.record(id).child_ids.clone() {
            self.render(child)?;
        }
        self.newline();
        Ok(())
    }

    fn table(&mut self, id: usize) -> Result<(), String> {
        let rows = self.record(id).child_ids.clone();
        let columns = rows
            .iter()
            .filter(|&&row| self.record(row).kind == "table-row")
            .map(|&row| self.record(row).child_ids.len())
            .max()
            .unwrap_or(1)
            .max(1);
        self.newline();
        let _ = writeln!(self.output, "\\begin{{tabular}}{{{}}}", "l".repeat(columns));
        for row in rows {
            let record = self.record(row);
            if record.kind == "keyword"
                && record
                    .field("key")
                    .is_some_and(|key| key.eq_ignore_ascii_case("TBLFM"))
            {
                continue;
            }
            if self.record(row).kind == "table-rule-row" {
                self.output.push_str("\\hline\n");
                continue;
            }
            if self.record(row).kind != "table-row" {
                return Err("Scheme AOT table has an unsupported row".to_owned());
            }
            let cells = self.record(row).child_ids.clone();
            for (index, cell) in cells.into_iter().enumerate() {
                if index > 0 {
                    self.output.push_str(" & ");
                }
                Self::text(
                    &mut self.output,
                    self.document.records()[cell]
                        .field("text")
                        .unwrap_or_default()
                        .trim(),
                    self.options.special_strings,
                );
            }
            self.output.push_str(" \\\\\n");
        }
        self.output.push_str("\\end{tabular}\n");
        Ok(())
    }

    fn target(&mut self, id: usize) {
        let label = self.record(id).field("value").unwrap_or_default();
        let mut sanitized = String::with_capacity(label.len());
        for ch in label.chars() {
            if ch.is_ascii_alphanumeric() || matches!(ch, ':' | '_' | '-' | '.') {
                sanitized.push(ch);
            } else if !sanitized.ends_with('-') {
                sanitized.push('-');
            }
        }
        let label = sanitized.trim_matches('-');
        if !label.is_empty() {
            let _ = write!(self.output, "\\label{{{label}}}");
        }
    }
}
