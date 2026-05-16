use orgize::lint::lint_org;

#[test]
fn lint_reports_progress_statistics_cookie_issues() {
    let source = r#"* TODO Recursive todo [1/2] [99%]
:PROPERTIES:
:COOKIE_DATA: todo recursive
:END:
** DONE Done child
** TODO Open child
*** DONE Nested done child
* TODO Ambiguous [0/0]
- [ ] checkbox evidence
** TODO todo evidence
* TODO Checkbox [0/2]
:PROPERTIES:
:COOKIE_DATA: checkbox recursive
:END:
- [X] done item
- [ ] open item
"#;
    let report = lint_org(source);
    let findings = report.findings;

    assert_eq!(findings.len(), 4);
    assert_eq!(findings[0].code, "ORG028");
    assert!(findings[0].message.contains("expected `[2/3]`"));
    assert_eq!(findings[1].code, "ORG028");
    assert!(findings[1].message.contains("expected `[66%]`"));
    assert_eq!(findings[2].code, "ORG027");
    assert!(findings[2].message.contains("COOKIE_DATA"));
    assert_eq!(findings[3].code, "ORG028");
    assert!(findings[3].message.contains("expected `[1/2]`"));
}

#[test]
fn lint_reports_list_item_checkbox_statistics_cookie_issues() {
    let source = r#"* TODO Lists
- Parent [0/3]
  - [X] done item
  - [ ] open item
  - [-] partial item
- Percent [99%]
  - [X] done item
  - [ ] open item
- Fresh [1/2]
  - [X] done item
  - [ ] open item
* TODO Recursive list
:PROPERTIES:
:COOKIE_DATA: checkbox recursive
:END:
- Recursive [1/2]
  - [X] done item
  - [ ] branch
    - [X] nested done item
* TODO Todo cookie data
:PROPERTIES:
:COOKIE_DATA: todo
:END:
- Ignored [0/0]
  - [X] child ignored by checkbox updater
"#;
    let report = lint_org(source);
    let findings = report.findings;

    assert_eq!(findings.len(), 3);
    assert_eq!(findings[0].code, "ORG028");
    assert!(findings[0].message.contains("expected `[1/3]`"));
    assert_eq!(findings[1].code, "ORG028");
    assert!(findings[1].message.contains("expected `[50%]`"));
    assert_eq!(findings[2].code, "ORG028");
    assert!(findings[2].message.contains("expected `[2/3]`"));
}
