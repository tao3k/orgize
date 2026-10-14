//! HTML rendering over the single Scheme-generated Org Element graph.

use std::fmt::Write as _;

use gerbil_parser_runtime::GraphRecord;

use crate::entities::ENTITIES;
use crate::export::{
    HtmlEscape, HtmlExportOptions, safe_source_block_data_attributes, special_strings,
};
use crate::org_aot::OrgAotDocument;

impl OrgAotDocument {
    /// Render the Scheme AOT graph as HTML.
    ///
    /// # Errors
    /// Returns the first Element or Object kind with no graph-backed HTML projection.
    pub fn try_to_html_with_options(&self, options: HtmlExportOptions) -> Result<String, String> {
        let source = self.to_org();
        let mut renderer = HtmlRenderer {
            document: self,
            source: &source,
            options,
            output: String::new(),
            headline_anchor: None,
        };
        renderer.render_document()?;
        Ok(renderer.output)
    }

    /// Render the Scheme AOT graph as HTML using default options.
    ///
    /// # Errors
    /// Returns the first unsupported Element or Object kind.
    pub fn try_to_html(&self) -> Result<String, String> {
        self.try_to_html_with_options(HtmlExportOptions::default())
    }

    /// Render HTML with a caller-owned headline anchor projection.
    ///
    /// The callback only controls presentation; headline recognition and title
    /// Objects still come from the Scheme-generated graph.
    ///
    /// # Errors
    /// Returns the first unsupported Element or Object kind.
    pub fn try_to_html_with_headline_anchor(
        &self,
        mut anchor: impl FnMut(&str) -> String,
    ) -> Result<String, String> {
        let source = self.to_org();
        let mut renderer = HtmlRenderer {
            document: self,
            source: &source,
            options: HtmlExportOptions::default(),
            output: String::new(),
            headline_anchor: Some(&mut anchor),
        };
        renderer.render_document()?;
        Ok(renderer.output)
    }

    /// Render Org as HTML, failing explicitly on an unimplemented graph kind.
    #[must_use]
    pub fn to_html_with_options(&self, options: HtmlExportOptions) -> String {
        self.try_to_html_with_options(options)
            .expect("Scheme AOT HTML projection does not cover this Org kind")
    }

    /// Render Org as HTML with default options.
    #[must_use]
    pub fn to_html(&self) -> String {
        self.to_html_with_options(HtmlExportOptions::default())
    }
}

struct HtmlRenderer<'a> {
    document: &'a OrgAotDocument,
    source: &'a str,
    options: HtmlExportOptions,
    output: String,
    headline_anchor: Option<&'a mut dyn FnMut(&str) -> String>,
}

