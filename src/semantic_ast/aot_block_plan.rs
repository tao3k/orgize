//! Bounded request-local framing; all line semantics remain in Scheme.

use super::aot_block_switches::project_block_switches;
use crate::org_aot::OrgAotDocument;
use std::collections::HashMap;

pub(super) fn block_document_plan(
    document: &OrgAotDocument,
    source: &str,
) -> HashMap<usize, Vec<Vec<String>>> {
    // This bounds records/input framing per crossing, not owner service time.
    // A single oversized block retains the existing native line owner.
    const MAX_BLOCKS: usize = 32;
    const TARGET_BYTES: usize = 256 * 1024;
    let width = document.config().src_tab_width.to_string();
    let mut facts = HashMap::new();
    let mut ids = Vec::new();
    let mut fields = Vec::new();
    let mut bytes: usize = 0;
    for record in document.records() {
        let fixed = record.kind == "fixed-width";
        if !fixed
            && !matches!(
                record.kind,
                "src-block"
                    | "example-block"
                    | "export-block"
                    | "quote-block"
                    | "verse-block"
                    | "center-block"
                    | "comment-block"
                    | "dynamic-block"
                    | "special-block"
            )
        {
            continue;
        }
        let value = if fixed {
            record.values("value").collect::<String>()
        } else {
            record.field("body").unwrap_or_default().to_owned()
        };
        let range = if fixed {
            Some(record.range)
        } else {
            record
                .field_range("raw-body")
                .or_else(|| record.field_range("body"))
        };
        let raw = range
            .map(|range| {
                source
                    .get(usize::from(range.start())..usize::from(range.end()))
                    .expect("native block source range")
            })
            .unwrap_or(&value)
            .to_owned();
        let (_, switches) = project_block_switches(record, source);
        let input = [
            value,
            raw,
            switches
                .label_format
                .as_deref()
                .unwrap_or("(ref:%s)")
                .to_owned(),
            width.clone(),
            (!fixed && switches.preserve_indentation).to_string(),
        ];
        let size = input.iter().map(|field| field.len() + 4).sum::<usize>();
        if !ids.is_empty() && (ids.len() == MAX_BLOCKS || bytes.saturating_add(size) > TARGET_BYTES)
        {
            admit_batch(&mut facts, &mut ids, &mut fields);
            bytes = 0;
        }
        ids.push(record.id);
        fields.extend(input);
        bytes += size;
    }
    if !ids.is_empty() {
        admit_batch(&mut facts, &mut ids, &mut fields);
    }
    facts
}

fn admit_batch(
    facts: &mut HashMap<usize, Vec<Vec<String>>>,
    ids: &mut Vec<usize>,
    fields: &mut Vec<String>,
) {
    let refs = fields.iter().map(String::as_str).collect::<Vec<_>>();
    let mut rows = super::org_values::rows("block-document-plan", &refs).into_iter();
    for id in ids.drain(..) {
        let header = rows.next().expect("native block plan header");
        let [tag, count]: [String; 2] = header.try_into().expect("native block plan header arity");
        assert_eq!(tag, "block", "native block plan header tag");
        let count = super::block_metadata::native_number(&count);
        let block = rows.by_ref().take(count).collect::<Vec<_>>();
        assert_eq!(block.len(), count, "truncated native block plan");
        assert!(
            facts.insert(id, block).is_none(),
            "duplicate native block plan input"
        );
    }
    assert!(rows.next().is_none(), "trailing native block plan rows");
    fields.clear();
}
