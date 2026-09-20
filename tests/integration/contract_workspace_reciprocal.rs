use std::fs;

use super::contract_workspace::WorkspaceFixture;

#[test]
fn workspace_contract_rejects_missing_or_duplicate_reciprocal_source_identities() {
    let expected = "node property SUPERSEDES with reciprocal constraint requires exactly one nonempty PRINCIPLE_ID source identity";
    for replacement in ["", ":PRINCIPLE_ID: P-002\n:PRINCIPLE_ID: P-003\n"] {
        let fixture = WorkspaceFixture::new();
        fixture.install_reference_policy();
        fixture.add_reciprocal_fixture();
        for language in ["cn", "en"] {
            let path = fixture.root.join(language).join("docs/doc.org");
            fs::write(
                &path,
                fs::read_to_string(&path)
                    .unwrap()
                    .replace(":PRINCIPLE_ID: P-002\n", replacement),
            )
            .unwrap();
        }

        let output = fixture.run();
        assert!(
            !output.status.success(),
            "workspace admission must fail for replacement {replacement:?}"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected),
            "replacement {replacement:?}: {}",
            String::from_utf8_lossy(&output.stderr),
        );
    }
}
