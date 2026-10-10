//! Native ABI source grammar admission; no host grammar fallback.
fn plan(name: &str, fields: &[&str]) -> Vec<Vec<String>> {
    let mut request = vec![name];
    request.extend_from_slice(fields);
    crate::org_aot::native_semantic_rows(23, &request).unwrap()
}
pub(super) fn source_values_preserve_presence_and_inert_syntax() {
    for (raw, expected) in [
        ("　\"路径 with spaces.org\" ", "路径 with spaces.org"),
        (" 'single.org' ", "single.org"),
        ("\"\"", ""),
        ("''", ""),
        ("\"", "\""),
        ("'", "'"),
        ("\"mismatch'", "\"mismatch'"),
        ("\"\"nested\"\"", "\"nested\""),
        ("\" inner \"", " inner "),
        ("　(system \"inert\") ", "(system \"inert\")"),
    ] {
        assert_eq!(plan("source-unquote", &[raw]), [vec![expected]]);
    }
    assert_eq!(
        crate::ast::AgendaDate::parse_ymd("2024-02-29"),
        Some(crate::ast::AgendaDate::new(2024, 2, 29))
    );
    assert_eq!(crate::ast::AgendaDate::parse_ymd("2025-02-29"), None);
    let parent = crate::ast::SddParentRef::parse("[[id:parent][Label]]").unwrap();
    assert_eq!(parent.target_id.as_deref(), Some("parent"));
    assert_eq!(parent.label.as_deref(), Some("Label"));
    assert_eq!(
        plan("workspace-link-key", &["id:parent::search"]),
        [vec!["id:parent"]]
    );
    assert!(plan("workspace-link-key", &["https://external"]).is_empty());
    assert_eq!(plan("interactive-choice", &[""])[0][0], "error");
    let choice = plan(
        "interactive-choice",
        &[
            "id: one\nmethod: choice\nstage: review\ninfo: select\ncategories: 1=one,?=detail\ndetails:\n| 1 | one | - | Full | Always |\n",
        ],
    );
    assert_eq!(
        choice[0],
        vec!["choice", "one", "choice", "review", "", "", "", "select"]
    );
    assert_eq!(choice[3], vec!["entry", "1", "one", "", "Full", "Always"]);
    assert_eq!(
        plan("datetree-title", &["day", "2026-99-00 title"]),
        [vec!["2026", "99", "0"]]
    );
    assert_eq!(
        plan("timestamp-sort-key", &["日期 <2026-10-06 Tue 09:30>"]),
        [vec!["2026", "10", "6", "9", "30"]]
    );
    assert_eq!(plan("clock-scope", &["tree+4"]), [vec!["tree-level"]]);
    assert_eq!(
        plan("column-summary-kind", &["X%"]),
        [vec!["checkbox-percent"]]
    );
    assert_eq!(plan("column-precision", &[" %.2f "]), [vec!["2"]]);
    assert_eq!(plan("checkbox-done", &["[0/0]"]), [vec!["false"]]);
    assert_eq!(
        plan("property-schema-reference", &["[[file:a.org][Label]]"]),
        [vec!["[[file:a.org][Label]]", "file:a.org", "org-file-link"]]
    );
    let query = crate::ast::AgendaMatchQuery::parse("+work|名称<=2").unwrap();
    assert_eq!(query.expression(), "+work|名称<=2");
    let error = crate::ast::AgendaMatchQuery::parse("名称=").unwrap_err();
    assert_eq!(error.position, 7);
    assert_eq!(error.message, "property match is missing a value");
    assert!(crate::ast::AgendaMatchQuery::parse("a||b").is_err());
    assert!(crate::ast::AgendaMatchQuery::parse("ok").is_ok());
    assert_eq!(
        plan("header-policy", &["noweb", "yes strip-tangle"]),
        [vec!["strip"]]
    );
    assert_eq!(
        plan("header-policy", &["result", "not-a-result"]),
        [vec!["other"]]
    );
    assert_eq!(
        plan("header-language", &["HEADER-ARGS:Scheme", "scheme"]),
        [vec!["true"]]
    );
    assert_eq!(plan("alignment-cookie", &[" <r> "]), [vec!["true"]]);
    assert_eq!(plan("include-lines", &["2-9"]), [vec!["range", "2", "9"]]);
    assert_eq!(plan("include-mode", &[]), [vec!["org"]]);
    assert_eq!(
        plan("source-variable", &["x=foo(a=1)"]),
        [vec!["x", "foo(a=1)", "true"]]
    );
    assert_eq!(
        plan("noweb-references", &["<<one>> <<two(x=1)>>"]),
        [vec!["one", "two"]]
    );
    assert_eq!(
        plan("var-target", &["source(x=[1,2])"]),
        [vec!["source", "call"]]
    );
    for literal in ["", "NaN", "1e2", "(system \"inert\")", "[1]"] {
        assert!(plan("var-target", &[literal]).is_empty());
    }
    assert_eq!(
        plan("radio-header", &["SEND target"]),
        [vec!["target", "", "false"]]
    );
    assert!(plan("radio-options", &["SEND target"]).is_empty());
    assert_eq!(
        plan("column-list", &["(1, 2 0 bad 3)"]),
        [vec!["1", "2", "3"]]
    );
    assert_eq!(
        plan("contract-reference", &["[[file:./a.org#id][Label]]"]),
        [vec![
            "[[file:./a.org#id][Label]]",
            "a.org",
            "true",
            "id",
            "true"
        ]]
    );
    let invalid = ["radio-marker", "only one argument"];
    assert!(crate::org_aot::native_semantic_rows(23, &invalid).is_err());
    assert_eq!(plan("severity", &["WARN"]), [vec!["warning"]]);
    assert_eq!(
        plan("contract-policy", &["scope", " SUBTREE "]),
        [vec!["subtree"]]
    );
    assert_eq!(
        plan("contract-policy", &["kind", "ORG-ELEMENTS"]),
        [vec!["other"]]
    );
    assert_eq!(
        plan(
            "contract-policy",
            &["language", " Org-Elements-Query-Expr "]
        ),
        [vec!["query"]]
    );
    assert_eq!(
        plan("contract-policy", &["named-id", " id.message "]),
        [vec![""]]
    );
    assert_eq!(
        plan("contract-policy", &["named-id", " id.MESSAGE "]),
        [vec!["id.MESSAGE"]]
    );
    assert_eq!(
        plan(
            "contract-qualified-link",
            &["[[file:a.org#id]]", "a.org", "id"]
        ),
        [vec!["true"]]
    );
    assert_eq!(
        plan("contract-qualified-link", &["[[file:a.org]]", "a.org", ""]),
        [vec!["false"]]
    );
}
