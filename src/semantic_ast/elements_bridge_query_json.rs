//! JSON contract for host-facing Org elements index queries.

use std::{collections::BTreeSet, error::Error, fmt};

use serde_json::{Map, Value, json};

use super::{
    OrgElementId, OrgElementQueryPredicate, OrgElementsIndexCategory, OrgElementsIndexKind,
    OrgElementsIndexQuery, OrgElementsIndexRelation, OrgElementsIndexSummaryPredicate,
    OrgElementsIndexSummaryTextPredicate, OrgElementsIndexSummaryValue,
};

/// Error returned when a host JSON query packet cannot be parsed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementsIndexQueryJsonError {
    message: String,
}

impl OrgElementsIndexQueryJsonError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for OrgElementsIndexQueryJsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for OrgElementsIndexQueryJsonError {}

/// Parses a host-facing JSON query packet into the shared query AST.
pub fn query_from_json_str(
    input: &str,
) -> Result<OrgElementsIndexQuery, OrgElementsIndexQueryJsonError> {
    let value = serde_json::from_str(input)
        .map_err(|error| OrgElementsIndexQueryJsonError::new(error.to_string()))?;
    query_from_json_value(&value)
}

/// Parses a host-facing JSON query packet into the shared query AST.
pub fn query_from_json_value(
    value: &Value,
) -> Result<OrgElementsIndexQuery, OrgElementsIndexQueryJsonError> {
    let object = expect_object(value, "query packet")?;
    validate_schema_version(object)?;
    query_from_json_object(object)
}

fn query_from_json_object(
    object: &Map<String, Value>,
) -> Result<OrgElementsIndexQuery, OrgElementsIndexQueryJsonError> {
    let mut query = OrgElementsIndexQuery::new();
    query = query_with_header_fields(query, object)?;
    query = query_with_summary_fields(query, object)?;
    query = query_with_relation_fields(query, object)?;
    query = query_with_predicate_field(query, object)?;
    query_with_limit(query, object)
}

fn query_with_header_fields(
    mut query: OrgElementsIndexQuery,
    object: &Map<String, Value>,
) -> Result<OrgElementsIndexQuery, OrgElementsIndexQueryJsonError> {
    if let Some(category) = optional_string(object, "category")? {
        query = query.category(OrgElementsIndexCategory::from_label(&category).ok_or_else(
            || OrgElementsIndexQueryJsonError::new(format!("unknown category `{category}`")),
        )?);
    }
    if let Some(kind) = optional_string(object, "kind")? {
        query = query.kind(kind);
    }
    if let Some(name) = optional_string(object, "affiliatedName")? {
        query = query.affiliated_name(name);
    }
    if let Some(context) = optional_string(object, "context")? {
        query = query.context(context);
    }
    if let Some(outline_path_prefix) = optional_string_array(object, "outlinePathPrefix")? {
        query = query.outline_path_prefix(outline_path_prefix);
    }
    if let Some(outline_path_exact_len) = optional_usize(object, "outlinePathExactLen")? {
        query = query.outline_path_exact_len(outline_path_exact_len);
    }
    Ok(query)
}

fn query_with_summary_fields(
    mut query: OrgElementsIndexQuery,
    object: &Map<String, Value>,
) -> Result<OrgElementsIndexQuery, OrgElementsIndexQueryJsonError> {
    for predicate in summary_predicates(object.get("propertyEquals"), "propertyEquals")? {
        query = query.property_eq(predicate.key, predicate.value);
    }
    for predicate in text_predicates(object.get("propertyContains"), "propertyContains")? {
        query = query.property_contains(predicate.key, predicate.needle);
    }
    for predicate in summary_predicates(object.get("summaryEquals"), "summaryEquals")? {
        query = query.summary_eq(predicate.key, predicate.value);
    }
    for predicate in text_predicates(object.get("summaryContains"), "summaryContains")? {
        query = query.summary_contains(predicate.key, predicate.needle);
    }
    Ok(query)
}

fn query_with_relation_fields(
    mut query: OrgElementsIndexQuery,
    object: &Map<String, Value>,
) -> Result<OrgElementsIndexQuery, OrgElementsIndexQueryJsonError> {
    if let Some(relations) = object.get("relations") {
        query.relations = relations_from_json(relations)?;
    }
    Ok(query)
}

