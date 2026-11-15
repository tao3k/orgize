//! Bounded native wire projection; document semantics remain Scheme-owned.
use crate::org_product::{DocumentProduct, GraphProjectionSpec, LanguageSpec};

pub(crate) const MAX_DOCUMENTS: usize = 64;
pub(crate) const MAX_SOURCE_BYTES: usize = 64 * 1024;

pub(super) fn encode(
    sources: &[&str],
    level: usize,
    policy: usize,
    profile: bool,
    todo_profile: Option<(&[String], &[String])>,
) -> Result<Vec<u8>, String> {
    if sources.is_empty() || sources.len() > MAX_DOCUMENTS {
        return Err("native Org batch requires 1..=64 documents".into());
    }
    let size = sources.iter().try_fold(0_usize, |total, source| {
        total
            .checked_add(source.len())
            .ok_or("Org batch size overflow")
    })?;
    if size > MAX_SOURCE_BYTES {
        return Err("native Org batch exceeds 64KiB of source".into());
    }
    let mut bytes = Vec::with_capacity(16 + 4 * sources.len() + size);
    bytes.extend_from_slice(b"ONR1");
    bytes.push(match (todo_profile.is_some(), profile) {
        (false, false) => 6,
        (false, true) => 7,
        (true, false) => 27,
        (true, true) => 28,
    });
    bytes.extend_from_slice(
        &u64::try_from(level)
            .map_err(|_| "Org level overflow")?
            .to_le_bytes(),
    );
    bytes.push(u8::try_from(policy).map_err(|_| "Org policy overflow")?);
    bytes.extend_from_slice(&(sources.len() as u16).to_le_bytes());
    if let Some((open, done)) = todo_profile {
        for words in [open, done] {
            bytes.extend_from_slice(
                &u32::try_from(words.len())
                    .map_err(|_| "Ledger TODO profile overflow")?
                    .to_le_bytes(),
            );
            for word in words {
                bytes.extend_from_slice(
                    &u32::try_from(word.len())
                        .map_err(|_| "Ledger TODO word overflow")?
                        .to_le_bytes(),
                );
                bytes.extend_from_slice(word.as_bytes());
            }
        }
    }
    for source in sources {
        bytes.extend_from_slice(&(source.len() as u32).to_le_bytes());
        bytes.extend_from_slice(source.as_bytes());
    }
    Ok(bytes)
}

pub(super) fn decode_batch(
    mut bytes: &[u8],
    sources: &[&str],
    grammar: &LanguageSpec,
    graph: &GraphProjectionSpec,
    ledger: bool,
    profile: bool,
) -> Result<Vec<Result<DocumentProduct, String>>, String> {
    let count = sources.len();
    #[cfg(feature = "runtime-profile")]
    let batch_fields = if profile {
        let (fields, payload) = crate::runtime_profile::decode_native_batch(bytes)?;
        bytes = payload;
        Some(fields)
    } else {
        None
    };
    #[cfg(not(feature = "runtime-profile"))]
    if profile {
        return Err("native batch diagnostics are not enabled".into());
    }
    if bytes.len() < 6
        || &bytes[..4] != b"OBT1"
        || usize::from(u16::from_le_bytes(bytes[4..6].try_into().unwrap())) != count
        || count == 0
        || count > MAX_DOCUMENTS
    {
        return Err("invalid native Org batch identity/count".into());
    }
    bytes = &bytes[6..];
    let mut results = Vec::with_capacity(count);
    for source in sources {
        if bytes.len() < 5 {
            return Err("truncated native Org batch record".into());
        }
        let status = bytes[0];
        let length = usize::try_from(u32::from_le_bytes(bytes[1..5].try_into().unwrap()))
            .map_err(|_| "native Org batch length overflow")?;
        bytes = &bytes[5..];
        let payload = bytes
            .get(..length)
            .ok_or("truncated native Org batch payload")?;
        bytes = &bytes[length..];
        results.push(match status {
            0 => {
                #[cfg(feature = "runtime-profile")]
                let payload = if profile {
                    crate::runtime_profile::native_tape(payload)?
                } else {
                    payload
                };
                #[cfg(not(feature = "runtime-profile"))]
                let _ = profile;
                // A corrupt successful tape rejects the envelope, not just a document.
                Ok(DocumentProduct::decode(
                    payload, source, grammar, graph, ledger,
                )?)
            }
            1 => Err(std::str::from_utf8(payload)
                .map_err(|_| "invalid native Org batch error UTF-8")?
                .to_owned()),
            _ => return Err("unknown native Org batch status".into()),
        });
    }
    if !bytes.is_empty() {
        return Err("trailing native Org batch bytes".into());
    }
    #[cfg(feature = "runtime-profile")]
    if let Some(fields) = batch_fields {
        crate::runtime_profile::record_native_batch(fields);
    }
    Ok(results)
}

#[cfg(test)]
#[path = "../../tests/unit/org_batch_wire.rs"]
mod tests;