impl HtmlRenderer<'_> {
    fn record(&self, id: usize) -> &GraphRecord {
        &self.document.records()[id]
    }

    fn bounds(&self, id: usize) -> (usize, usize) {
        let range = self.record(id).range;
        (usize::from(range.start()), usize::from(range.end()))
    }

    fn source(&self, id: usize) -> &str {
        let (start, end) = self.bounds(id);
        &self.source[start..end]
    }

    fn write_text(output: &mut String, value: &str, special: bool) {
        if special {
            let _ = write!(output, "{}", HtmlEscape(special_strings(value)));
        } else {
            let _ = write!(output, "{}", HtmlEscape(value));
        }
    }

    fn text(&mut self, value: &str) {
        Self::write_text(&mut self.output, value, self.options.special_strings);
    }

    fn render_document(&mut self) -> Result<(), String> {
        self.output.push_str("<main>");
        self.render_sections(&self.record(0).child_ids.clone())?;
        self.output.push_str("</main>");
        Ok(())
    }

    fn render_sections(&mut self, children: &[usize]) -> Result<(), String> {
        let mut section_open = false;
        for &id in children {
            if self.record(id).kind == "headline" {
                if section_open {
                    self.output.push_str("</section>");
                    section_open = false;
                }
                self.render_headline(id)?;
            } else {
                if !section_open {
                    self.output.push_str("<section>");
                    section_open = true;
                }
                self.render(id)?;
            }
        }
        if section_open {
            self.output.push_str("</section>");
        }
        Ok(())
    }

    fn render_headline(&mut self, id: usize) -> Result<(), String> {
        let level = self
            .record(id)
            .field("markers")
            .map_or(1, str::len)
            .clamp(1, 6);
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
        let _ = write!(self.output, "<h{level}>");
        let anchor = self
            .headline_anchor
            .as_mut()
            .map(|callback| callback(&title));
        if let Some(ref anchor) = anchor {
            let _ = write!(
                self.output,
                "<a id=\"{}\" href=\"#{}\">",
                HtmlEscape(anchor),
                HtmlEscape(anchor)
            );
        }
        self.render_inline_span(title_start, title_end, &self.record(id).child_ids.clone())?;
        if anchor.is_some() {
            self.output.push_str("</a>");
        }
        let _ = write!(self.output, "</h{level}>");
        let body: Vec<_> = self
            .record(id)
            .child_ids
            .iter()
            .copied()
            .filter(|&child| usize::from(self.record(child).range.start()) >= line_end)
            .collect();
        self.render_sections(&body)?;
        Ok(())
    }

    fn render_inline_span(
        &mut self,
        start: usize,
        end: usize,
        children: &[usize],
    ) -> Result<(), String> {
        let mut cursor = start;
        for &id in children {
            let (child_start, child_end) = self.bounds(id);
            if child_start < cursor || child_end > end {
                continue;
            }
            Self::write_text(
                &mut self.output,
                &self.source[cursor..child_start],
                self.options.special_strings,
            );
            self.render(id)?;
            cursor = child_end;
        }
        Self::write_text(
            &mut self.output,
            &self.source[cursor..end],
            self.options.special_strings,
        );
        Ok(())
    }

    fn render(&mut self, id: usize) -> Result<(), String> {
        let kind = self.record(id).kind;
        match kind {
            "paragraph" => {
                let (start, end) = self.bounds(id);
                let trailing = self.source[start..end]
                    .bytes()
                    .rev()
                    .take_while(|byte| *byte == b'\n')
                    .count();
                let end = end.saturating_sub(trailing.saturating_sub(1));
                self.output.push_str("<p>");
                self.render_inline_span(start, end, &self.record(id).child_ids.clone())?;
                self.output.push_str("</p>");
            }
            "plain-list" => self.render_list(id)?,
            "item" => self.render_item(id)?,
            "bold" | "italic" | "underline" | "strike-through" | "code" | "verbatim"
            | "subscript" | "superscript" => self.render_markup(id),
            "link" => self.render_link(id)?,
            "entity" => self.render_entity(id),
            "line-break" => self.output.push_str("<br/>"),
            "export-snippet" => {
                if self
                    .record(id)
                    .field("backend")
                    .is_some_and(|backend| backend.eq_ignore_ascii_case("html"))
                {
                    self.output.push_str(
                        self.document.records()[id]
                            .field("value")
                            .unwrap_or_default(),
                    );
                }
            }
            "citation" | "timestamp" | "latex-fragment" | "latex-environment" => {
                let (start, end) = self.bounds(id);
                self.output.push_str(&self.source[start..end]);
            }
            "statistics-cookie" | "macro" | "inline-babel-call" => {
                let (start, end) = self.bounds(id);
                let _ = write!(self.output, "{}", HtmlEscape(&self.source[start..end]));
            }
            "footnote-reference" => {
                let label = self.document.records()[id]
                    .field("label")
                    .unwrap_or_default();
                if label.is_empty() {
                    let (start, end) = self.bounds(id);
                    let _ = write!(self.output, "{}", HtmlEscape(&self.source[start..end]));
                } else {
                    let _ = write!(
                        self.output,
                        "<sup class=\"footnote-reference\"><a href=\"#fn-{}\">{}</a></sup>",
                        HtmlEscape(label),
                        HtmlEscape(label)
                    );
                }
            }
            "footnote-definition" => self.render_footnote_definition(id)?,
            "src-block" => self.render_source_block(id),
            "inline-src-block" => {
                let record = &self.document.records()[id];
                let language = record.field("language").unwrap_or_default();
                let value = record.field("value").unwrap_or_default();
                let _ = write!(
                    self.output,
                    "<code class=\"src src-{}\">{}</code>",
                    HtmlEscape(language),
                    HtmlEscape(value)
                );
            }
            "example-block" | "fixed-width" => {
                self.output.push_str("<pre class=\"example\">");
                let body = self.record(id).field("body").unwrap_or_default().to_owned();
                self.text(&body);
                self.output.push_str("</pre>");
            }
            "export-block" => {
                if self
                    .record(id)
                    .field("backend")
                    .is_some_and(|backend| backend.eq_ignore_ascii_case("html"))
                {
                    self.output.push_str(
                        self.document.records()[id]
                            .field("body")
                            .unwrap_or_default(),
                    );
                }
            }
            "quote-block" | "verse-block" | "center-block" => self.render_container(id)?,
            "horizontal-rule" => self.output.push_str("<hr/>"),
            "keyword" | "babel-call" | "planning" | "clock" | "property-drawer"
            | "node-property" | "target" | "radio-target" => {}
            "comment" | "comment-block" => {
                self.output.push_str("<!--");
                let (start, end) = self.bounds(id);
                self.output.push_str(&self.source[start..end]);
                self.output.push_str("-->");
            }
            "table" => self.render_table(id)?,
            "table-cell" => Self::write_text(
                &mut self.output,
                self.document.records()[id]
                    .field("text")
                    .unwrap_or_default()
                    .trim(),
                self.options.special_strings,
            ),
            "headline" => self.render_headline(id)?,
            "org-data" => self.render_document()?,
            _ => {
                return Err(format!(
                    "Scheme AOT HTML projection does not support {kind}"
                ));
            }
        }
        Ok(())
    }

    fn render_markup(&mut self, id: usize) {
        let tag = match self.record(id).kind {
            "bold" => "b",
            "italic" => "i",
            "underline" => "u",
            "strike-through" => "s",
            "subscript" => "sub",
            "superscript" => "sup",
            _ => "code",
        };
        let value = self
            .record(id)
            .field("value")
            .unwrap_or_default()
            .to_owned();
        let _ = write!(self.output, "<{tag}>");
        self.text(&value);
        let _ = write!(self.output, "</{tag}>");
    }

    fn render_link(&mut self, id: usize) -> Result<(), String> {
        let path = self.record(id).field("path").unwrap_or_default().to_owned();
        let href = path.trim_start_matches("file:");
        let _ = write!(self.output, "<a href=\"{}\">", HtmlEscape(href));
        if let Some(description) = self.record(id).field("description") {
            let description = description.to_owned();
            let (_, end) = self.bounds(id);
            let description_end = end.saturating_sub(2);
            let description_start = description_end.saturating_sub(description.len());
            self.render_inline_span(
                description_start,
                description_end,
                &self.record(id).child_ids.clone(),
            )?;
        } else {
            self.text(href);
        }
        self.output.push_str("</a>");
        Ok(())
    }

    fn render_entity(&mut self, id: usize) {
        let name = self.record(id).field("name").unwrap_or_default();
        if self.options.expand_entities
            && let Some(entity) = ENTITIES.iter().find(|entity| entity.0 == name)
        {
            self.output.push_str(entity.3);
        } else {
            let (start, end) = self.bounds(id);
            Self::write_text(
                &mut self.output,
                &self.source[start..end],
                self.options.special_strings,
            );
        }
    }

    fn render_source_block(&mut self, id: usize) {
        let language = self
            .record(id)
            .field("language")
            .unwrap_or_default()
            .to_owned();
        if language.is_empty() {
            self.output.push_str("<pre");
        } else {
            let _ = write!(
                self.output,
                "<pre class=\"src src-{}\"",
                HtmlEscape(&language)
            );
        }
        let attributes: Vec<_> = self
            .document
            .affiliated_keyword_ids(id)
            .iter()
            .flat_map(|&keyword_id| safe_source_block_data_attributes(self.source(keyword_id)))
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect();
        for (name, value) in attributes {
            let _ = write!(self.output, " {name}=\"{}\"", HtmlEscape(value));
        }
        if language.is_empty() {
            self.output.push_str("><code>");
        } else {
            let _ = write!(
                self.output,
                "><code class=\"language-{}\">",
                HtmlEscape(&language)
            );
        }
        Self::write_text(
            &mut self.output,
            self.document.records()[id]
                .field("body")
                .unwrap_or_default(),
            self.options.special_strings,
        );
        self.output.push_str("</code></pre>");
    }

    fn render_container(&mut self, id: usize) -> Result<(), String> {
        let (open, close) = match self.record(id).kind {
            "quote-block" => ("<blockquote>", "</blockquote>"),
            "verse-block" => ("<p class=\"verse\">", "</p>"),
            _ => ("<div class=\"center\">", "</div>"),
        };
        self.output.push_str(open);
        for child in self.record(id).child_ids.clone() {
            self.render(child)?;
        }
        self.output.push_str(close);
        Ok(())
    }

    fn render_footnote_definition(&mut self, id: usize) -> Result<(), String> {
        let label = self
            .record(id)
            .field("label")
            .unwrap_or_default()
            .to_owned();
        let _ = write!(
            self.output,
            "<aside class=\"footnote\" id=\"fn-{}\">",
            HtmlEscape(&label)
        );
        for child in self.record(id).child_ids.clone() {
            self.render(child)?;
        }
        self.output.push_str("</aside>");
        Ok(())
    }

    fn render_table(&mut self, id: usize) -> Result<(), String> {
        self.output.push_str("<table>");
        let rows = self.record(id).child_ids.clone();
        let first_rule = rows
            .iter()
            .position(|&row| self.record(row).kind == "table-rule-row");
        let has_header = first_rule.is_some_and(|position| position > 0);
        let mut group = "";
        let mut seen_rule = false;
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
                if !group.is_empty() {
                    let _ = write!(self.output, "</{group}>");
                    group = "";
                }
                seen_rule = true;
                continue;
            }
            if self.record(row).kind != "table-row" {
                return Err("Scheme AOT table has an unsupported row".to_owned());
            }
            if group.is_empty() {
                group = if has_header && !seen_rule {
                    "thead"
                } else {
                    "tbody"
                };
                let _ = write!(self.output, "<{group}>");
            }
            self.output.push_str("<tr>");
            for cell in self.record(row).child_ids.clone() {
                self.output.push_str("<td>");
                self.render(cell)?;
                self.output.push_str("</td>");
            }
            self.output.push_str("</tr>");
        }
        if !group.is_empty() {
            let _ = write!(self.output, "</{group}>");
        }
        self.output.push_str("</table>");
        Ok(())
    }

    fn render_list(&mut self, id: usize) -> Result<(), String> {
        let children = self.record(id).child_ids.clone();
        let first = children.first().ok_or("Scheme AOT list has no items")?;
        let bullet = self.record(*first).field("bullet").unwrap_or_default();
        let tag = if bullet
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_alphanumeric())
        {
            "ol"
        } else {
            "ul"
        };
        let _ = write!(self.output, "<{tag}>");
        for item in children {
            if self.record(item).kind != "item" {
                return Err("Scheme AOT list has an unsupported child".to_owned());
            }
            self.render_item(item)?;
        }
        let _ = write!(self.output, "</{tag}>");
        Ok(())
    }

    fn render_item(&mut self, id: usize) -> Result<(), String> {
        self.output.push_str("<li>");
        for child in self.record(id).child_ids.clone() {
            self.render(child)?;
        }
        self.output.push_str("</li>");
        Ok(())
    }
}
