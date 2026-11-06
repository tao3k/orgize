//! Request-local admission of Scheme-owned property descriptors and tokens.
use super::{AstRef, Document, ParsedAnnotation, Property};
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub(super) const FIXED: [(&str, &str); 2] = [
    ("VISIBILITY_ALL", "folded children content all"),
    ("CLOCK_MODELINE_TOTAL_ALL", "current today repeat all auto"),
];

pub(crate) struct PropertyNativeFacts {
    pub(crate) descriptor_key: String,
    pub(crate) descriptor_name: Option<String>,
    pub(crate) tokens: Vec<String>,
}

pub(crate) struct PropertyNativePlan {
    facts: HashMap<String, HashMap<String, PropertyNativeFacts>>,
}

impl PropertyNativePlan {
    pub(super) fn new(document: &Document<ParsedAnnotation>) -> Self {
        // Deduplicate before copying inherited values, not after allocating
        // every ancestor value once per descendant. Preserve exact byte order.
        let mut unique: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut insert = |key: &str, value: &str| {
            if let Some(values) = unique.get_mut(key) {
                if !values.contains(value) {
                    values.insert(value.to_owned());
                }
            } else {
                unique.insert(key.to_owned(), BTreeSet::from([value.to_owned()]));
            }
        };
        for (key, value) in FIXED {
            insert(key, value);
        }
        document.visit(|node| match node {
            AstRef::Property(property) => {
                insert(&property.key, &property.value);
            }
            AstRef::Section(section) => {
                for p in &section.effective_properties {
                    insert(&p.key, &p.value);
                }
            }
            _ => {}
        });
        // Exact bytes only: Scheme, not this transport, normalizes descriptors.
        let inputs = unique
            .into_iter()
            .flat_map(|(key, values)| values.into_iter().map(move |value| (key.clone(), value)))
            .collect::<Vec<_>>();
        let fields = inputs
            .iter()
            .flat_map(|(key, value)| [key.as_str(), value.as_str()])
            .collect::<Vec<_>>();
        let rows = super::org_values::rows("property-profile-plan", &fields);
        Self::admit(inputs, rows)
    }

    fn admit(inputs: Vec<(String, String)>, rows: Vec<Vec<String>>) -> Self {
        assert_eq!(inputs.len(), rows.len(), "native property plan row count");
        let mut facts: HashMap<String, HashMap<String, PropertyNativeFacts>> = HashMap::new();
        for ((key, value), row) in inputs.into_iter().zip(rows) {
            let mut row = row.into_iter();
            let descriptor_key = row.next().expect("native descriptor key");
            let present = row.next().expect("native descriptor presence");
            let name = row.next().expect("native descriptor name");
            let descriptor_name = match present.as_str() {
                "true" => Some(name),
                "false" => {
                    assert!(name.is_empty(), "absent native descriptor name");
                    None
                }
                _ => panic!("native descriptor presence"),
            };
            assert!(
                facts
                    .entry(key)
                    .or_default()
                    .insert(
                        value,
                        PropertyNativeFacts {
                            descriptor_key,
                            descriptor_name,
                            tokens: row.collect(),
                        }
                    )
                    .is_none(),
                "duplicate native property input"
            );
        }
        Self { facts }
    }

    pub(crate) fn fact(&self, key: &str, value: &str) -> &PropertyNativeFacts {
        self.facts
            .get(key)
            .and_then(|values| values.get(value))
            .expect("property belongs to this native plan")
    }

    pub(crate) fn property(&self, property: &Property<ParsedAnnotation>) -> &PropertyNativeFacts {
        self.fact(&property.key, &property.value)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/property_plan.rs"]
mod tests;
