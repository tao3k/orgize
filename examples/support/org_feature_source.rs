//! Build-time access to feature-tagged Scheme blocks through Org Element records.

use gerbil_parser_runtime::GraphRecord;

/// Preserve quoted file imports and admit canonical Gerbil library identifiers.
pub fn module_reference(value: &str) -> Result<String, String> {
    match value.strip_prefix(':') {
        Some(name)
            if name.split('/').all(|segment| {
                !segment.is_empty()
                    && segment
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
            }) =>
        {
            Ok(value.to_owned())
        }
        Some(_) => Err("invalid canonical Gerbil module identifier".into()),
        None if value.is_empty() => Err("Scheme interface module cannot be empty".into()),
        None => Ok(format!("{value:?}")),
    }
}

pub fn records(source: &str) -> Result<Vec<GraphRecord>, String> {
    orgize::org_aot::parse_org_aot(source)
        .map(|document| document.records().to_vec())
        .map_err(|error| format!("invalid Org feature source: {error:?}"))
}

pub fn owning_headline(records: &[GraphRecord], mut id: usize) -> Option<usize> {
    loop {
        let record = records.get(id)?;
        if record.kind == "headline" {
            return Some(id);
        }
        id = record.parent_id?;
    }
}

pub fn property<'a>(
    records: &'a [GraphRecord],
    section: usize,
    key: &str,
) -> Result<Option<&'a str>, String> {
    let values: Vec<_> = records
        .iter()
        .filter(|record| {
            record.kind == "node-property" && owning_headline(records, record.id) == Some(section)
        })
        .filter(|record| record.field("key") == Some(key))
        .map(|record| record.field("value").map(str::trim))
        .collect();
    match values.as_slice() {
        [] => Ok(None),
        [Some(value)] => Ok(Some(value)),
        _ => Err(format!(
            "duplicate or empty {key} property in feature section"
        )),
    }
}

pub fn feature_block<'a>(
    records: &'a [GraphRecord],
    section: usize,
    feature: &str,
) -> Result<&'a str, String> {
    let blocks: Vec<_> = records
        .iter()
        .filter(|record| {
            record.kind == "src-block" && owning_headline(records, record.id) == Some(section)
        })
        .filter(|record| {
            record
                .field("language")
                .is_some_and(|language| language.eq_ignore_ascii_case("scheme"))
        })
        .filter(|record| {
            record.field("header").is_some_and(|header| {
                header
                    .split_ascii_whitespace()
                    .any(|part| part.eq_ignore_ascii_case(feature))
            })
        })
        .collect();
    let [block] = blocks.as_slice() else {
        return Err(format!("expected exactly one scheme {feature} block"));
    };
    let header = block.field("header").unwrap_or_default();
    if header
        .split_ascii_whitespace()
        .filter(|part| {
            part.eq_ignore_ascii_case(":org-elements-query")
                || part.eq_ignore_ascii_case(":org-contract")
        })
        .count()
        != 1
    {
        return Err("a Scheme block must select exactly one Org feature".into());
    }
    block
        .field("body")
        .ok_or_else(|| format!("scheme {feature} block has no body"))
}
