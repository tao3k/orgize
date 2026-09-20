use std::fs;

use super::WorkspaceFixture;

#[test]
fn workspace_contract_rejects_sentinel_mixed_with_identity_reference() {
    let fixture = WorkspaceFixture::new();
    fixture.install_reference_policy();
    fixture.add_reference_fixture("P-001", "none");

    for language in ["cn", "en"] {
        let path = fixture.root.join(language).join("docs/doc.org");
        let source = fs::read_to_string(&path).unwrap();
        fs::write(
            &path,
            source.replace(":REFINES: none", ":REFINES: none P-001"),
        )
        .unwrap();
    }

    fixture.assert_failure(
        "property REFINES must not mix allowed sentinel values with identity references",
    );
}
