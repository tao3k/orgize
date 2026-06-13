use crate::lint::{LintSeverity, lint_org};

#[test]
fn lint_reports_missing_document_title_and_properties() {
    let report = lint_org(
        "Steer\n\n    To pick up a draggable item, press the space bar.\n    While dragging, use the arrow keys to move the item.\n",
    );

    assert_finding(
        &report,
        "ORG044",
        LintSeverity::Warning,
        "document is missing a #+TITLE keyword",
    );
    assert_finding(
        &report,
        "ORG044",
        LintSeverity::Warning,
        "document is missing document-level properties",
    );
}

#[test]
fn lint_accepts_document_title_and_properties() {
    let report = lint_org("#+TITLE: Steer\n#+PROPERTY: ID steer\n\nSteer instructions.\n");

    assert_no_message(&report, "document is missing a #+TITLE keyword");
    assert_no_message(&report, "document is missing document-level properties");
}

fn assert_finding(
    report: &crate::lint::LintReport,
    code: &'static str,
    severity: LintSeverity,
    message_fragment: &'static str,
) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.code == code
                && finding.severity == severity
                && finding.message.contains(message_fragment)
        }),
        "missing {code}: {:?}",
        report.findings
    );
}

fn assert_no_message(report: &crate::lint::LintReport, message_fragment: &'static str) {
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.message.contains(message_fragment)),
        "unexpected {message_fragment}: {:?}",
        report.findings
    );
}
