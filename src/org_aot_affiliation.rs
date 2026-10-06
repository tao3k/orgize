//! Linear association of adjacent AOT graph nodes using Scheme-owned membership.

use std::collections::HashMap;

use gerbil_parser_rowan::GraphRecord;

pub(super) fn project(records: &[GraphRecord], source: &str) -> HashMap<usize, Vec<usize>> {
    let keywords = records
        .iter()
        .filter(|record| record.kind == "keyword")
        .collect::<Vec<_>>();
    let names = crate::config::org_affiliated_keyword_names();
    let mut fields = vec![names.len().to_string()];
    fields.extend(names.iter().map(|name| (*name).to_owned()));
    fields.push(keywords.len().to_string());
    fields.extend(
        keywords
            .iter()
            .map(|record| record.field("key").unwrap_or_default().to_owned()),
    );
    let refs = fields.iter().map(String::as_str).collect::<Vec<_>>();
    let rows =
        super::native_semantic_rows(19, &refs).expect("initialized native affiliation batch");
    assert_eq!(rows.len(), keywords.len(), "native affiliation count");
    let admitted = keywords
        .into_iter()
        .zip(rows)
        .map(|(record, row)| {
            assert_eq!(row.len(), 1, "native affiliation arity");
            let flag = match row[0].as_str() {
                "true" => true,
                "false" => false,
                _ => panic!("native affiliation flag"),
            };
            (record.id, flag)
        })
        .collect::<HashMap<_, _>>();
    let mut by_target = HashMap::new();
    for parent in records {
        let mut pending = Vec::new();
        let mut preceding_end = None;
        for &id in &parent.child_ids {
            let record = &records[id];
            let start = usize::from(record.range.start());
            if let Some(end) = preceding_end
                && source.get(end..start).is_none_or(|gap| gap.contains('\n'))
            {
                pending.clear();
            }
            if admitted.get(&record.id).copied().unwrap_or(false) {
                pending.push(id);
            } else if record.category == "element"
                && record.kind != "headline"
                && !pending.is_empty()
            {
                by_target.insert(id, std::mem::take(&mut pending));
            } else {
                pending.clear();
            }
            preceding_end = Some(usize::from(record.range.end()));
        }
    }
    by_target
}
