//! Build-time access to feature-tagged Scheme blocks through Org Element records.

use gerbil_parser_rowan::GraphRecord;

#[rustfmt::skip]
#[path = "../../languages/org/v1/generated/parser.rs"]
mod grammar;
#[rustfmt::skip]
#[path = "../../languages/org/v1/generated/structure.rs"]
mod structure;
#[rustfmt::skip]
#[path = "../../languages/org/v1/generated/graph.rs"]
mod graph;

pub fn records(source: &str) -> Result<Vec<GraphRecord>, String> {
    let parse = gerbil_parser_rowan::parse_structural_lines(
        &grammar::LANGUAGE,
        &structure::STRUCTURE,
        source,
    )
    .map_err(|error| format!("invalid Org feature source: {error:?}"))?;
    gerbil_parser_rowan::project_syntax_graph(&grammar::LANGUAGE, &graph::GRAPH, &parse.syntax())
        .map_err(|error| format!("invalid Org Element projection: {error:?}"))
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
        .filter(|record| record.field("language") == Some("scheme"))
        .filter(|record| {
            record
                .field("header")
                .is_some_and(|header| header.split_ascii_whitespace().any(|part| part == feature))
        })
        .collect();
    let [block] = blocks.as_slice() else {
        return Err(format!("expected exactly one scheme {feature} block"));
    };
    block
        .field("body")
        .ok_or_else(|| format!("scheme {feature} block has no body"))
}
