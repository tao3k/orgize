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

pub(super) fn decode(bytes: &[u8]) -> Result<(u64, u64, u64, &[u8]), String> {
    if bytes.len() < 32 || &bytes[..4] != b"OPR1" || &bytes[28..32] != b"OEV1" {
        return Err("invalid native diagnostic envelope".into());
    }
    let fold = u64::from_le_bytes(bytes[4..12].try_into().unwrap());
    let tape = u64::from_le_bytes(bytes[12..20].try_into().unwrap());
    let cpu = u64::from_le_bytes(bytes[20..28].try_into().unwrap());
    Ok((fold, tape, cpu, &bytes[28..]))
}
