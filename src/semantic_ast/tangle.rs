//! Safe Babel tangle and table-formula projections.

use std::collections::BTreeMap;

use super::{
    Document, Element, ElementData, ListItem, ParsedAnnotation, Section, SectionIndexSource,
    SourceBlockRecord, SourceBlockRecordKind, SourceBlockTangleMode, SourceTangleBlock,
    SourceTangleFile, SourceTangleOptions, SourceTanglePlan, SourceTangleSkip,
    SourceTangleSkipReason, Table, TableFormulaRecord,
};

impl Document<ParsedAnnotation> {
    /// Builds a safe, non-executing tangle plan from source block metadata.
    pub fn source_tangle_plan(&self, options: &SourceTangleOptions) -> SourceTanglePlan {
        let mut files: BTreeMap<String, Vec<SourceTangleBlock>> = BTreeMap::new();
        let mut skipped = Vec::new();
        for record in self.source_block_records() {
            match tangle_target(&record, options) {
                TangleDecision::File(target) => {
                    files
                        .entry(target)
                        .or_default()
                        .push(source_tangle_block(&record));
                }
                TangleDecision::Skip(reason) => skipped.push(SourceTangleSkip {
                    source: record.source,
                    name: record.name,
                    language: record.language,
                    mode: record.tangle.map(|tangle| tangle.mode),
                    reason,
                }),
            }
        }
        SourceTanglePlan {
            files: files
                .into_iter()
                .map(|(target, blocks)| SourceTangleFile { target, blocks })
                .collect(),
            skipped,
        }
    }

    /// Collects table formula side-table records for lint/index/export helpers.
    pub fn table_formula_records(&self) -> Vec<TableFormulaRecord<ParsedAnnotation>> {
        let mut records = Vec::new();
        collect_table_formula_records_in_elements(&self.children, &mut records);
        for section in &self.sections {
            collect_table_formula_records_in_section(section, &mut records);
        }
        records
    }
}

enum TangleDecision {
    File(String),
    Skip(SourceTangleSkipReason),
}

fn tangle_target(record: &SourceBlockRecord, options: &SourceTangleOptions) -> TangleDecision {
    if record.kind == SourceBlockRecordKind::InlineSource {
        return TangleDecision::Skip(SourceTangleSkipReason::InlineSource);
    }
    let Some(tangle) = record.tangle.as_ref() else {
        return TangleDecision::Skip(SourceTangleSkipReason::Disabled);
    };
    match tangle.mode {
        SourceBlockTangleMode::No => TangleDecision::Skip(SourceTangleSkipReason::Disabled),
        SourceBlockTangleMode::File => tangle
            .target
            .clone()
            .filter(|target| !target.trim().is_empty())
            .map(TangleDecision::File)
            .unwrap_or(TangleDecision::Skip(SourceTangleSkipReason::MissingTarget)),
        SourceBlockTangleMode::Yes => default_tangle_target(record, options)
            .map(TangleDecision::File)
            .unwrap_or(TangleDecision::Skip(SourceTangleSkipReason::MissingTarget)),
    }
}

fn default_tangle_target(
    record: &SourceBlockRecord,
    options: &SourceTangleOptions,
) -> Option<String> {
    let stem = options.default_stem.as_deref()?;
    let extension = record
        .language
        .as_deref()
        .and_then(language_extension)
        .unwrap_or("txt");
    Some(format!("{stem}.{extension}"))
}

fn language_extension(language: &str) -> Option<&'static str> {
    match language.to_ascii_lowercase().as_str() {
        "bash" | "sh" | "shell" => Some("sh"),
        "c" => Some("c"),
        "cpp" | "c++" => Some("cpp"),
        "css" => Some("css"),
        "emacs-lisp" | "elisp" => Some("el"),
        "go" => Some("go"),
        "html" => Some("html"),
        "java" => Some("java"),
        "javascript" | "js" => Some("js"),
        "json" => Some("json"),
        "lua" => Some("lua"),
        "nix" => Some("nix"),
        "python" | "py" => Some("py"),
        "rust" | "rs" => Some("rs"),
        "toml" => Some("toml"),
        "typescript" | "ts" => Some("ts"),
        "yaml" | "yml" => Some("yml"),
        _ => None,
    }
}

fn source_tangle_block(record: &SourceBlockRecord) -> SourceTangleBlock {
    SourceTangleBlock {
        source: record.source.clone(),
        name: record.name.clone(),
        language: record.language.clone(),
        header_args: record.normalized_header_args.clone(),
        value: record.value.clone(),
    }
}

fn collect_table_formula_records_in_section(
    section: &Section<ParsedAnnotation>,
    records: &mut Vec<TableFormulaRecord<ParsedAnnotation>>,
) {
    collect_table_formula_records_in_elements(&section.children, records);
    for subsection in &section.subsections {
        collect_table_formula_records_in_section(subsection, records);
    }
}

fn collect_table_formula_records_in_elements(
    elements: &[Element<ParsedAnnotation>],
    records: &mut Vec<TableFormulaRecord<ParsedAnnotation>>,
) {
    for element in elements {
        match &element.data {
            ElementData::Table(table) => collect_table_formula_record(element, table, records),
            ElementData::Drawer(drawer) => {
                collect_table_formula_records_in_elements(&drawer.children, records)
            }
            ElementData::List(list) => {
                collect_table_formula_records_in_list_items(&list.items, records)
            }
            ElementData::Block(block) => {
                collect_table_formula_records_in_elements(&block.children, records)
            }
            ElementData::FootnoteDef(footnote) => {
                collect_table_formula_records_in_elements(&footnote.children, records)
            }
            ElementData::Inlinetask(task) => {
                collect_table_formula_records_in_elements(&task.children, records)
            }
            ElementData::Paragraph(_)
            | ElementData::Keyword(_)
            | ElementData::BabelCall(_)
            | ElementData::Clock(_)
            | ElementData::PropertyDrawer(_)
            | ElementData::TableEl { .. }
            | ElementData::Comment(_)
            | ElementData::FixedWidth(_)
            | ElementData::Rule
            | ElementData::LatexEnvironment(_)
            | ElementData::Unknown { .. } => {}
        }
    }
}

fn collect_table_formula_records_in_list_items(
    items: &[ListItem<ParsedAnnotation>],
    records: &mut Vec<TableFormulaRecord<ParsedAnnotation>>,
) {
    for item in items {
        collect_table_formula_records_in_elements(&item.children, records);
    }
}

fn collect_table_formula_record(
    element: &Element<ParsedAnnotation>,
    table: &Table<ParsedAnnotation>,
    records: &mut Vec<TableFormulaRecord<ParsedAnnotation>>,
) {
    if table.parsed_formulas.is_empty() {
        return;
    }
    records.push(TableFormulaRecord {
        source: SectionIndexSource::from_annotation(&element.ann),
        row_count: table.rows.iter().filter(|row| !row.is_rule).count(),
        column_count: table
            .rows
            .iter()
            .map(|row| row.cells.len())
            .max()
            .unwrap_or_default(),
        formulas: table.parsed_formulas.clone(),
    });
}
