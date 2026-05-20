//! Stable renderers for Org lint reports.

use super::lint_model::{LintFinding, LintLocation, LintReport, LintSeverity};

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
            "ORG010" => {
                "write headline priority cookies as [#A], [#B], [#C], or a configured numeric priority"
            }
            "ORG011" => "write EFFORT values with Org duration syntax such as 1:30, 2h, or 1d3h",
            "ORG012" => "keep one value for a property key in each local property drawer scope",
            "ORG013" => {
                "rename the property key to the agenda-sensitive spelling shown in the finding"
            }
            "ORG014" => {
                "write LOGBOOK lifecycle lines with the standard quoted state-change or CLOCK shape"
            }
            "ORG015" => "provide an archive destination such as archive.org::* Archived",
            "ORG016" => {
                "add a DIR, ATTACH_DIR, or ID property for the entry, or fix the attachment link target"
            }
            "ORG017" => "point the file link at an existing local file or remove the link",
            "ORG018" => {
                "point archive metadata at an existing local file and heading, or keep it remote/unresolved intentionally"
            }
            "ORG019" => {
                "point refile lifecycle metadata at an existing local file and heading, or repair the LOGBOOK line"
            }
            "ORG020" => {
                "rename the duplicate source block or update callers to one canonical block name"
            }
            "ORG021" => {
                "add a matching local source block, noweb-ref, or rename the Babel reference"
            }
            "ORG022" => {
                "review the explicit :eval header and prefer no/query only when execution is intentional"
            }
            "ORG023" => "set :tangle to yes, no, or a non-empty target path",
            "ORG024" => {
                "write TBLFM assignments as supported Org table targets with non-empty right-hand sides"
            }
            "ORG025" => "keep one formula assignment per TBLFM target in each active TBLFM line",
            "ORG026" => {
                "point table formula row and column references inside the current table shape"
            }
            "ORG027" => {
                "set COOKIE_DATA to todo, checkbox, or direct so Org can update this statistics cookie deterministically"
            }
            "ORG028" => {
                "update the statistics cookie to the expected Org count, or repair the TODO/checkbox source it summarizes"
            }
            "ORG029" => {
                "finish, reorder, or explicitly defer this task until the previous open ORDERED sibling is complete"
            }
            "ORG030" => {
                "choose one of the inherited PROPERTY_ALL values, or update the allowed-value descriptor"
            }
            "ORG031" => "add a UUID-shaped or ULID-shaped ID property to the SDD heading",
            "ORG032" => "set SDD_KIND to system, capability, view, decision, or audit",
            "ORG033" => {
                "set SDD_PARENT to an Org id link that resolves to a visible parent SDD node"
            }
            "ORG034" => "assign a unique ID to each SDD heading",
            "ORG035" => "add at least one direct Scenario child heading under this SDD requirement",
            "ORG036" => {
                "move task state, progress cookies, and checklists from SDD headings into an Org task or ExecPlan"
            }
            "ORG037" => "add the architecture metadata required by this SDD_KIND",
            "ORG038" => {
                "treat this crypt-tagged subtree body as opaque for indexing/export, or remove the crypt tag if the body is intentionally public"
            }
            "ORG039" => {
                "add the crypt tag to the section or remove the CRYPTKEY property if it is not an Org Crypt entry"
            }
            _ => "inspect the Org source near this location and repair the lint finding",
        }
    }

    fn contract(&self) -> &'static str {
        match self.code {
            "ORG001" => {
                "Semantic parser diagnostics must be fixed before export or indexing depends on this source."
            }
            "ORG002" => {
                "Document-local targets, CUSTOM_ID values, and org-id IDs must resolve without ambiguity."
            }
            "ORG003" => {
                "Real-file lint runs must not leave local #+INCLUDE paths missing or pointing at directories."
            }
            "ORG004" => {
                "Macro calls should have document-local definitions when linting for export/index readiness."
            }
            "ORG005" => {
                "LINK abbreviation keywords must include both an abbreviation name and a replacement."
            }
            "ORG006" => "LINK abbreviation names should be unique within one document.",
            "ORG007" => {
                "Supported #+OPTIONS keys must use values that parser-v2 can interpret deterministically."
            }
            "ORG008" => {
                "Macro definition names should be unique before opt-in macro expansion chooses a local template."
            }
            "ORG009" => {
                "Per-file TODO declarations should not assign one keyword to multiple states or duplicate it."
            }
            "ORG010" => {
                "Priority cookies affect agenda sorting and matching, so malformed cookies must be made explicit."
            }
            "ORG011" => {
                "Effort properties feed agenda filters and duration math; invalid durations should not silently degrade to strings."
            }
            "ORG012" => {
                "Duplicate local properties make inherited agenda/property lookup ambiguous."
            }
            "ORG013" => {
                "Common agenda property typos should be repaired before lint/index/export consumers depend on them."
            }
            "ORG014" => {
                "Lifecycle history is used by memory and agenda projections, so malformed LOGBOOK events should be repaired."
            }
            "ORG015" => {
                "Archive metadata should point at a resolvable archive destination before archive-aware tooling consumes it."
            }
            "ORG016" => {
                "Attachment links are file-like Org links and need a resolvable attachment directory when linting real files."
            }
            "ORG017" => {
                "Ordinary file links should resolve when linting real files, while remote file links remain parser-only metadata."
            }
            "ORG018" => {
                "Archive destinations should be resolvable in real-file lint runs before archive-aware tooling consumes them."
            }
            "ORG019" => {
                "Refile lifecycle destinations should be resolvable in real-file lint runs before memory projections treat them as provenance."
            }
            "ORG020" => {
                "Babel source block names should be unique before calls, result association, or noweb references consume them."
            }
            "ORG021" => {
                "Babel call and noweb references should point at a local block name or noweb-ref before agent tooling depends on them."
            }
            "ORG022" => {
                "Explicit eval-sensitive headers should be reviewed before automated tooling treats source blocks as executable context."
            }
            "ORG023" => {
                "Tangle metadata should have a deterministic target when it is not explicitly yes or no."
            }
            "ORG024" => {
                "TBLFM formulas should use Org-supported left-hand targets and complete assignments before table tooling consumes them."
            }
            "ORG025" => {
                "Each TBLFM target should have one active definition so agents and exporters do not guess which formula wins."
            }
            "ORG026" => {
                "Absolute table formula references should stay within the table rows and columns visible to this parser projection."
            }
            "ORG027" => {
                "Statistics cookies should declare whether TODO, checkbox, or direct mixed progress owns the count when both evidence types are present."
            }
            "ORG028" => {
                "Statistics cookies should match the parsed TODO or checkbox progress before downstream planning tools consume them."
            }
            "ORG029" => {
                "Blocked-state advice must be derived from native local ORDERED sibling evidence, not custom dependency syntax."
            }
            "ORG030" => {
                "Property allowed-value advice must be derived from native inherited PROPERTY_ALL descriptors."
            }
            "ORG031" => {
                "SDD nodes need stable machine identity in ID while keeping semantic naming in headings, tags, and properties."
            }
            "ORG032" => {
                "SDD node kinds must stay in the supported architecture-description hierarchy so status projections remain deterministic."
            }
            "ORG033" => {
                "SDD parent edges should be native Org id links with readable labels, not path or stringly hierarchy conventions."
            }
            "ORG034" => {
                "SDD ID values must resolve to one visible node before parent edges and archives depend on them."
            }
            "ORG035" => {
                "SDD requirements should have directly visible scenarios so Agent and test planning can verify behavior."
            }
            "ORG036" => {
                "SDD describes architecture and audit rationale; implementation progress belongs to plan/task surfaces."
            }
            "ORG037" => {
                "Architecture descriptions need explicit concerns, viewpoints, or rationale instead of task-only labels."
            }
            "ORG038" => {
                "Org Crypt protects subtree body text, while headline and property metadata remain visible to parsers and indexers."
            }
            "ORG039" => {
                "CRYPTKEY only selects an encryption key for entries matched by the Org Crypt tag matcher."
            }
            _ => {
                "Org lint findings should be fixed in source or intentionally reviewed before downstream use."
            }
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
