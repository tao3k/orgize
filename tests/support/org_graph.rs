use gerbil_parser_runtime::GraphRecord;

pub(crate) fn assert_graph_integrity(source: &str, records: &[GraphRecord]) {
    let root = records.first().expect("document graph has a root");
    assert_eq!(root.kind, "org-data");
    let source_field = root
        .fields
        .iter()
        .find(|field| field.name == "source-text")
        .expect("Scheme graph declaration retains the root source text");
    assert_eq!(source_field.value, source);
    assert_eq!(usize::from(source_field.range.start()), 0);
    assert_eq!(usize::from(source_field.range.end()), source.len());
    for (index, record) in records.iter().enumerate() {
        assert_eq!(record.id, index);
        assert!(usize::from(record.range.end()) <= source.len());
        if let Some(parent_id) = record.parent_id {
            let parent = &records[parent_id];
            assert!(parent.child_ids.contains(&index));
            assert!(parent.range.start() <= record.range.start());
            assert!(record.range.end() <= parent.range.end());
        }
        for &child_id in &record.child_ids {
            assert_eq!(records[child_id].parent_id, Some(index));
        }
    }
}
