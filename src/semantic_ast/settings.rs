//! Typed admission of one native keyword plan; no word or option scanning.

use super::{ExportSettings, Keyword, LinkAbbreviation, ParsedAnnotation};

pub(super) struct KeywordFacts(Vec<Vec<String>>);

impl KeywordFacts {
    pub(super) fn from_native_rows(rows: &[Vec<String>]) -> Self {
        Self(rows.to_vec())
    }

    pub(super) fn new(key: &str, value: &str) -> Self {
        Self::batch(&[(key, value)])
            .pop()
            .expect("one native keyword plan")
    }

    pub(super) fn batch(inputs: &[(&str, &str)]) -> Vec<Self> {
        if inputs.is_empty() {
            return Vec::new();
        }
        let fields = inputs
            .iter()
            .flat_map(|(key, value)| [*key, *value])
            .collect::<Vec<_>>();
        let rows = crate::org_aot::native_semantic_rows(12, &fields)
            .expect("initialized native keyword operation");
        assert!(
            rows.iter().all(|row| row.len() == 2),
            "native keyword row arity"
        );
        let mut plans: Vec<Self> = Vec::with_capacity(inputs.len());
        for row in rows {
            if row[0] == "index" {
                assert_eq!(
                    row[1].parse::<usize>().expect("native keyword index"),
                    plans.len()
                );
                plans.push(Self(Vec::new()));
            } else {
                plans
                    .last_mut()
                    .expect("native keyword plan start")
                    .0
                    .push(row);
            }
        }
        assert_eq!(plans.len(), inputs.len(), "native keyword plan count");
        plans
    }
    pub(super) fn field(&self, key: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|row| row[0] == key)
            .map(|row| row[1].as_str())
    }
    pub(super) fn values(&self, key: &str) -> Vec<String> {
        self.0
            .iter()
            .filter(|row| row[0] == key)
            .map(|row| row[1].clone())
            .collect()
    }
    pub(super) fn apply_options(&self, settings: &mut ExportSettings) {
        if let Some(value) = self.field("H") {
            settings.headline_levels = value.parse().ok();
        }
        if let Some(value) = self.field("-") {
            settings.special_strings = native_boolean(value);
        }
        if let Some(value) = self.field("e") {
            settings.expand_entities = native_boolean(value);
        }
    }
}
fn native_boolean(value: &str) -> Option<bool> {
    match value {
        "true" => Some(true),
        "false" => Some(false),
        "" => None,
        _ => panic!("native keyword boolean"),
    }
}
pub(super) fn link_abbreviation(keyword: &Keyword<ParsedAnnotation>) -> Option<LinkAbbreviation> {
    link_abbreviation_from_facts(keyword, &KeywordFacts::new(&keyword.key, &keyword.value))
}
pub(super) fn link_abbreviation_from_facts(
    keyword: &Keyword<ParsedAnnotation>,
    facts: &KeywordFacts,
) -> Option<LinkAbbreviation> {
    let name = facts.field("first")?;
    let replacement = facts.field("rest")?;
    if name.is_empty() || replacement.is_empty() {
        return None;
    }
    Some(LinkAbbreviation {
        name: super::org_values::scalar("ascii-lower", &[name]),
        replacement: replacement.to_owned(),
        raw_value: keyword.value.clone(),
    })
}

pub(super) fn expand_link_abbreviation(
    protocol: &str,
    path: &str,
    abbreviations: &[LinkAbbreviation],
) -> Option<String> {
    let index = abbreviation_index(protocol, abbreviations)?;
    Some(crate::org_aot::org_expand_link_abbreviation(
        &abbreviations[index].replacement,
        path,
        &percent_encode(path),
    ))
}

pub(super) fn abbreviation_index(
    protocol: &str,
    abbreviations: &[LinkAbbreviation],
) -> Option<usize> {
    if abbreviations.is_empty() {
        return None;
    }
    let mut fields = vec![protocol];
    fields.extend(
        abbreviations
            .iter()
            .map(|abbreviation| abbreviation.name.as_str()),
    );
    let mut rows = super::org_values::rows("link-abbreviation-index", &fields);
    assert!(rows.len() <= 1, "native abbreviation count");
    let [index]: [String; 1] = rows.pop()?.try_into().expect("native abbreviation arity");
    let index: usize = index.parse().expect("native abbreviation index");
    abbreviations
        .get(index)
        .expect("native abbreviation bounds");
    Some(index)
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}
