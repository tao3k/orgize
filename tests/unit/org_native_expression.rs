use super::decode;

#[test]
fn expression_tape_rejects_bad_identity_structure_lengths_and_utf8() {
    for payload in [
        &[2][..],
        &[1],
        &[0],
        &[3],
        &[4, 1, 0, 0, 0, 0, 0, 0, 0, 255],
        &[3, 255, 255, 255, 255, 255, 255, 255, 255],
    ] {
        let mut tape = b"OXV1digest".to_vec();
        tape.extend_from_slice(payload);
        assert!(decode(&tape, "digest").is_err());
    }
    assert!(decode(b"OXV1other", "digest").is_err());
    assert_eq!(decode(b"OXV1digest", "digest"), Ok(Vec::new()));
}
