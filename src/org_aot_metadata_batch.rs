//! Bounded metadata transport; Scheme retains each document's semantic owner.
use super::{OrgAotError, PreparedDocument, native_semantic_rows};

pub(super) fn finish<T>(
    prepared: Vec<Result<PreparedDocument, OrgAotError>>,
    single_operation: u8,
    batch_operation: u8,
    shared_fields: &[String],
    admit: impl Fn(PreparedDocument, Vec<Vec<String>>) -> Result<T, OrgAotError>,
) -> Result<Vec<Result<T, OrgAotError>>, OrgAotError> {
    let mut output = Vec::with_capacity(prepared.len());
    let mut pending = Vec::new();
    let shared_bytes = shared_fields
        .iter()
        .map(|field| field.len() + 4)
        .sum::<usize>();
    let mut bytes = shared_bytes;
    for document in prepared {
        match document {
            Err(error) => {
                flush(
                    &mut pending,
                    &mut output,
                    batch_operation,
                    shared_fields,
                    &admit,
                )?;
                bytes = shared_bytes;
                output.push(Err(error));
            }
            Ok(document) => {
                let size = document
                    .fields
                    .iter()
                    .map(|field| field.len() + 4)
                    .sum::<usize>()
                    + 16;
                if bytes + size > super::BATCH_MAX_SOURCE_BYTES {
                    flush(
                        &mut pending,
                        &mut output,
                        batch_operation,
                        shared_fields,
                        &admit,
                    )?;
                    bytes = shared_bytes;
                }
                if shared_bytes + size > super::BATCH_MAX_SOURCE_BYTES {
                    // Preserve the existing single-document configuration surface.
                    let refs = shared_fields
                        .iter()
                        .chain(&document.fields)
                        .map(String::as_str)
                        .collect::<Vec<_>>();
                    output.push(
                        native_semantic_rows(single_operation, &refs)
                            .map_err(OrgAotError::Native)
                            .and_then(|rows| admit(document, rows)),
                    );
                } else {
                    bytes += size;
                    pending.push(document);
                }
            }
        }
    }
    flush(
        &mut pending,
        &mut output,
        batch_operation,
        shared_fields,
        &admit,
    )?;
    Ok(output)
}

fn flush<T>(
    pending: &mut Vec<PreparedDocument>,
    output: &mut Vec<Result<T, OrgAotError>>,
    operation: u8,
    shared_fields: &[String],
    admit: &impl Fn(PreparedDocument, Vec<Vec<String>>) -> Result<T, OrgAotError>,
) -> Result<(), OrgAotError> {
    if pending.is_empty() {
        return Ok(());
    }
    let mut rows = {
        let count = pending.len().to_string();
        let sizes: Vec<_> = pending
            .iter()
            .map(|doc| doc.fields.len().to_string())
            .collect();
        let field_count = 1
            + shared_fields.len()
            + pending
                .iter()
                .map(|doc| 1 + doc.fields.len())
                .sum::<usize>();
        let mut refs = Vec::with_capacity(field_count);
        refs.extend(shared_fields.iter().map(String::as_str));
        refs.push(count.as_str());
        for (document, size) in pending.iter().zip(&sizes) {
            refs.push(size.as_str());
            refs.extend(document.fields.iter().map(String::as_str));
        }
        native_semantic_rows(operation, &refs)
            .map_err(OrgAotError::Native)?
            .into_iter()
    };
    for document in pending.drain(..) {
        let frame = rows
            .next()
            .ok_or_else(|| OrgAotError::Native("missing metadata frame".into()))?;
        match frame.as_slice() {
            [status, count] if status == "ok" => {
                let count = count
                    .parse::<usize>()
                    .map_err(|_| OrgAotError::Native("invalid metadata count".into()))?;
                let body = rows.by_ref().take(count).collect::<Vec<_>>();
                if body.len() != count {
                    return Err(OrgAotError::Native("truncated metadata frame".into()));
                }
                output.push(admit(document, body));
            }
            [status, error] if status == "error" => {
                output.push(Err(OrgAotError::Native(error.clone())))
            }
            _ => return Err(OrgAotError::Native("invalid metadata frame".into())),
        }
    }
    if rows.next().is_some() {
        return Err(OrgAotError::Native("trailing metadata rows".into()));
    }
    Ok(())
}
