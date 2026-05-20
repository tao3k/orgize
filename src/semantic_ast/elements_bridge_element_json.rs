//! Element-level JSON projection for the Org elements bridge.

use serde_json::{Value, json};

use super::{
    Block, BlockKind, BlockLine, BlockLineNumberMode, BlockLineNumbering, BlockSwitches, Checkbox,
    Element, ElementData, ListItem, ListType, ParsedAnnotation, TableColumnAlignment, TableFormula,
    TableFormulaReferenceKind, TableRow,
};

pub(super) fn elements_json(elements: &[Element<ParsedAnnotation>]) -> Vec<Value> {
    elements.iter().map(element_json).collect()
}

fn element_json(element: &Element<ParsedAnnotation>) -> Value {
    let (kind, value) = match &element.data {
        ElementData::Paragraph(objects) => (
            "paragraph",
            json!({ "objects": super::elements_bridge_object_json::objects_json(objects) }),
        ),
        ElementData::Keyword(keyword) => (
            "keyword",
            json!({ "keyword": super::elements_bridge_json::keyword_json(keyword) }),
        ),
        ElementData::BabelCall(keyword) => (
            "babel-call",
            json!({ "keyword": super::elements_bridge_json::keyword_json(keyword) }),
        ),
        ElementData::Clock(clock) => (
            "clock",
            json!({
                "raw": &clock.raw,
                "timestamp": clock.value.as_ref().map(super::elements_bridge_object_json::timestamp_json),
                "duration": &clock.duration,
                "parsedDuration": clock
                    .parsed_duration
                    .as_ref()
                    .map(super::elements_bridge_json::duration_json),
            }),
        ),
        ElementData::Drawer(drawer) => (
            "drawer",
            json!({
                "name": &drawer.name,
                "raw": &drawer.raw,
                "elements": elements_json(&drawer.children),
            }),
        ),
        ElementData::PropertyDrawer(properties) => (
            "property-drawer",
            json!({ "properties": super::elements_bridge_json::properties_json(properties) }),
        ),
        ElementData::List(list) => (
            "plain-list",
            json!({
                "listType": list_type(list.list_type),
                "items": list.items.iter().map(list_item_json).collect::<Vec<_>>(),
            }),
        ),
        ElementData::Table(table) => (
            "table",
            json!({
                "columnAlignments": table
                    .column_alignments
                    .iter()
                    .map(|alignment| alignment.map(table_column_alignment))
                    .collect::<Vec<_>>(),
                "rows": table.rows.iter().map(table_row_json).collect::<Vec<_>>(),
                "formulas": table
                    .formulas
                    .iter()
                    .map(super::elements_bridge_json::keyword_json)
                    .collect::<Vec<_>>(),
                "parsedFormulas": table
                    .parsed_formulas
                    .iter()
                    .map(table_formula_json)
                    .collect::<Vec<_>>(),
            }),
        ),
        ElementData::TableEl { raw } => ("table.el", json!({ "raw": raw })),
        ElementData::Block(block) => (block_kind(block), block_json(block)),
        ElementData::FootnoteDef(footnote) => (
            "footnote-definition",
            json!({
                "label": &footnote.label,
                "elements": elements_json(&footnote.children),
            }),
        ),
        ElementData::Inlinetask(task) => (
            "inlinetask",
            json!({
                "level": task.level,
                "todo": task.todo.as_ref().map(|todo| todo.name.as_str()),
                "todoState": task.todo.as_ref().map(super::elements_bridge_json::todo_state),
                "priority": super::elements_bridge_json::priority_json(&task.priority),
                "title": task.raw_title.trim_end(),
                "titleObjects": super::elements_bridge_object_json::objects_json(&task.title),
                "tags": &task.tags,
                "planning": super::elements_bridge_json::planning_json(&task.planning),
                "properties": super::elements_bridge_json::properties_json(&task.properties),
                "elements": elements_json(&task.children),
                "end": task.end.as_ref().map(|end| {
                    json!({
                        "source": super::elements_bridge_json::annotation_json(&end.ann),
                        "level": end.level,
                        "raw": &end.raw,
                    })
                }),
            }),
        ),
        ElementData::Comment(raw) => ("comment", json!({ "raw": raw })),
        ElementData::FixedWidth(fixed) => (
            "fixed-width",
            json!({
                "value": &fixed.value,
                "normalizedValue": fixed.normalized_value(),
                "lines": fixed.lines.iter().map(block_line_json).collect::<Vec<_>>(),
            }),
        ),
        ElementData::Rule => ("horizontal-rule", json!({})),
        ElementData::LatexEnvironment(raw) => ("latex-environment", json!({ "raw": raw })),
        ElementData::Unknown { kind, raw } => (
            "unknown",
            json!({
                "syntaxKind": kind.as_str(),
                "raw": raw,
            }),
        ),
    };
    with_element_base(element, kind, value)
}

fn with_element_base(element: &Element<ParsedAnnotation>, kind: &str, mut value: Value) -> Value {
    if let Value::Object(map) = &mut value {
        map.insert(
            "source".to_string(),
            super::elements_bridge_json::annotation_json(&element.ann),
        );
        map.insert("kind".to_string(), json!(kind));
        map.insert(
            "affiliatedKeywords".to_string(),
            json!(
                element
                    .affiliated_keywords
                    .iter()
                    .map(super::elements_bridge_json::keyword_json)
                    .collect::<Vec<_>>()
            ),
        );
    }
    value
}