fn query_with_predicate_field(
    mut query: OrgElementsIndexQuery,
    object: &Map<String, Value>,
) -> Result<OrgElementsIndexQuery, OrgElementsIndexQueryJsonError> {
    if let Some(predicate) = object.get("predicate") {
        query = query.predicate(predicate_from_json(predicate)?);
    }
    Ok(query)
}

fn query_with_limit(
    mut query: OrgElementsIndexQuery,
    object: &Map<String, Value>,
) -> Result<OrgElementsIndexQuery, OrgElementsIndexQueryJsonError> {
    if let Some(limit) = optional_usize(object, "limit")? {
        query = query.limit(limit);
    }
    Ok(query)
}

/// Renders a canonical JSON query packet for snapshots and host receipts.
pub fn query_to_json_value(query: &OrgElementsIndexQuery) -> Value {
    Value::Object(query_to_json_object(query))
}

fn query_to_json_object(query: &OrgElementsIndexQuery) -> Map<String, Value> {
    let mut object = Map::new();
    object.insert("schemaVersion".to_string(), json!(1));
    insert_query_header_json(&mut object, query);
    insert_query_summary_json(&mut object, query);
    insert_query_relation_json(&mut object, query);
    insert_query_predicate_json(&mut object, query);
    insert_query_limit_json(&mut object, query);
    object
}

fn insert_query_header_json(object: &mut Map<String, Value>, query: &OrgElementsIndexQuery) {
    if let Some(category) = query.category {
        object.insert("category".to_string(), json!(category.as_str()));
    }
    if let Some(kind) = &query.kind {
        object.insert("kind".to_string(), json!(kind.as_str()));
    }
    if let Some(name) = &query.affiliated_name {
        object.insert("affiliatedName".to_string(), json!(name));
    }
    if let Some(context) = &query.context {
        object.insert("context".to_string(), json!(context));
    }
    if !query.outline_path_prefix.is_empty() {
        object.insert(
            "outlinePathPrefix".to_string(),
            json!(&query.outline_path_prefix),
        );
    }
    if let Some(outline_path_exact_len) = query.outline_path_exact_len {
        object.insert(
            "outlinePathExactLen".to_string(),
            json!(outline_path_exact_len),
        );
    }
}

fn insert_query_summary_json(object: &mut Map<String, Value>, query: &OrgElementsIndexQuery) {
    if !query.property_equals.is_empty() {
        object.insert(
            "propertyEquals".to_string(),
            summary_predicates_json(&query.property_equals),
        );
    }
    if !query.property_contains.is_empty() {
        object.insert(
            "propertyContains".to_string(),
            text_predicates_json(&query.property_contains),
        );
    }
    if !query.summary_equals.is_empty() {
        object.insert(
            "summaryEquals".to_string(),
            summary_predicates_json(&query.summary_equals),
        );
    }
    if !query.summary_contains.is_empty() {
        object.insert(
            "summaryContains".to_string(),
            text_predicates_json(&query.summary_contains),
        );
    }
}

fn insert_query_relation_json(object: &mut Map<String, Value>, query: &OrgElementsIndexQuery) {
    if !query.relations.is_empty() {
        object.insert("relations".to_string(), relations_json(&query.relations));
    }
}

fn insert_query_predicate_json(object: &mut Map<String, Value>, query: &OrgElementsIndexQuery) {
    if query.predicate != OrgElementQueryPredicate::default() {
        object.insert("predicate".to_string(), predicate_json(&query.predicate));
    }
}

fn insert_query_limit_json(object: &mut Map<String, Value>, query: &OrgElementsIndexQuery) {
    if let Some(limit) = query.limit {
        object.insert("limit".to_string(), json!(limit));
    }
}

fn validate_schema_version(
    object: &Map<String, Value>,
) -> Result<(), OrgElementsIndexQueryJsonError> {
    match object.get("schemaVersion") {
        None => Ok(()),
        Some(Value::Number(version)) if version.as_u64() == Some(1) => Ok(()),
        Some(_) => Err(OrgElementsIndexQueryJsonError::new(
            "schemaVersion must be 1 when present",
        )),
    }
}

