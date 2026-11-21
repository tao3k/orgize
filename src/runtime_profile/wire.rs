//! Diagnostic wire admission only; no parser or timing fallback.
pub(crate) fn decode_batch(bytes: &[u8]) -> Result<([u64; 6], &[u8]), String> {
    if bytes.len() < 58 || &bytes[..4] != b"OBP1" || &bytes[52..56] != b"OBT1" {
        return Err("invalid native batch diagnostic envelope".into());
    }
    let fields = std::array::from_fn(|index| {
        let start = 4 + 8 * index;
        u64::from_le_bytes(bytes[start..start + 8].try_into().unwrap())
    });
    Ok((fields, &bytes[52..]))
}

pub(super) fn decode(bytes: &[u8]) -> Result<([u64; 4], &[u8]), String> {
    if bytes.len() < 40 || &bytes[..4] != b"OPR2" || &bytes[36..40] != b"ODV1" {
        return Err("invalid native diagnostic envelope".into());
    }
    let fields = std::array::from_fn(|index| {
        let start = 4 + 8 * index;
        u64::from_le_bytes(bytes[start..start + 8].try_into().unwrap())
    });
    Ok((fields, &bytes[36..]))
}