fn block_json(block: &Block<ParsedAnnotation>) -> Value {
    json!({
        "blockKind": block_kind(block),
        "name": &block.name,
        "language": &block.language,
        "switches": &block.switches,
        "switchOptions": block_switches_json(&block.switch_options),
        "lineNumbering": block.line_numbering.as_ref().map(block_line_numbering_json),
        "preserveIndentation": block.preserve_indentation,
        "parameters": &block.parameters,
        "headerArgs": block
            .header_args
            .iter()
            .map(super::elements_bridge_json::block_header_arg_json)
            .collect::<Vec<_>>(),
        "codeRefs": block
            .code_refs
            .iter()
            .map(|code_ref| json!({
                "line": code_ref.line,
                "column": code_ref.column,
                "endColumn": code_ref.end_column,
                "name": &code_ref.name,
                "raw": &code_ref.raw,
            }))
            .collect::<Vec<_>>(),
        "value": &block.value,
        "normalizedValue": block.normalized_value(),
        "valueWithoutCodeRefs": block.value_without_code_refs(),
        "normalizedValueWithoutCodeRefs": block.normalized_value_without_code_refs(),
        "lines": block.lines.iter().map(block_line_json).collect::<Vec<_>>(),
        "elements": elements_json(&block.children),
    })
}

fn block_line_json(line: &BlockLine<ParsedAnnotation>) -> Value {
    json!({
        "source": super::elements_bridge_json::annotation_json(&line.ann),
        "number": line.number,
        "sourceText": &line.source,
        "value": &line.value,
        "normalizedValue": &line.normalized_value,
        "valueWithoutCodeRef": &line.value_without_code_ref,
        "normalizedValueWithoutCodeRef": &line.normalized_value_without_code_ref,
        "removedIndent": line.removed_indent,
        "lineEnding": &line.line_ending,
        "codeRef": line.code_ref.as_ref().map(|code_ref| json!({
            "line": code_ref.line,
            "column": code_ref.column,
            "endColumn": code_ref.end_column,
            "name": &code_ref.name,
            "raw": &code_ref.raw,
        })),
    })
}

fn block_switches_json(switches: &BlockSwitches) -> Value {
    json!({
        "raw": &switches.raw,
        "lineNumbering": switches.line_numbering.as_ref().map(block_line_numbering_json),
        "preserveIndentation": switches.preserve_indentation,
        "keepLabels": switches.keep_labels,
        "removeLabels": switches.remove_labels,
        "labelFormat": &switches.label_format,
    })
}

fn block_line_numbering_json(numbering: &BlockLineNumbering) -> Value {
    json!({
        "mode": match numbering.mode {
            BlockLineNumberMode::New => "new",
            BlockLineNumberMode::Continued => "continued",
        },
        "start": numbering.start,
    })
}

fn list_item_json(item: &ListItem<ParsedAnnotation>) -> Value {
    json!({
        "source": super::elements_bridge_json::annotation_json(&item.ann),
        "bullet": &item.bullet,
        "counter": &item.counter,
        "checkbox": item.checkbox.map(checkbox),
        "tag": super::elements_bridge_object_json::objects_json(&item.tag),
        "elements": elements_json(&item.children),
    })
}

fn table_row_json(row: &TableRow<ParsedAnnotation>) -> Value {
    json!({
        "source": super::elements_bridge_json::annotation_json(&row.ann),
        "isRule": row.is_rule,
        "cells": row
            .cells
            .iter()
            .map(|cell| json!({
                "source": super::elements_bridge_json::annotation_json(&cell.ann),
                "objects": super::elements_bridge_object_json::objects_json(&cell.objects),
            }))
            .collect::<Vec<_>>(),
    })
}

fn table_formula_json(formula: &TableFormula<ParsedAnnotation>) -> Value {
    json!({
        "source": super::elements_bridge_json::annotation_json(&formula.ann),
        "raw": &formula.raw,
        "assignments": formula
            .assignments
            .iter()
            .map(|assignment| json!({
                "raw": &assignment.raw,
                "lhs": &assignment.lhs,
                "rhs": &assignment.rhs,
                "flags": &assignment.flags,
                "references": assignment
                    .references
                    .iter()
                    .map(|reference| json!({
                        "raw": &reference.raw,
                        "kind": table_formula_reference_kind(reference.kind),
                    }))
                    .collect::<Vec<_>>(),
            }))
            .collect::<Vec<_>>(),
    })
}

fn block_kind(block: &Block<ParsedAnnotation>) -> &str {
    match &block.kind {
        BlockKind::Source => "src-block",
        BlockKind::Example => "example-block",
        BlockKind::Export => "export-block",
        BlockKind::Quote => "quote-block",
        BlockKind::Verse => "verse-block",
        BlockKind::Center => "center-block",
        BlockKind::Comment => "comment-block",
        BlockKind::Dynamic => "dynamic-block",
        BlockKind::Special(_) => "special-block",
    }
}

fn list_type(list_type: ListType) -> &'static str {
    match list_type {
        ListType::Ordered => "ordered",
        ListType::Unordered => "unordered",
        ListType::Descriptive => "descriptive",
    }
}

fn checkbox(checkbox: Checkbox) -> &'static str {
    match checkbox {
        Checkbox::On => "on",
        Checkbox::Off => "off",
        Checkbox::Trans => "trans",
    }
}

fn table_column_alignment(alignment: TableColumnAlignment) -> &'static str {
    match alignment {
        TableColumnAlignment::Left => "left",
        TableColumnAlignment::Center => "center",
        TableColumnAlignment::Right => "right",
    }
}

fn table_formula_reference_kind(kind: TableFormulaReferenceKind) -> &'static str {
    match kind {
        TableFormulaReferenceKind::Field => "field",
        TableFormulaReferenceKind::Remote => "remote",
        TableFormulaReferenceKind::Row => "row",
    }
}
