//! Bounded metadata transport; Scheme retains each document's semantic owner.
use super::{
    OrgAotDocument, OrgAotError, ParseConfig, PreparedDocument, finish_document,
    native_semantic_rows,
};

pub(super) fn finish(
    prepared: Vec<Result<PreparedDocument, OrgAotError>>,
    config: &ParseConfig,
) -> Result<Vec<Result<OrgAotDocument, OrgAotError>>, OrgAotError> {
    let mut output = Vec::with_capacity(prepared.len());
    let mut pending = Vec::new();
    let mut bytes = 0;
    for document in prepared {
        match document {
            Err(error) => {
                flush(&mut pending, &mut output, config)?;
                bytes = 0;
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
                    flush(&mut pending, &mut output, config)?;
                    bytes = 0;
                }
                if size > super::BATCH_MAX_SOURCE_BYTES {
                    // Preserve the existing single-document configuration surface.
                    let refs = document
                        .fields
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>();
                    output.push(
                        native_semantic_rows(17, &refs)
                            .map_err(OrgAotError::Native)
                            .and_then(|rows| finish_document(document, rows, config)),
                    );
                } else {
                    bytes += size;
                    pending.push(document);
                }
            }
        }
    }
    flush(&mut pending, &mut output, config)?;
    Ok(output)
}

fn flush(
    pending: &mut Vec<PreparedDocument>,
    output: &mut Vec<Result<OrgAotDocument, OrgAotError>>,
    config: &ParseConfig,
) -> Result<(), OrgAotError> {
    if pending.is_empty() {
        return Ok(());
    }
    let mut fields = vec![pending.len().to_string()];
    for document in pending.iter() {
        fields.push(document.fields.len().to_string());
        fields.extend(document.fields.iter().cloned());
    }
    let refs = fields.iter().map(String::as_str).collect::<Vec<_>>();
    let mut rows = native_semantic_rows(24, &refs)
        .map_err(OrgAotError::Native)?
        .into_iter();
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
                output.push(finish_document(document, body, config));
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
