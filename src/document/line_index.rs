pub(super) struct LineIndex {
    line_starts: Vec<usize>,
}

impl LineIndex {
    pub(super) fn new(source: &str) -> Self {
        let line_starts = std::iter::once(0)
            .chain(source.bytes().enumerate().filter_map(|(index, byte)| {
                (byte == b'\n' && index + 1 < source.len()).then_some(index + 1)
            }))
            .collect();
        Self { line_starts }
    }

    pub(super) fn line_for(&self, byte_index: usize) -> usize {
        self.line_starts
            .partition_point(|start| *start <= byte_index)
    }

    pub(super) fn line_for_single_offset(source: &str, byte_index: usize) -> usize {
        // Match the index: the newline belongs to its preceding line and a
        // trailing newline alone does not create a line.
        let end = byte_index.min(source.len().saturating_sub(1));
        1 + memchr::memchr_iter(b'\n', &source.as_bytes()[..end]).count()
    }
}
