use super::{MAX_SOURCE_BYTES, encode};
use crate::org_aot::{org_graph_spec, org_language_spec};
use crate::org_product::DocumentProduct;

fn decode_batch(
    bytes: &[u8],
    count: usize,
    profile: bool,
) -> Result<Vec<Result<DocumentProduct, String>>, String> {
    super::decode_batch(
        bytes,
        &vec![""; count],
        org_language_spec(),
        org_graph_spec(),
        false,
        profile,
    )
}

fn empty_document_product() -> Vec<u8> {
    let mut bytes = format!(
        "ODV1{}{}",
        org_language_spec().grammar_digest,
        org_graph_spec().projection_digest
    )
    .into_bytes();
    let counts = [1, 1];
    let navigation = [u32::from(org_language_spec().root_kind), u32::MAX, 0, 0, 1];
    let element = [0, u32::MAX, 0, 0, 1, 1];
    let source_text_field = [0, 1, 0, 0];
    for word in counts
        .into_iter()
        .chain(navigation)
        .chain(element)
        .chain(source_text_field)
    {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    bytes
}

fn envelope(rows: &[(u8, &[u8])]) -> Vec<u8> {
    let mut bytes = b"OBT1".to_vec();
    bytes.extend_from_slice(&(rows.len() as u16).to_le_bytes());
    for (status, payload) in rows {
        bytes.push(*status);
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(payload);
    }
    bytes
}

#[test]
fn bounded_encoder_preserves_utf8_lengths_order_and_config() {
    let bytes = encode(&["α", "", "β"], 258, 2, false, None).unwrap();
    assert_eq!(&bytes[..5], b"ONR1\x06");
    assert_eq!(&bytes[5..13], &258_u64.to_le_bytes());
    assert_eq!(&bytes[13..16], &[2, 3, 0]);
    assert_eq!(
        &bytes[16..],
        &[2, 0, 0, 0, 0xce, 0xb1, 0, 0, 0, 0, 2, 0, 0, 0, 0xce, 0xb2]
    );
    assert_eq!(encode(&[""], 15, 2, true, None).unwrap()[4], 7);
    let open = ["λ".to_owned()];
    let profile = Some((open.as_slice(), &[][..]));
    let ledger = encode(&["β"], 15, 2, false, profile).unwrap();
    assert_eq!(ledger[4], 27);
    assert_eq!(
        &ledger[16..],
        &[
            1, 0, 0, 0, 2, 0, 0, 0, 0xce, 0xbb, 0, 0, 0, 0, 2, 0, 0, 0, 0xce, 0xb2
        ]
    );
    assert_eq!(encode(&[""], 15, 2, true, profile).unwrap()[4], 28);
}

#[test]
fn encoder_enforces_count_and_aggregate_byte_boundaries() {
    assert!(encode(&[], 15, 2, false, None).is_err());
    assert!(encode(&[""; 64], 15, 2, false, None).is_ok());
    assert!(encode(&[""; 65], 15, 2, false, None).is_err());
    let limit = "x".repeat(MAX_SOURCE_BYTES);
    assert!(encode(&[&limit], 15, 2, false, None).is_ok());
    assert!(encode(&[&limit, "x"], 15, 2, false, None).is_err());
    assert!(encode(&[""], 15, 256, false, None).is_err());
}

#[test]
fn decoder_preserves_individual_results_and_rejects_every_truncation() {
    let tape = empty_document_product();
    let bytes = envelope(&[(0, &tape), (1, b"failed"), (0, &tape)]);
    let rows = decode_batch(&bytes, 3, false).unwrap();
    assert!(
        rows[0]
            .as_ref()
            .unwrap()
            .syntax
            .as_ref()
            .unwrap()
            .text()
            .is_empty()
    );
    assert_eq!(rows[1].as_ref().unwrap_err(), "failed");
    assert!(rows[2].as_ref().unwrap().records[0].field("source-text") == Some(""));
    for end in 0..bytes.len() {
        assert!(decode_batch(&bytes[..end], 3, false).is_err());
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(decode_batch(&trailing, 3, false).is_err());
    assert!(decode_batch(&bytes, 2, false).is_err());
}

#[test]
fn decoder_rejects_unknown_status_foreign_tape_and_invalid_error_utf8() {
    for bytes in [
        envelope(&[(2, b"")]),
        envelope(&[(0, b"OEV1foreign")]),
        envelope(&[(1, &[255])]),
        envelope(&[]),
    ] {
        assert!(decode_batch(&bytes, 1, false).is_err());
    }
}

#[test]
fn decoder_rejects_invalid_navigation_and_element_columns() {
    let header =
        4 + org_language_spec().grammar_digest.len() + org_graph_spec().projection_digest.len();
    let navigation = header + 8;
    let element = navigation + 20;
    let field = element + 24;
    for (offset, value) in [
        (header, 0_u32),
        (navigation, u32::MAX),
        (navigation + 4, 0),
        (navigation + 12, 1),
        (navigation + 16, 0),
        (element, u32::MAX),
        (element + 16, 2),
        (field, u32::MAX),
        (field + 4, 0),
    ] {
        let mut product = empty_document_product();
        product[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(
            decode_batch(&envelope(&[(0, &product)]), 1, false).is_err(),
            "column {offset}"
        );
    }
}

#[cfg(feature = "runtime-profile")]
#[test]
fn batch_diagnostics_are_strict_opt_in_and_preserve_ordered_payloads() {
    let tape = empty_document_product();
    let mut row = b"OPR2".to_vec();
    for value in [7_u64, 11, 13, 17] {
        row.extend_from_slice(&value.to_le_bytes());
    }
    row.extend_from_slice(&tape);
    let ordinary = envelope(&[(0, &row), (1, b"failed"), (0, &row)]);
    let fields = [17_u64, 19, 23, 29, 31, 37];
    let mut bytes = b"OBP1".to_vec();
    for field in fields {
        bytes.extend_from_slice(&field.to_le_bytes());
    }
    bytes.extend_from_slice(&ordinary);
    let (rows, observations) =
        crate::runtime_profile::measure(|| decode_batch(&bytes, 3, true).unwrap());
    assert!(
        rows[0]
            .as_ref()
            .unwrap()
            .syntax
            .as_ref()
            .unwrap()
            .text()
            .is_empty()
    );
    assert_eq!(rows[1].as_ref().unwrap_err(), "failed");
    assert!(rows[2].as_ref().unwrap().records[0].field("source-text") == Some(""));
    for (name, expected) in [
        ("fold.batch_body_wall_sum", 17),
        ("fold.batch_owner_thread_cpu_sum", 19),
        ("fold.batch_process_cpu_interval_sum", 23),
        ("fold.batch_vm_gc_cpu_interval_sum", 29),
        ("fold.batch_vm_gc_wall_interval_sum", 31),
        ("fold.batch_vm_gc_count_interval_sum", 37),
    ] {
        assert_eq!(observations[name], expected);
    }
    assert_eq!(observations["fold.thread_cpu"], 26);
    assert_eq!(observations["product.encode_thread_cpu"], 34);
    assert!(decode_batch(&ordinary, 3, true).is_err());
    assert!(decode_batch(&bytes, 3, false).is_err());
    for end in 0..bytes.len() {
        assert!(decode_batch(&bytes[..end], 3, true).is_err());
    }
    let mut invalid = bytes.clone();
    invalid.push(0);
    let (_, observations) = crate::runtime_profile::measure(|| {
        assert!(decode_batch(&invalid, 3, true).is_err());
    });
    assert!(!observations.contains_key("fold.batch_body_wall_sum"));
    for offset in [0, 52] {
        let mut invalid = bytes.clone();
        invalid[offset] = b'X';
        assert!(decode_batch(&invalid, 3, true).is_err());
    }
    assert!(decode_batch(&bytes, 2, true).is_err());
    assert!(crate::runtime_profile::measure(|| ()).1.is_empty());
}
