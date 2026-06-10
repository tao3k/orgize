//! SQL-friendly row projection for the flat Org elements index.

#[cfg(feature = "datafusion-sql")]
use std::sync::Arc;

#[cfg(feature = "datafusion-sql")]
use datafusion::{
    arrow::{
        array::{ArrayRef, Int64Array, StringArray},
        datatypes::{DataType, Field, Schema, SchemaRef},
        record_batch::RecordBatch,
    },
    datasource::MemTable,
    error::{DataFusionError, Result as DataFusionResult},
    prelude::SessionContext,
};
use serde_json::{Map, Value, json};

use super::{
    Document, OrgElementsIndexCategory, OrgElementsIndexKind, OrgElementsIndexRecord,
    OrgElementsIndexSummary, OrgElementsIndexSummaryValue, ParsedAnnotation,
};

/// One column in the stable `org_elements` SQL projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrgElementsSqlColumn {
    pub name: &'static str,
    pub sql_type: &'static str,
    pub nullable: bool,
}

/// One source-backed row in the stable `org_elements` SQL projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementsSqlRow {
    pub ordinal: usize,
    pub category: OrgElementsIndexCategory,
    pub kind: OrgElementsIndexKind,
    pub affiliated_name: Option<String>,
    pub outline_path_json: String,
    pub context: String,
    pub summary_json: String,
    pub language: Option<String>,
    pub source_start_line: usize,
    pub source_start_column: usize,
    pub source_end_line: usize,
    pub source_end_column: usize,
    pub source_range_start: u32,
    pub source_range_end: u32,
    pub source_raw: String,
}

/// Stable schema for the `org_elements` SQL projection.
pub const ORG_ELEMENTS_SQL_COLUMNS: &[OrgElementsSqlColumn] = &[
    OrgElementsSqlColumn {
        name: "ordinal",
        sql_type: "BIGINT",
        nullable: false,
    },
    OrgElementsSqlColumn {
        name: "category",
        sql_type: "TEXT",
        nullable: false,
    },
    OrgElementsSqlColumn {
        name: "kind",
        sql_type: "TEXT",
        nullable: false,
    },
    OrgElementsSqlColumn {
        name: "affiliated_name",
        sql_type: "TEXT",
        nullable: true,
    },
    OrgElementsSqlColumn {
        name: "outline_path_json",
        sql_type: "TEXT",
        nullable: false,
    },
    OrgElementsSqlColumn {
        name: "context",
        sql_type: "TEXT",
        nullable: false,
    },
    OrgElementsSqlColumn {
        name: "summary_json",
        sql_type: "TEXT",
        nullable: false,
    },
    OrgElementsSqlColumn {
        name: "language",
        sql_type: "TEXT",
        nullable: true,
    },
    OrgElementsSqlColumn {
        name: "source_start_line",
        sql_type: "BIGINT",
        nullable: false,
    },
    OrgElementsSqlColumn {
        name: "source_start_column",
        sql_type: "BIGINT",
        nullable: false,
    },
    OrgElementsSqlColumn {
        name: "source_end_line",
        sql_type: "BIGINT",
        nullable: false,
    },
    OrgElementsSqlColumn {
        name: "source_end_column",
        sql_type: "BIGINT",
        nullable: false,
    },
    OrgElementsSqlColumn {
        name: "source_range_start",
        sql_type: "BIGINT",
        nullable: false,
    },
    OrgElementsSqlColumn {
        name: "source_range_end",
        sql_type: "BIGINT",
        nullable: false,
    },
    OrgElementsSqlColumn {
        name: "source_raw",
        sql_type: "TEXT",
        nullable: false,
    },
];

pub(super) fn sql_rows(document: &Document<ParsedAnnotation>) -> Vec<OrgElementsSqlRow> {
    super::elements_bridge_index::index_records(document)
        .iter()
        .map(sql_row)
        .collect()
}

pub(super) fn sql_rows_from_records(
    records: &[OrgElementsIndexRecord<ParsedAnnotation>],
) -> Vec<OrgElementsSqlRow> {
    records.iter().map(sql_row).collect()
}

pub(super) fn sql_rows_json(rows: &[OrgElementsSqlRow]) -> String {
    serde_json::to_string(
        &rows
            .iter()
            .map(sql_row_json)
            .collect::<Vec<serde_json::Value>>(),
    )
    .expect("Org elements SQL rows JSON serialization should not fail")
}

