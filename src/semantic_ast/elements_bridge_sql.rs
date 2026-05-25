//! SQL-friendly row projection for the flat Org elements index.

use serde_json::{Map, Value, json};

use super::{
    Document, OrgElementsIndexRecord, OrgElementsIndexSummary, OrgElementsIndexSummaryValue,
    ParsedAnnotation,
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
    pub category: String,
    pub kind: String,
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

fn sql_row(record: &OrgElementsIndexRecord<ParsedAnnotation>) -> OrgElementsSqlRow {
    OrgElementsSqlRow {
        ordinal: record.ordinal,
        category: record.category.as_str().to_string(),
        kind: record.kind.as_str().to_string(),
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
        "category": row.category,
        "kind": row.kind,
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