fn predicate_from_json(
    value: &Value,
) -> Result<OrgElementQueryPredicate, OrgElementsIndexQueryJsonError> {
    let object = expect_object(value, "predicate")?;
    if let Some(predicates) = object.get("all") {
        return Ok(OrgElementQueryPredicate::all(predicate_array(
            predicates, "all",
        )?));
    }
    if let Some(predicates) = object.get("any") {
        return Ok(OrgElementQueryPredicate::any(predicate_array(
            predicates, "any",
        )?));
    }
    if let Some(predicate) = object.get("not") {
        return Ok(OrgElementQueryPredicate::negate(predicate_from_json(
            predicate,
        )?));
    }
    if let Some(category) = optional_string(object, "category")? {
        return OrgElementsIndexCategory::from_label(&category)
            .map(OrgElementQueryPredicate::Category)
            .ok_or_else(|| {
                OrgElementsIndexQueryJsonError::new(format!("unknown category `{category}`"))
            });
    }
    if let Some(kind) = optional_string(object, "kind")? {
        return Ok(OrgElementQueryPredicate::Kind(OrgElementsIndexKind::new(
            kind,
        )));
    }
    if let Some(name) = optional_string(object, "affiliatedName")? {
        return Ok(OrgElementQueryPredicate::AffiliatedName(name));
    }
    if let Some(context) = optional_string(object, "context")? {
        return Ok(OrgElementQueryPredicate::Context(context));
    }
    if let Some(property) = object.get("property") {
        return field_predicate(property, "property");
    }
    if let Some(summary) = object.get("summary") {
        return field_predicate(summary, "summary");
    }
    Err(OrgElementsIndexQueryJsonError::new(
        "predicate must contain one supported predicate key",
    ))
}

fn predicate_array(
    value: &Value,
    field: &str,
) -> Result<Vec<OrgElementQueryPredicate>, OrgElementsIndexQueryJsonError> {
    let values = value.as_array().ok_or_else(|| {
        OrgElementsIndexQueryJsonError::new(format!("predicate `{field}` must be an array"))
    })?;
    values.iter().map(predicate_from_json).collect()
}

fn field_predicate(
    value: &Value,
    field: &str,
) -> Result<OrgElementQueryPredicate, OrgElementsIndexQueryJsonError> {
    let object = expect_object(value, field)?;
    let key = required_string(object, "key")?;
    if let Some(value) = object.get("equals") {
        let value = summary_value(value)?;
        return match field {
            "property" => Ok(OrgElementQueryPredicate::PropertyEquals(
                OrgElementsIndexSummaryPredicate { key, value },
            )),
            "summary" => Ok(OrgElementQueryPredicate::SummaryEquals(
                OrgElementsIndexSummaryPredicate { key, value },
            )),
            _ => unreachable!("field predicate caller controls field"),
        };
    }
    if let Some(needle) = optional_string(object, "contains")? {
        return match field {
            "property" => Ok(OrgElementQueryPredicate::PropertyContains(
                OrgElementsIndexSummaryTextPredicate { key, needle },
            )),
            "summary" => Ok(OrgElementQueryPredicate::SummaryContains(
                OrgElementsIndexSummaryTextPredicate { key, needle },
            )),
            _ => unreachable!("field predicate caller controls field"),
        };
    }
    Err(OrgElementsIndexQueryJsonError::new(format!(
        "`{field}` predicate must contain `equals` or `contains`",
    )))
}

fn summary_predicates(
    value: Option<&Value>,
    field: &str,
) -> Result<Vec<OrgElementsIndexSummaryPredicate>, OrgElementsIndexQueryJsonError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let values = value.as_array().ok_or_else(|| {
        OrgElementsIndexQueryJsonError::new(format!("`{field}` must be an array"))
    })?;
    values
        .iter()
        .map(|value| {
            let object = expect_object(value, field)?;
            Ok(OrgElementsIndexSummaryPredicate {
                key: required_string(object, "key")?,
                value: summary_value(object.get("value").ok_or_else(|| {
                    OrgElementsIndexQueryJsonError::new(
                        format!("`{field}` entry requires `value`",),
                    )
                })?)?,
            })
        })
        .collect()
}

