use std::{fs, path::PathBuf};

use orgize::lint::{LintOptions, lint_org_with_options};

#[test]
fn lint_reports_file_link_path_issues_with_snapshot() {
    let dir = test_dir("lint-file-link-paths");
    fs::write(dir.join("present.org"), "* Present\n").unwrap();
    fs::create_dir_all(dir.join("directory")).unwrap();

    let source = r#"[[file:present.org]]
[[file:missing.org::*Heading]]
[[file:directory]]
[[file:]]
[[file:/ssh:host:/remote.org]]
"#;
    let report = lint_org_with_options(
        source,
        &LintOptions {
            file_base_dir: Some(dir),
            ..LintOptions::default()
        },
    );

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_text("file-link-issues.org")
    ));
}

#[test]
fn lint_enforces_skill_package_relative_paths() {
    let root = test_dir("lint-skill-package-relative-paths");
    let skills = root.join("skills");
    fs::create_dir_all(&skills).unwrap();

    let source = r#"* ASP Org
[[../templates/README.org][templates README]]
[[../contracts/agent.execplan.v1.org][agent.execplan.v1]]
[[languages/org/templates/README.org][repo-root template]]
#+begin_src shell
asp org contract trace --org-contract-registry <ASP_ORG_ROOT>/contracts/agent.execplan.v1.org templates/agent.execplan.v1.org
#+end_src
"#;

    let report = lint_org_with_options(
        source,
        &LintOptions {
            file_base_dir: Some(skills),
            ..LintOptions::default()
        },
    );

    let findings = report
        .findings
        .iter()
        .filter(|finding| finding.code == "ORG018")
        .collect::<Vec<_>>();
    assert_eq!(findings.len(), 3, "{:#?}", report.findings);
    assert!(
        findings.iter().any(|finding| finding
            .message
            .contains("languages/org/templates/README.org")),
        "{findings:#?}"
    );
    assert!(
        findings.iter().any(|finding| finding
            .message
            .contains("<ASP_ORG_ROOT>/contracts/agent.execplan.v1.org")),
        "{findings:#?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("templates/agent.execplan.v1.org")),
        "{findings:#?}"
    );
}

#[test]
fn lint_enforces_template_package_relative_paths() {
    let root = test_dir("lint-template-package-relative-paths");
    let templates = root.join("templates");
    fs::create_dir_all(&templates).unwrap();

    let source = r#"#+TITLE: Org Templates
#+PROPERTY: TEMPLATE_INDEX org.templates.v1

* Templates
- Template: [[languages/org/templates/agent.execplan.v1.org][agent.execplan.v1.org]]
- Contract: [[languages/org/contracts/agent.execplan.v1.org][agent.execplan.v1]]
:TEMPLATE_CONTRACT_ORG: [[languages/org/contracts/agent.execplan.v1.org][agent.execplan.v1]]
"#;

    let report = lint_org_with_options(
        source,
        &LintOptions {
            file_base_dir: Some(templates),
            ..LintOptions::default()
        },
    );

    let findings = report
        .findings
        .iter()
        .filter(|finding| finding.code == "ORG018")
        .collect::<Vec<_>>();
    assert_eq!(findings.len(), 3, "{:#?}", report.findings);
    assert!(
        findings.iter().any(|finding| finding
            .message
            .contains("languages/org/templates/agent.execplan.v1.org")),
        "{findings:#?}"
    );
    assert!(
        findings.iter().any(|finding| finding
            .message
            .contains("languages/org/contracts/agent.execplan.v1.org")),
        "{findings:#?}"
    );
}

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("orgize-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}
