use std::path::Path;

use crate::ast::{parse_contract_reference, parse_contract_reference_from_source};

#[test]
fn resolves_a_relative_file_link_from_the_owning_org_document() {
    let reference = parse_contract_reference_from_source(
        "[[file:../contracts/90.01_document_contract.org][tao3k.document]]",
        Some(Path::new("docs/brand/10.01_positioning.org")),
    );

    assert_eq!(
        reference.path.as_deref(),
        Some("docs/contracts/90.01_document_contract.org")
    );
    assert_eq!(reference.contract_id.as_deref(), Some("tao3k.document"));
    assert!(reference.is_path_qualified_org_link());
}

#[test]
fn accepts_a_path_and_fragment_contract_link() {
    let reference = parse_contract_reference(
        "[[file:docs/contracts/90.01_document_contract.org#tao3k.document][Documentation contract]]",
    );

    assert_eq!(
        reference.path.as_deref(),
        Some("docs/contracts/90.01_document_contract.org")
    );
    assert_eq!(reference.contract_id.as_deref(), Some("tao3k.document"));
    assert!(reference.is_path_qualified_org_link());
}

#[test]
fn rejects_a_bare_id_as_a_path_qualified_reference() {
    let reference = parse_contract_reference("tao3k.document");

    assert!(!reference.is_path_qualified_org_link());
}
