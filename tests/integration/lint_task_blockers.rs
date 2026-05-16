use orgize::lint::lint_org;

#[test]
fn lint_reports_ordered_sibling_blocker_advice() {
    let source = r#"* TODO Project
:PROPERTIES:
:ORDERED: t
:END:
** TODO First
** TODO Second
*** TODO Nested A
*** TODO Nested B
** DONE Finished
** TODO Third
"#;

    let report = lint_org(source);
    let findings = &report.findings;

    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].code, "ORG029");
    assert_eq!(findings[0].location.start.line, 6);
    assert!(findings[0]
        .message
        .contains("task `Second` is blocked by previous open sibling `First`"));
    assert_eq!(findings[1].code, "ORG029");
    assert_eq!(findings[1].location.start.line, 10);
    assert!(findings[1]
        .message
        .contains("task `Third` is blocked by previous open sibling `Second`"));

    let rendered = report.to_compact_text("fixture.org", source);
    assert!(rendered.contains("[ORG029] Warning"));
    assert!(rendered.contains("fix: finish, reorder, or explicitly defer this task"));
    assert!(rendered.contains(
        "Contract: Blocked-state advice must be derived from native local ORDERED sibling evidence"
    ));
    assert!(!rendered.contains("Nested B is blocked"));
}
