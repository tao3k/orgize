//! JSON projection for the flat Org elements index.

use serde_json::{Map, Value, json};

use super::{
    Document, OrgElementsIndexRecord, OrgElementsIndexSummary, OrgElementsIndexSummaryValue,
    ParsedAnnotation,
};

pub(super) fn index_json(document: &Document<ParsedAnnotation>) -> Vec<Value> {
    super::elements_bridge_index::index_records(document)
        .iter()
        .map(record_json)
        .collect()
}

pub(super) fn index_json_from_records(
    records: &[OrgElementsIndexRecord<ParsedAnnotation>],
) -> Vec<Value> {
    records.iter().map(record_json).collect()
}

fn record_json(record: &OrgElementsIndexRecord<ParsedAnnotation>) -> Value {
    json!({
        "id": record.id.as_usize(),
        "parentId": record.parent_id.map(|id| id.as_usize()),
        "childIds": record.child_ids.iter().map(|id| id.as_usize()).collect::<Vec<_>>(),
        "ordinal": record.ordinal,
        "category": record.category.as_str(),
        "kind": record.kind.as_str(),
        "affiliated": {
            "name": &record.affiliated.name,
        },
        "source": super::elements_bridge_json::annotation_json(&record.ann),
        "outlinePath": record.outline_path,
        "context": record.context,
        "properties": summary_json(&record.properties),
        "summary": summary_json(&record.summary),
    })
}

fn summary_json(summary: &OrgElementsIndexSummary) -> Value {
    Value::Object(
        summary
            .iter()
            .map(|(key, value)| (key.clone(), summary_value_json(value)))
            .collect::<Map<_, _>>(),
    )
}

fn summary_value_json(value: &OrgElementsIndexSummaryValue) -> Value {
    match value {
        OrgElementsIndexSummaryValue::Null => Value::Null,
        OrgElementsIndexSummaryValue::Bool(value) => Value::Bool(*value),
        OrgElementsIndexSummaryValue::Integer(value) => json!(value),
        OrgElementsIndexSummaryValue::Text(value) => json!(value),
        OrgElementsIndexSummaryValue::StringList(value) => json!(value),
    }
}
