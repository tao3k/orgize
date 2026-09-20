use std::fs;

use super::{elements::DocumentSource, model::DocumentLanguage, packets::document_query_evidence};

#[test]
fn evidence_hashes_the_same_bytes_given_to_the_parser() {
    let root = std::env::temp_dir().join(format!(
        "orgize-snapshot-source-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    fs::create_dir_all(&root).expect("create snapshot fixture");
    let path = root.join("note.org");
    fs::write(&path, "changed after source admission\n").expect("write changed source");
    let admitted = "* Admitted bytes\n";
    let sources = [DocumentSource {
        path: path.clone(),
        source: admitted.to_string(),
    }];

    let evidence = document_query_evidence(
        DocumentLanguage::Org,
        &sources,
        Some(&path),
        &root,
        &["query".to_string(), "--json".to_string()],
    )
    .expect("build evidence from admitted bytes");

    assert_eq!(
        evidence.resolution_evidence["ownerBlobDigest"],
        format!("blake3:{}", blake3::hash(admitted.as_bytes()).to_hex())
    );
    let _ = fs::remove_dir_all(root);
}
