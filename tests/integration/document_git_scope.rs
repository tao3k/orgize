use std::{fs, path::PathBuf};

use orgize::document::{DocumentLanguage, DocumentWalkConfig, index_project_with_config};

#[test]
fn document_owner_admission_uses_git_scope_and_parser_structural_identity() {
    let root = test_dir("document-git-scope");
    fs::create_dir(root.join(".git")).expect("create Git scope marker");
    fs::write(root.join(".gitignore"), "ignored.org\n").expect("write Git ignore contract");
    fs::write(root.join("visible.org"), "* Visible owner\n").expect("write visible Org owner");
    fs::write(root.join("ignored.org"), "* Ignored owner\n").expect("write ignored Org owner");

    let elements =
        index_project_with_config(DocumentLanguage::Org, &root, &DocumentWalkConfig::default())
            .expect("index Git-scoped Org documents");

    assert!(
        elements
            .iter()
            .any(|element| element.path.ends_with("visible.org")),
        "visible Git-scoped Org owner was omitted"
    );
    assert!(
        elements
            .iter()
            .all(|element| !element.path.ends_with("ignored.org")),
        "Git-ignored Org owner leaked into document evidence"
    );
    for element in elements {
        assert!(
            element.structural_selector.starts_with("org://"),
            "document element lacks parser-owned structural selector: {}",
            element.structural_selector
        );
        assert!(
            element.structural_selector.contains('#'),
            "document selector lacks structural fragment: {}",
            element.structural_selector
        );
        assert!(
            !element
                .structural_selector
                .ends_with(&format!(":{}", element.line)),
            "display line anchor leaked into structural identity: {}",
            element.structural_selector
        );
    }
}

fn test_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("orgize-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create document Git scope fixture");
    root
}
