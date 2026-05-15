//! Stable renderers for Org lint reports.

use crate::lint::{LintFinding, LintLocation, LintReport, LintSeverity};

impl LintReport {
    /// Renders findings as stable, line-oriented text.
    pub fn to_text(&self, path: &str) -> String {
        let mut output = String::new();
        for finding in &self.findings {
            output.push_str(path);
            output.push(':');
            output.push_str(&finding.location.start.line.to_string());
            output.push(':');
            output.push_str(&finding.location.start.column.to_string());
            output.push_str(": ");
            output.push_str(finding.severity.as_str());
            output.push(' ');
            output.push_str(finding.code);
            output.push_str(": ");
            output.push_str(&finding.message);
            output.push('\n');
        }
        output
    }

    /// Renders findings as compact agent-facing repair cards.
    ///
    /// This keeps structured JSON as an explicit machine mode while making the
    /// default text useful for coding agents that need location, source, and a
    /// small repair contract without reading a full audit payload.
    pub fn to_compact_text(&self, path: &str, source: &str) -> String {
        if self.findings.is_empty() {
            return "[ok] orgize lint\n".to_string();
        }

        self.findings
            .iter()
            .map(|finding| finding.to_compact_text(path, source))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Renders findings as a stable JSON object for one file.
    pub fn to_json_file(&self, path: &str) -> String {
        let mut output = String::new();
        output.push_str("{\"path\":\"");
        push_json_string_body(&mut output, path);
        output.push_str("\",\"findings\":[");
        for (index, finding) in self.findings.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            finding.push_json(&mut output);
        }
        output.push_str("]}");
        output
    }
}

impl LintFinding {
    fn to_compact_text(&self, path: &str, source: &str) -> String {
        let mut output = String::new();
        output.push('[');
        output.push_str(self.code);
        output.push_str("] ");
        output.push_str(self.severity.title());
        output.push_str(": ");
        output.push_str(&self.message);
        output.push('\n');
        output.push_str("@ ");
        output.push_str(path);
        output.push(':');
        output.push_str(&self.location.start.line.to_string());
        output.push(':');
        output.push_str(&self.location.start.column.to_string());
        output.push('\n');
        output.push_str("fix: ");
        output.push_str(self.fix_hint());
        output.push('\n');
        if let Some(source_line) = source_line(source, self.location.start.line) {
            output.push_str("line: ");
            output.push_str(&self.location.start.line.to_string());
            output.push_str(" | ");
            output.push_str(source_line);
            output.push('\n');
        }
        output.push_str("Help: ");
        output.push_str(&self.message);
        output.push('\n');
        output.push_str("Contract: ");
        output.push_str(self.contract());
        output.push('\n');
        output
    }

    fn push_json(&self, output: &mut String) {
        output.push_str("{\"code\":\"");
        output.push_str(self.code);
        output.push_str("\",\"severity\":\"");
        output.push_str(self.severity.as_str());
        output.push_str("\",\"message\":\"");
        push_json_string_body(output, &self.message);
        output.push_str("\",\"location\":");
        self.location.push_json(output);
        output.push('}');
    }

    fn fix_hint(&self) -> &'static str {
        match self.code {
            "ORG001" => "repair the Org construct so semantic projection resolves cleanly",
            "ORG002" => "make document-local targets unique or update links to the intended target",
            "ORG003" => "point the include directive at an existing local file or remove it",
            "ORG004" => "add a matching local #+MACRO definition or rename the macro call",
            "ORG005" => "write LINK definitions as #+LINK: name replacement",
            "ORG006" => "keep one LINK abbreviation definition per name",
            "ORG007" => "use supported #+OPTIONS value shapes for parser-v2 export settings",
            "ORG008" => "keep one local #+MACRO definition per name",
            "ORG009" => "declare each TODO keyword once and in only one state group",
            "ORG010" => "write headline priority cookies as [#A], [#B], [#C], or a configured numeric priority",
            "ORG011" => "write EFFORT values with Org duration syntax such as 1:30, 2h, or 1d3h",
            "ORG012" => "keep one value for a property key in each local property drawer scope",
            "ORG013" => "rename the property key to the agenda-sensitive spelling shown in the finding",
            _ => "inspect the Org source near this location and repair the lint finding",
        }
    }

    fn contract(&self) -> &'static str {
        match self.code {
            "ORG001" => "Semantic parser diagnostics must be fixed before export or indexing depends on this source.",
            "ORG002" => "Document-local targets, CUSTOM_ID values, and org-id IDs must resolve without ambiguity.",
            "ORG003" => "Real-file lint runs must not leave local #+INCLUDE paths missing or pointing at directories.",
            "ORG004" => "Macro calls should have document-local definitions when linting for export/index readiness.",
            "ORG005" => "LINK abbreviation keywords must include both an abbreviation name and a replacement.",
            "ORG006" => "LINK abbreviation names should be unique within one document.",
            "ORG007" => "Supported #+OPTIONS keys must use values that parser-v2 can interpret deterministically.",
            "ORG008" => "Macro definition names should be unique before opt-in macro expansion chooses a local template.",
            "ORG009" => "Per-file TODO declarations should not assign one keyword to multiple states or duplicate it.",
            "ORG010" => "Priority cookies affect agenda sorting and matching, so malformed cookies must be made explicit.",
            "ORG011" => "Effort properties feed agenda filters and duration math; invalid durations should not silently degrade to strings.",
            "ORG012" => "Duplicate local properties make inherited agenda/property lookup ambiguous.",
            "ORG013" => "Common agenda property typos should be repaired before lint/index/export consumers depend on them.",
            _ => "Org lint findings should be fixed in source or intentionally reviewed before downstream use.",
        }
    }
}

impl LintSeverity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }

    const fn title(self) -> &'static str {
        match self {
            Self::Error => "Error",
            Self::Warning => "Warning",
        }
    }
}

impl LintLocation {
    fn push_json(&self, output: &mut String) {
        output.push_str("{\"line\":");
        output.push_str(&self.start.line.to_string());
        output.push_str(",\"column\":");
        output.push_str(&self.start.column.to_string());
        output.push_str(",\"end_line\":");
        output.push_str(&self.end.line.to_string());
        output.push_str(",\"end_column\":");
        output.push_str(&self.end.column.to_string());
        output.push_str(",\"range_start\":");
        output.push_str(&self.range_start.to_string());
        output.push_str(",\"range_end\":");
        output.push_str(&self.range_end.to_string());
        output.push('}');
    }
}

fn source_line(source: &str, line_number: usize) -> Option<&str> {
    source
        .lines()
        .nth(line_number.saturating_sub(1))
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
}

fn push_json_string_body(output: &mut String, value: &str) {
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch.is_control() => {
                output.push_str("\\u");
                output.push_str(&format!("{:04x}", ch as u32));
            }
            ch => output.push(ch),
        }
    }
}
