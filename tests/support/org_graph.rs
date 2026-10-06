use gerbil_parser_rowan::GraphRecord;

pub(crate) fn assert_graph_integrity(source: &str, records: &[GraphRecord]) {
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
