//! Batched admission of native duration facts for parser-owned properties.
use crate::org_aot::OrgAotDocument;
use std::collections::HashMap;

pub(super) fn property_duration_plan(document: &OrgAotDocument) -> HashMap<usize, Option<u64>> {
    let records = document
        .records()
        .iter()
        .filter(|record| record.kind == "node-property")
        .filter_map(|record| record.field("value").map(|value| (record.id, value)))
        .collect::<Vec<_>>();
    let mut facts = HashMap::with_capacity(records.len());
    for chunk in records.chunks(32) {
        let fields = chunk.iter().map(|(_, value)| *value).collect::<Vec<_>>();
        let rows = super::org_native_values::rows("duration-plan", &fields);
        assert_eq!(rows.len(), chunk.len(), "native property duration count");
        for ((id, _), row) in chunk.iter().zip(rows) {
            let [present, seconds]: [String; 2] =
                row.try_into().expect("native property duration arity");
            let value = match present.as_str() {
                "true" => Some(seconds.parse().expect("native property duration seconds")),
                "false" => {
                    assert!(seconds.is_empty(), "absent native property duration");
                    None
                }
                _ => panic!("native property duration presence"),
            };
            assert!(
                facts.insert(*id, value).is_none(),
                "duplicate native duration property"
            );
        }
    }
    facts
}
