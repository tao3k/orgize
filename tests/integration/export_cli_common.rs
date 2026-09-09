use serde_json::Value;

pub(super) fn test_dir(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("orgize-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create test dir");
    root
}

pub(super) fn assert_document_query_evidence(packet: &Value) {
    let snapshot = &packet["sourceSnapshot"];
    assert_eq!(snapshot["schemaId"], "asp.source-snapshot.v1");
    assert_eq!(snapshot["algorithm"], "blake3-merkle-v1");
    assert_eq!(snapshot["sourceKind"], "filesystem");
    assert!(
        snapshot["leafCount"]
            .as_u64()
            .is_some_and(|count| count > 0)
    );
    let root_digest = snapshot["rootDigest"]
        .as_str()
        .expect("snapshot root digest");
    assert_lower_hex(root_digest, 64);
    assert_integrity_ref(
        snapshot["providerDigest"]
            .as_str()
            .expect("provider digest"),
        "blake3:",
    );

    let resolution = &packet["resolutionEvidence"];
    assert_eq!(resolution["schemaId"], "asp.source-resolution.v1");
    assert_eq!(resolution["snapshotRoot"], root_digest);
    assert_eq!(resolution["authority"], "live-parser");
    assert_eq!(resolution["state"], "live-hit");
    assert_eq!(
        resolution["parserArtifactDigest"],
        snapshot["providerDigest"]
    );

    assert_integrity_ref(
        packet["itemDigest"].as_str().expect("query item digest"),
        "blake3:",
    );
    assert_integrity_ref(
        packet["executionCommandDigest"]
            .as_str()
            .expect("execution command digest"),
        "sha256:",
    );
    assert!(
        packet["contentBlocks"]
            .as_array()
            .expect("content blocks")
            .iter()
            .all(|block| block["itemDigest"]
                .as_str()
                .is_some_and(|digest| integrity_ref_is_valid(digest, "blake3:")))
    );
}

pub(super) fn assert_document_selector_query_evidence(packet: &Value, owner_path: &str) {
    assert_document_query_evidence(packet);
    let resolution = &packet["resolutionEvidence"];
    assert_eq!(resolution["ownerPath"], owner_path);
    assert_integrity_ref(
        resolution["ownerBlobDigest"]
            .as_str()
            .expect("owner blob digest"),
        "blake3:",
    );
}

fn assert_integrity_ref(value: &str, prefix: &str) {
    assert!(integrity_ref_is_valid(value, prefix), "{value}");
}

fn integrity_ref_is_valid(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(prefix)
        .is_some_and(|digest| lower_hex_is_valid(digest, 64))
}

fn assert_lower_hex(value: &str, len: usize) {
    assert!(lower_hex_is_valid(value, len), "{value}");
}

fn lower_hex_is_valid(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}
