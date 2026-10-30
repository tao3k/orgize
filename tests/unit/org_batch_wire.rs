use super::{MAX_SOURCE_BYTES, decode_batch, encode};
use crate::org_aot::org_language_spec;

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
    let bytes = encode(&["α", "", "β"], 258, 2, false).unwrap();
    assert_eq!(&bytes[..5], b"ONR1\x06");
    assert_eq!(&bytes[5..13], &258_u64.to_le_bytes());
    assert_eq!(&bytes[13..16], &[2, 3, 0]);
    assert_eq!(
        &bytes[16..],
        &[2, 0, 0, 0, 0xce, 0xb1, 0, 0, 0, 0, 2, 0, 0, 0, 0xce, 0xb2]
    );
    assert_eq!(encode(&[""], 15, 2, true).unwrap()[4], 7);
}

#[test]
fn encoder_enforces_count_and_aggregate_byte_boundaries() {
    assert!(encode(&[], 15, 2, false).is_err());
    assert!(encode(&[""; 64], 15, 2, false).is_ok());
    assert!(encode(&[""; 65], 15, 2, false).is_err());
    let limit = "x".repeat(MAX_SOURCE_BYTES);
    assert!(encode(&[&limit], 15, 2, false).is_ok());
    assert!(encode(&[&limit, "x"], 15, 2, false).is_err());
    assert!(encode(&[""], 15, 256, false).is_err());
}

#[test]
fn decoder_preserves_individual_results_and_rejects_every_truncation() {
    let tape = format!("OEV1{}", org_language_spec().grammar_digest).into_bytes();
    let bytes = envelope(&[(0, &tape), (1, b"failed"), (0, &tape)]);
    let rows = decode_batch(&bytes, 3, org_language_spec(), false).unwrap();
    assert!(rows[0].as_ref().unwrap().is_empty());
    assert_eq!(rows[1].as_ref().unwrap_err(), "failed");
    assert!(rows[2].as_ref().unwrap().is_empty());
    for end in 0..bytes.len() {
        assert!(decode_batch(&bytes[..end], 3, org_language_spec(), false).is_err());
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(decode_batch(&trailing, 3, org_language_spec(), false).is_err());
    assert!(decode_batch(&bytes, 2, org_language_spec(), false).is_err());
}

#[test]
fn decoder_rejects_unknown_status_foreign_tape_and_invalid_error_utf8() {
    for bytes in [
        envelope(&[(2, b"")]),
        envelope(&[(0, b"OEV1foreign")]),
        envelope(&[(1, &[255])]),
        envelope(&[]),
    ] {
        assert!(decode_batch(&bytes, 1, org_language_spec(), false).is_err());
    }
}

#[cfg(feature = "runtime-profile")]
#[test]
fn batch_diagnostics_are_strict_opt_in_and_preserve_ordered_payloads() {
    let tape = format!("OEV1{}", org_language_spec().grammar_digest).into_bytes();
    let mut row = b"OPR1".to_vec();
    for value in [7_u64, 11, 13] {
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
    let (rows, observations) = crate::runtime_profile::measure(|| {
        decode_batch(&bytes, 3, org_language_spec(), true).unwrap()
    });
    assert!(rows[0].as_ref().unwrap().is_empty());
    assert_eq!(rows[1].as_ref().unwrap_err(), "failed");
    assert!(rows[2].as_ref().unwrap().is_empty());
    for (name, expected) in [
        ("native.batch_body_wall_sum", 17),
        ("native.batch_owner_thread_cpu_sum", 19),
        ("native.batch_process_cpu_interval_sum", 23),
        ("native.batch_vm_gc_cpu_interval_sum", 29),
        ("native.batch_vm_gc_wall_interval_sum", 31),
        ("native.batch_vm_gc_count_interval_sum", 37),
    ] {
        assert_eq!(observations[name], expected);
    }
    assert_eq!(observations["native.scheme_thread_cpu"], 26);
    assert!(decode_batch(&ordinary, 3, org_language_spec(), true).is_err());
    assert!(decode_batch(&bytes, 3, org_language_spec(), false).is_err());
    for end in 0..bytes.len() {
        assert!(decode_batch(&bytes[..end], 3, org_language_spec(), true).is_err());
    }
    let mut invalid = bytes.clone();
    invalid.push(0);
    let (_, observations) = crate::runtime_profile::measure(|| {
        assert!(decode_batch(&invalid, 3, org_language_spec(), true).is_err());
    });
    assert!(!observations.contains_key("native.batch_body_wall_sum"));
    for offset in [0, 52] {
        let mut invalid = bytes.clone();
        invalid[offset] = b'X';
        assert!(decode_batch(&invalid, 3, org_language_spec(), true).is_err());
    }
    assert!(decode_batch(&bytes, 2, org_language_spec(), true).is_err());
    assert!(crate::runtime_profile::measure(|| ()).1.is_empty());
}
