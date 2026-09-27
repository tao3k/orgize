//! Linear association of adjacent AOT graph nodes using Scheme-owned membership.

use std::{collections::HashMap, sync::OnceLock};

use gerbil_parser_rowan::GraphRecord;

include!(concat!(env!("OUT_DIR"), "/org_affiliated_keyword_p.rs"));

fn affiliated_keyword(key: &str) -> bool {
    static NAMES: OnceLock<Vec<String>> = OnceLock::new();
    let names = NAMES.get_or_init(|| {
        crate::config::org_affiliated_keyword_names()
            .iter()
            .map(|name| (*name).to_owned())
            .collect()
    });
    org_affiliated_keyword_p(key, names)
}

pub(super) fn project(records: &[GraphRecord], source: &str) -> HashMap<usize, Vec<usize>> {
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
            if record.kind == "keyword" && record.field("key").is_some_and(affiliated_keyword) {
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
