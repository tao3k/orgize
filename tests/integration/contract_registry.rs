use std::fs;

#[test]
fn cli_trace_loads_dependencies_from_registry_only_aggregator() {
    let dir = test_dir("contract-trace-registry-only-aggregator");
    fs::create_dir_all(dir.join("contracts")).unwrap();
    fs::write(
        dir.join("contracts/basic.org"),
        r#"* basic-template-v1
:PROPERTIES:
:CONTRACT_ID: basic.template.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** has-heading
:PROPERTIES:
:ASSERT_ID: basic-template.has-heading
:SEVERITY: error
:END:
#+BEGIN_SRC org-contract
(assert exists (headline))
#+END_SRC
"#,
    )
    .unwrap();
    fs::write(
        dir.join("contracts/registry.org"),
        ":PROPERTIES:\n:CONTRACT_ORG: [[./basic.org][basic.template.v1]]\n:END:\n",
    )
    .unwrap();
    fs::write(
        dir.join("target.org"),
        ":PROPERTIES:\n:CONTRACT_ORG: basic.template.v1\n:END:\n* Target\n",
    )
    .unwrap();

    let output = crate::library_cli::orgize_cli_command()
        .current_dir(&dir)
        .args([
            "contract",
            "trace",
            "--org-contract-registry",
            "contracts/registry.org",
            "target.org",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("basic.template.v1"));
}

fn test_dir(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("orgize-{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    root
}
