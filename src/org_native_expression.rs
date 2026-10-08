//! Decode admitted Scheme expression values, never source syntax.

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NativeExpressionValue {
    Atom(String),
    String(String),
    List(Vec<Self>),
}

pub(super) fn decode(mut bytes: &[u8], digest: &str) -> Result<Vec<NativeExpressionValue>, String> {
    bytes = bytes
        .strip_prefix(b"OXV1")
        .and_then(|body| body.strip_prefix(digest.as_bytes()))
        .ok_or("expression tape identity mismatch")?;
    let mut stack = vec![Vec::new()];
    while let Some((&tag, rest)) = bytes.split_first() {
        bytes = rest;
        match tag {
            1 => stack.push(Vec::new()),
            2 => {
                if stack.len() == 1 {
                    return Err("unexpected expression list finish".into());
                }
                let value = NativeExpressionValue::List(stack.pop().unwrap());
                stack.last_mut().unwrap().push(value);
            }
            3 | 4 => {
                let length = bytes.get(..8).ok_or("truncated expression length")?;
                let length = usize::try_from(u64::from_le_bytes(length.try_into().unwrap()))
                    .map_err(|_| "expression length overflow")?;
                bytes = &bytes[8..];
                let value =
                    std::str::from_utf8(bytes.get(..length).ok_or("truncated expression value")?)
                        .map_err(|_| "invalid expression UTF-8")?
                        .to_owned();
                bytes = &bytes[length..];
                stack.last_mut().unwrap().push(if tag == 3 {
                    NativeExpressionValue::Atom(value)
                } else {
                    NativeExpressionValue::String(value)
                });
            }
            _ => return Err("unknown expression tape tag".into()),
        }
    }
    if stack.len() != 1 {
        return Err("unfinished expression list".into());
    }
    Ok(stack.pop().unwrap())
}

#[cfg(test)]
#[path = "../tests/unit/org_native_expression.rs"]
mod tests;
