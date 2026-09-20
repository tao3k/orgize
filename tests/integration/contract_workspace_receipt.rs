use std::{fs, process::Command};

#[test]
fn workspace_contract_json_emits_qualification_receipt() {
    let root =
        std::env::temp_dir().join(format!("orgize-workspace-receipt-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("policy.org"), "#+TITLE: Policy\n:PROPERTIES:\n:WORKSPACE_CONTRACT_ID: test.workspace.v1\n:END:\n* Docs\n:PROPERTIES:\n:PATH: docs\n:PATH_KIND: directory\n:ROLE: maintained\n:CONTRACT_ORG_EXACT: test.document.v1\n:END:\n* Policy\n:PROPERTIES:\n:PATH: policy.org\n:PATH_KIND: file\n:ROLE: support\n:END:\n* Registry\n:PROPERTIES:\n:PATH: contracts.org\n:PATH_KIND: file\n:ROLE: support\n:END:\n").unwrap();
    fs::write(root.join("contracts.org"), "* Contract\n:PROPERTIES:\n:CONTRACT_ID: test.document.v1\n:CONTRACT_SCOPE: document\n:CONTRACT_KIND: org-elements\n:END:\n** Title\n:PROPERTIES:\n:ASSERT_ID: test.document.has-title\n:SEVERITY: error\n:END:\n#+begin_src org-contract\n(assert exists (keyword :context \"metadata\" :summary (key \"TITLE\")))\n#+end_src\n").unwrap();
    fs::write(
        root.join("docs/note.org"),
        "#+TITLE: Note\n:PROPERTIES:\n:CONTRACT_ORG: test.document.v1\n:END:\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .args(["contract", "workspace", "--root"])
        .arg(&root)
        .arg("--policy")
        .arg(root.join("policy.org"))
        .arg("--org-contract-registry")
        .arg(root.join("contracts.org"))
        .arg("--summary-json")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(receipt["status"], "passed");
    assert_eq!(receipt["documentCount"], 1);
    assert_eq!(receipt["evaluationCount"], 1);
    assert!(receipt.get("root").is_none());
    assert!(receipt.get("files").is_none());
    assert_eq!(receipt["orgizeRevision"].as_str().unwrap().len(), 40);
    assert!(
        receipt["workspaceDigest"]
            .as_str()
            .unwrap()
            .starts_with("blake3:")
    );
    let relative_output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&root)
        .args([
            "contract",
            "workspace",
            "--root",
            ".",
            "--policy",
            "./policy.org",
            "--org-contract-registry",
            "./contracts.org",
            "--summary-json",
        ])
        .output()
        .unwrap();
    assert!(
        relative_output.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&relative_output.stderr),
        String::from_utf8_lossy(&relative_output.stdout)
    );
    let relative_receipt: serde_json::Value =
        serde_json::from_slice(&relative_output.stdout).unwrap();
    assert_eq!(
        relative_receipt["workspaceDigest"], receipt["workspaceDigest"],
        "equivalent canonical workspace inputs must produce one digest"
    );
    let external_root = root.with_file_name(format!(
        "{}-external",
        root.file_name().unwrap().to_string_lossy()
    ));
    let _ = fs::remove_dir_all(&external_root);
    fs::create_dir_all(&external_root).unwrap();
    let external_registry = external_root.join("contracts.org");
    fs::copy(root.join("contracts.org"), &external_registry).unwrap();
    let external_absolute = workspace_receipt(&root, &external_registry);
    let external_relative = workspace_receipt(
        &root,
        &std::path::PathBuf::from("..")
            .join(external_root.file_name().unwrap())
            .join("contracts.org"),
    );
    assert_eq!(
        external_absolute["workspaceDigest"], external_relative["workspaceDigest"],
        "external registry labels must be canonical"
    );
    fs::remove_dir_all(external_root).unwrap();
    fs::remove_dir_all(root).unwrap();
}

fn workspace_receipt(root: &std::path::Path, registry: &std::path::Path) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(root)
        .args([
            "contract",
            "workspace",
            "--root",
            ".",
            "--policy",
            "policy.org",
        ])
        .arg("--org-contract-registry")
        .arg(registry)
        .arg("--summary-json")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