fn text_predicates(
    value: Option<&Value>,
    field: &str,
) -> Result<Vec<OrgElementsIndexSummaryTextPredicate>, OrgElementsIndexQueryJsonError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let values = value.as_array().ok_or_else(|| {
        OrgElementsIndexQueryJsonError::new(format!("`{field}` must be an array"))
    })?;
    values
        .iter()
        .map(|value| {
            let object = expect_object(value, field)?;
            Ok(OrgElementsIndexSummaryTextPredicate {
                key: required_string(object, "key")?,
                needle: required_string(object, "needle")?,
            })
        })
        .collect()
}

fn relations_from_json(
    value: &Value,
) -> Result<Vec<OrgElementsIndexRelation>, OrgElementsIndexQueryJsonError> {
    let values = value
        .as_array()
        .ok_or_else(|| OrgElementsIndexQueryJsonError::new("`relations` must be an array"))?;
    values
        .iter()
        .map(|value| {
            let object = expect_object(value, "relation")?;
            let relation_type = required_string(object, "type")?;
            let ids =
                id_set(object.get("ids").ok_or_else(|| {
                    OrgElementsIndexQueryJsonError::new("relation requires `ids`")
                })?)?;
            match relation_type.as_str() {
                "childOf" => Ok(OrgElementsIndexRelation::ChildOf(ids)),
                "descendantOf" => Ok(OrgElementsIndexRelation::DescendantOf(ids)),
                "ancestorOf" => Ok(OrgElementsIndexRelation::AncestorOf(ids)),
                "at" => Ok(OrgElementsIndexRelation::At(ids)),
                _ => Err(OrgElementsIndexQueryJsonError::new(format!(
                    "unknown relation type `{relation_type}`",
                ))),
            }
        })
        .collect()
}

fn id_set(value: &Value) -> Result<BTreeSet<OrgElementId>, OrgElementsIndexQueryJsonError> {
    let values = value
        .as_array()
        .ok_or_else(|| OrgElementsIndexQueryJsonError::new("relation `ids` must be an array"))?;
    values
        .iter()
        .map(|value| {
            value
                .as_u64()
                .and_then(|value| usize::try_from(value).ok())
                .map(OrgElementId::new)
                .ok_or_else(|| {
                    OrgElementsIndexQueryJsonError::new(
                        "relation ids must be non-negative integers",
                    )
                })
        })
        .collect()
}

fn predicate_json(predicate: &OrgElementQueryPredicate) -> Value {
    match predicate {
        OrgElementQueryPredicate::All(predicates) => json!({
            "all": predicates.iter().map(predicate_json).collect::<Vec<_>>()
        }),
        OrgElementQueryPredicate::Any(predicates) => json!({
            "any": predicates.iter().map(predicate_json).collect::<Vec<_>>()
        }),
        OrgElementQueryPredicate::Not(predicate) => json!({ "not": predicate_json(predicate) }),
        OrgElementQueryPredicate::Category(category) => json!({ "category": category.as_str() }),
        OrgElementQueryPredicate::Kind(kind) => json!({ "kind": kind.as_str() }),
        OrgElementQueryPredicate::AffiliatedName(name) => json!({ "affiliatedName": name }),
        OrgElementQueryPredicate::Context(context) => json!({ "context": context }),
        OrgElementQueryPredicate::PropertyEquals(predicate) => json!({
            "property": {
                "key": &predicate.key,
                "equals": summary_value_json(&predicate.value),
            }
        }),
        OrgElementQueryPredicate::PropertyContains(predicate) => json!({
            "property": {
                "key": &predicate.key,
                "contains": &predicate.needle,
            }
        }),
        OrgElementQueryPredicate::SummaryEquals(predicate) => json!({
            "summary": {
                "key": &predicate.key,
                "equals": summary_value_json(&predicate.value),
            }
        }),
        OrgElementQueryPredicate::SummaryContains(predicate) => json!({
            "summary": {
                "key": &predicate.key,
                "contains": &predicate.needle,
            }
        }),
    }
}

fn relations_json(relations: &[OrgElementsIndexRelation]) -> Value {
    Value::Array(
        relations
            .iter()
            .map(|relation| match relation {
                OrgElementsIndexRelation::ChildOf(ids) => relation_json("childOf", ids),
                OrgElementsIndexRelation::DescendantOf(ids) => relation_json("descendantOf", ids),
                OrgElementsIndexRelation::AncestorOf(ids) => relation_json("ancestorOf", ids),
                OrgElementsIndexRelation::At(ids) => relation_json("at", ids),
            })
            .collect(),
    )
}

