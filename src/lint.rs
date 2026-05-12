//! Org document linting built on the semantic parser projection.

use std::collections::BTreeMap;

use rowan::TextRange;

use crate::{
    ast::{Diagnostic, ParsedAst, SourcePosition, TargetDefinition, TargetKind},
    Org,
};

/// Lint result for one Org source string.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LintReport {
    pub findings: Vec<LintFinding>,
}

impl LintReport {
    /// Returns true when no lint findings were produced.
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }

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

/// One lint finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LintFinding {
    pub code: &'static str,
    pub severity: LintSeverity,
    pub message: String,
    pub location: LintLocation,
}

impl LintFinding {
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
}

/// Finding severity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LintSeverity {
    Error,
    Warning,
}

impl LintSeverity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

/// Source location for one finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LintLocation {
    pub start: SourcePosition,
    pub end: SourcePosition,
    pub range_start: usize,
    pub range_end: usize,
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

/// Lints Org source with the default parser configuration.
pub fn lint_org(source: &str) -> LintReport {
    let org = Org::parse(source);
    lint_document(&org.document(), source)
}

/// Lints an already projected semantic document.
pub fn lint_document(document: &ParsedAst, source: &str) -> LintReport {
    let mut findings = Vec::new();

    findings.extend(
        document
            .diagnostics
            .iter()
            .map(|diagnostic| finding_from_diagnostic(diagnostic, source)),
    );
    findings.extend(duplicate_target_findings(&document.targets, source));

    findings.sort_by(|left, right| {
        left.location
            .range_start
            .cmp(&right.location.range_start)
            .then_with(|| left.code.cmp(right.code))
            .then_with(|| left.message.cmp(&right.message))
    });

    LintReport { findings }
}

fn finding_from_diagnostic(diagnostic: &Diagnostic, source: &str) -> LintFinding {
    LintFinding {
        code: "ORG001",
        severity: LintSeverity::Error,
        message: diagnostic.message.clone(),
        location: location_for_range(source, diagnostic.range),
    }
}

fn duplicate_target_findings(
    targets: &[TargetDefinition<crate::ast::ParsedAnnotation>],
    source: &str,
) -> Vec<LintFinding> {
    let mut by_key = BTreeMap::<&str, Vec<&TargetDefinition<_>>>::new();
    for target in targets {
        by_key.entry(&target.key).or_default().push(target);
    }

    let mut findings = Vec::new();
    for (key, definitions) in by_key {
        if definitions.len() < 2 {
            continue;
        }
        let first = definitions[0];
        findings.push(LintFinding {
            code: "ORG002",
            severity: duplicate_target_severity(key, &definitions),
            message: format!("target `{key}` is defined {} times", definitions.len()),
            location: location_for_range(source, first.ann.range),
        });
    }
    findings
}

fn duplicate_target_severity(
    key: &str,
    definitions: &[&TargetDefinition<crate::ast::ParsedAnnotation>],
) -> LintSeverity {
    if key.starts_with("id:")
        || key.starts_with('#')
        || definitions
            .iter()
            .any(|target| matches!(target.kind, TargetKind::Id | TargetKind::CustomId))
    {
        LintSeverity::Error
    } else {
        LintSeverity::Warning
    }
}

fn location_for_range(source: &str, range: TextRange) -> LintLocation {
    let start = usize::from(range.start()).min(source.len());
    let end = usize::from(range.end()).min(source.len());
    LintLocation {
        start: position_for_offset(source, start),
        end: position_for_offset(source, end),
        range_start: start,
        range_end: end,
    }
}

fn position_for_offset(source: &str, offset: usize) -> SourcePosition {
    let offset = offset.min(source.len());
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let column = source[line_start..offset].chars().count() + 1;
    SourcePosition { line, column }
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
