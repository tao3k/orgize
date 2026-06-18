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
}