fn relation_json(relation_type: &str, ids: &BTreeSet<OrgElementId>) -> Value {
    json!({
        "type": relation_type,
        "ids": ids.iter().map(|id| id.as_usize()).collect::<Vec<_>>(),
    })
}

fn summary_predicates_json(predicates: &[OrgElementsIndexSummaryPredicate]) -> Value {
    Value::Array(
        predicates
            .iter()
            .map(|predicate| {
                json!({
                    "key": &predicate.key,
                    "value": summary_value_json(&predicate.value),
                })
            })
            .collect(),
    )
}

fn text_predicates_json(predicates: &[OrgElementsIndexSummaryTextPredicate]) -> Value {
    Value::Array(
        predicates
            .iter()
            .map(|predicate| {
                json!({
                    "key": &predicate.key,
                    "needle": &predicate.needle,
                })
            })
            .collect(),
    )
}

fn summary_value(
    value: &Value,
) -> Result<OrgElementsIndexSummaryValue, OrgElementsIndexQueryJsonError> {
    match value {
        Value::Null => Ok(OrgElementsIndexSummaryValue::Null),
        Value::Bool(value) => Ok(OrgElementsIndexSummaryValue::Bool(*value)),
        Value::Number(value) => value
            .as_i64()
            .map(OrgElementsIndexSummaryValue::Integer)
            .ok_or_else(|| {
                OrgElementsIndexQueryJsonError::new("numeric summary values must fit in i64")
            }),
        Value::String(value) => Ok(OrgElementsIndexSummaryValue::Text(value.clone())),
        Value::Array(values) => values
            .iter()
            .map(|value| {
                value.as_str().map(str::to_string).ok_or_else(|| {
                    OrgElementsIndexQueryJsonError::new("summary arrays must contain only strings")
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(OrgElementsIndexSummaryValue::StringList),
        Value::Object(_) => Err(OrgElementsIndexQueryJsonError::new(
            "object summary values are not supported",
        )),
    }
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

fn optional_string(
    object: &Map<String, Value>,
    key: &str,
) -> Result<Option<String>, OrgElementsIndexQueryJsonError> {
    object
        .get(key)
        .map(|value| {
            value.as_str().map(str::to_string).ok_or_else(|| {
                OrgElementsIndexQueryJsonError::new(format!("`{key}` must be a string"))
            })
        })
        .transpose()
}

fn required_string(
    object: &Map<String, Value>,
    key: &str,
) -> Result<String, OrgElementsIndexQueryJsonError> {
    optional_string(object, key)?.ok_or_else(|| {
        OrgElementsIndexQueryJsonError::new(format!("missing required string `{key}`"))
    })
}

fn optional_string_array(
    object: &Map<String, Value>,
    key: &str,
) -> Result<Option<Vec<String>>, OrgElementsIndexQueryJsonError> {
    object
        .get(key)
        .map(|value| {
            let values = value.as_array().ok_or_else(|| {
                OrgElementsIndexQueryJsonError::new(format!("`{key}` must be an array"))
            })?;
            values
                .iter()
                .map(|value| {
                    value.as_str().map(str::to_string).ok_or_else(|| {
                        OrgElementsIndexQueryJsonError::new(format!(
                            "`{key}` must contain only strings",
                        ))
                    })
                })
                .collect()
        })
        .transpose()
}

fn optional_usize(
    object: &Map<String, Value>,
    key: &str,
) -> Result<Option<usize>, OrgElementsIndexQueryJsonError> {
    object
        .get(key)
        .map(|value| {
            value
                .as_u64()
                .and_then(|value| usize::try_from(value).ok())
                .ok_or_else(|| {
                    OrgElementsIndexQueryJsonError::new(format!(
                        "`{key}` must be a non-negative integer",
                    ))
                })
        })
        .transpose()
}

fn expect_object<'a>(
    value: &'a Value,
    context: &str,
) -> Result<&'a Map<String, Value>, OrgElementsIndexQueryJsonError> {
    value.as_object().ok_or_else(|| {
        OrgElementsIndexQueryJsonError::new(format!("`{context}` must be a JSON object"))
    })
}
