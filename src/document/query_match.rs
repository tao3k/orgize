//! Mechanical predicates over already admitted document facts, not Org syntax.
use super::model::DocumentElement;

pub(super) struct FactView<'a> {
    pub kind: &'a str,
    pub source_kind: &'a str,
    pub path: &'a str,
    pub text: &'a str,
    pub content: &'a str,
    pub fields: &'a [(String, String)],
}

struct FieldPredicate {
    key: String,
    value: Option<String>,
}

struct PreparedTerm {
    bytes: Vec<u8>,
    prefixes: Vec<usize>,
}

impl PreparedTerm {
    fn new(term: &str) -> Self {
        let bytes = term.to_ascii_lowercase().into_bytes();
        let mut prefixes = vec![0; bytes.len()];
        for index in 1..bytes.len() {
            let mut matched = prefixes[index - 1];
            while matched > 0 && bytes[index] != bytes[matched] {
                matched = prefixes[matched - 1];
            }
            if bytes[index] == bytes[matched] {
                matched += 1;
            }
            prefixes[index] = matched;
        }
        Self { bytes, prefixes }
    }
}

pub(super) struct PreparedQuery {
    terms: Vec<PreparedTerm>,
    kinds: Vec<String>,
    fields: Vec<FieldPredicate>,
}

impl PreparedQuery {
    pub fn new(terms: &[String], kinds: &[String], fields: &[String]) -> Self {
        Self {
            terms: terms
                .iter()
                .flat_map(|term| term.split_whitespace())
                .map(PreparedTerm::new)
                .collect(),
            kinds: kinds.iter().map(|kind| kind.trim().to_owned()).collect(),
            fields: fields
                .iter()
                .map(|field| field.trim())
                .filter(|field| !field.is_empty())
                .map(|field| match field.split_once('=') {
                    Some((key, value)) => FieldPredicate {
                        key: key.trim().to_owned(),
                        value: Some(value.trim().to_owned()),
                    },
                    None => FieldPredicate {
                        key: field.to_owned(),
                        value: None,
                    },
                })
                .collect(),
        }
    }

    pub fn matches(&self, element: &DocumentElement) -> bool {
        self.matches_view(FactView {
            kind: element.kind,
            source_kind: element.source_kind,
            path: &element.path,
            text: &element.text,
            content: &element.content,
            fields: &element.fields,
        })
    }

    pub fn matches_view(&self, fact: FactView<'_>) -> bool {
        self.kinds
            .iter()
            .all(|kind| fact.kind.eq_ignore_ascii_case(kind))
            && self.fields.iter().all(|predicate| match &predicate.value {
                Some(value) if predicate.key.eq_ignore_ascii_case("text") => {
                    fact.text.contains(value)
                }
                value => fact.fields.iter().any(|(key, actual)| {
                    key.eq_ignore_ascii_case(&predicate.key)
                        && value.as_ref().is_none_or(|value| actual.contains(value))
                }),
            })
            && self.terms.iter().all(|term| {
                [
                    fact.kind,
                    fact.source_kind,
                    fact.path,
                    fact.text,
                    fact.content,
                ]
                .into_iter()
                .any(|value| contains_ascii_folded(value, term))
                    || fact.fields.iter().any(|(key, value)| {
                        contains_ascii_folded(key, term) || contains_ascii_folded(value, term)
                    })
            })
    }
}

// Preserve ASCII-only folding: non-ASCII UTF-8 bytes are compared unchanged.
// SIMD candidates have a linear comparison budget. Repeated-prefix inputs
// switch to prepared KMP rather than degrade to O(haystack * needle).
fn contains_ascii_folded(value: &str, term: &PreparedTerm) -> bool {
    let needle = term.bytes.as_slice();
    let Some(&first) = needle.first() else {
        return true;
    };
    let bytes = value.as_bytes();
    let budget = bytes.len() / needle.len() + 1;
    for (attempt, start) in
        memchr::memchr2_iter(first, first.to_ascii_uppercase(), bytes).enumerate()
    {
        if attempt == budget {
            let mut matched = 0;
            for byte in bytes {
                let byte = byte.to_ascii_lowercase();
                while matched > 0 && byte != needle[matched] {
                    matched = term.prefixes[matched - 1];
                }
                if byte == needle[matched] {
                    matched += 1;
                }
                if matched == needle.len() {
                    return true;
                }
            }
            return false;
        }
        if bytes
            .get(start..start + needle.len())
            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(needle))
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
#[path = "../../tests/unit/document_query_match.rs"]
mod tests;
