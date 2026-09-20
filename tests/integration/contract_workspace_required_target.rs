use std::fs;

use super::{WorkspaceFixture, receipt};

#[test]
fn workspace_contract_requires_trace_target_to_be_maintained() {
    let fixture = WorkspaceFixture::new();
    let admitted = fixture.run_requiring(&fixture.root.join("cn/docs/doc.org"));
    assert!(admitted.status.success(), "{}", receipt(&admitted));

    let non_org = fixture.root.join("cn/docs/notes.txt");
    fs::write(&non_org, "not an Org document").unwrap();
    let rejected_non_org = fixture.run_requiring(&non_org);
    assert!(
        !rejected_non_org.status.success(),
        "non-Org target must be rejected"
    );
    assert!(
        String::from_utf8_lossy(&rejected_non_org.stderr)
            .contains("must be an admitted Org document"),
        "{}",
        receipt(&rejected_non_org)
    );

    let rejected = fixture.run_requiring(&fixture.root.join("contracts.org"));
    assert!(
        !rejected.status.success(),
        "support target must be rejected"
    );
    assert!(
        String::from_utf8_lossy(&rejected.stderr)
            .contains("must match exactly one maintained workspace route"),
        "{}",
        receipt(&rejected)
    );
}
