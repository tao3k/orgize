use orgize::lint::lint_org;

#[test]
fn lint_reports_org_crypt_advice_with_snapshot() {
    let source = r#"* Secret note :crypt:
Visible plaintext body should be encrypted by the editor workflow.
* Key only
:PROPERTIES:
:CRYPTKEY: 0xfeed
:END:
"#;
    let report = lint_org(source);

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_compact_text("crypt.org", source)
    ));
}