#[cfg(feature = "datafusion-sql")]
pub(super) fn sql_record_batch(rows: &[OrgElementsSqlRow]) -> DataFusionResult<RecordBatch> {
    let schema = sql_schema();
    let arrays: Vec<ArrayRef> = vec![
        int_array(rows.iter().map(|row| row.ordinal)),
        text_array(rows.iter().map(|row| row.category.as_str())),
        text_array(rows.iter().map(|row| row.kind.as_str())),
        nullable_text_array(rows.iter().map(|row| row.affiliated_name.as_deref())),
        text_array(rows.iter().map(|row| row.outline_path_json.as_str())),
        text_array(rows.iter().map(|row| row.context.as_str())),
        text_array(rows.iter().map(|row| row.summary_json.as_str())),
        nullable_text_array(rows.iter().map(|row| row.language.as_deref())),
        int_array(rows.iter().map(|row| row.source_start_line)),
        int_array(rows.iter().map(|row| row.source_start_column)),
        int_array(rows.iter().map(|row| row.source_end_line)),
        int_array(rows.iter().map(|row| row.source_end_column)),
        int_array(rows.iter().map(|row| row.source_range_start as usize)),
        int_array(rows.iter().map(|row| row.source_range_end as usize)),
        text_array(rows.iter().map(|row| row.source_raw.as_str())),
    ];
    RecordBatch::try_new(schema, arrays)
        .map_err(|error| DataFusionError::ArrowError(Box::new(error), None))
}

#[cfg(feature = "datafusion-sql")]
pub(super) async fn query_sql_rows(
    rows: &[OrgElementsSqlRow],
    sql: &str,
) -> DataFusionResult<Vec<RecordBatch>> {
    let batch = sql_record_batch(rows)?;
    let table = MemTable::try_new(batch.schema(), vec![vec![batch]])?;
    let context = SessionContext::new();
    context.register_table("org_elements", Arc::new(table))?;
    context.sql(sql).await?.collect().await
}

#[cfg(feature = "datafusion-sql")]
fn sql_schema() -> SchemaRef {
    Arc::new(Schema::new(
        ORG_ELEMENTS_SQL_COLUMNS
            .iter()
            .map(|column| {
                Field::new(
                    column.name,
                    match column.sql_type {
                        "BIGINT" => DataType::Int64,
                        "TEXT" => DataType::Utf8,
                        other => unreachable!("unsupported org_elements SQL type: {other}"),
                    },
                    column.nullable,
                )
            })
            .collect::<Vec<_>>(),
    ))
}

#[cfg(feature = "datafusion-sql")]
fn int_array(values: impl Iterator<Item = usize>) -> ArrayRef {
    Arc::new(Int64Array::from(
        values.map(|value| value as i64).collect::<Vec<_>>(),
    ))
}

#[cfg(feature = "datafusion-sql")]
fn text_array<'a>(values: impl Iterator<Item = &'a str>) -> ArrayRef {
    Arc::new(StringArray::from(
        values
            .map(|value| Some(value.to_string()))
            .collect::<Vec<_>>(),
    ))
}

#[cfg(feature = "datafusion-sql")]
fn nullable_text_array<'a>(values: impl Iterator<Item = Option<&'a str>>) -> ArrayRef {
    Arc::new(StringArray::from(
        values
            .map(|value| value.map(str::to_string))
            .collect::<Vec<_>>(),
    ))
}

fn sql_row(record: &OrgElementsIndexRecord<ParsedAnnotation>) -> OrgElementsSqlRow {
    OrgElementsSqlRow {
        ordinal: record.ordinal,
        category: record.category,
        kind: record.kind.clone(),
        affiliated_name: record.affiliated.name.clone(),
        outline_path_json: serde_json::to_string(&record.outline_path)
            .expect("outline path JSON serialization should not fail"),
        context: record.context.clone(),
        summary_json: serde_json::to_string(&summary_json(&record.summary))
            .expect("summary JSON serialization should not fail"),
        language: summary_text(&record.summary, "language"),
        source_start_line: record.ann.start.line,
        source_start_column: record.ann.start.column,
        source_end_line: record.ann.end.line,
        source_end_column: record.ann.end.column,
        source_range_start: u32::from(record.ann.range.start()),
        source_range_end: u32::from(record.ann.range.end()),
        source_raw: record.ann.raw.clone(),
    }
}

fn sql_row_json(row: &OrgElementsSqlRow) -> Value {
    json!({
        "ordinal": row.ordinal,
        "category": row.category.as_str(),
        "kind": row.kind.as_str(),
        "affiliatedName": row.affiliated_name,
        "outlinePathJson": row.outline_path_json,
        "context": row.context,
        "summaryJson": row.summary_json,
        "language": row.language,
        "sourceStartLine": row.source_start_line,
        "sourceStartColumn": row.source_start_column,
        "sourceEndLine": row.source_end_line,
        "sourceEndColumn": row.source_end_column,
        "sourceRangeStart": row.source_range_start,
        "sourceRangeEnd": row.source_range_end,
        "sourceRaw": row.source_raw,
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

fn summary_text(summary: &OrgElementsIndexSummary, key: &str) -> Option<String> {
    match summary.get(key) {
        Some(OrgElementsIndexSummaryValue::Text(value)) => Some(value.clone()),
        _ => None,
    }
}
