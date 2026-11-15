use std::borrow::Cow;

use super::GraphRecord;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GraphIndexError {
    RecordCount,
    ScopeMissing,
    TargetMissing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GraphRelation {
    Any,
    At,
    ChildOf,
    DescendantOf,
}

#[derive(Debug)]
pub(crate) struct GraphIndex(pub Vec<usize>);

impl GraphIndex {
    pub(crate) fn subtree_end(&self, id: usize) -> Option<usize> {
        self.0.get(id).copied()
    }

    pub(crate) fn select(
        &self,
        records: &[GraphRecord],
        scope: usize,
        kind: &str,
        relation: GraphRelation,
        targets: &[usize],
        mut predicate: impl FnMut(&GraphRecord) -> bool,
    ) -> Result<Vec<usize>, GraphIndexError> {
        let end = self
            .subtree_end(scope)
            .ok_or(GraphIndexError::ScopeMissing)?;
        if records.len() != self.0.len() {
            return Err(GraphIndexError::RecordCount);
        }
        if targets.iter().any(|&id| id >= records.len()) {
            return Err(GraphIndexError::TargetMissing);
        }
        let mut targets = Cow::Borrowed(targets);
        if !targets.is_sorted() {
            targets.to_mut().sort_unstable();
        }
        let mut eligible = targets.iter().copied().peekable();
        let mut descendant_end = 0;
        let mut found = Vec::new();
        for record in &records[scope..end] {
            let id = record.id;
            let related = match relation {
                GraphRelation::Any => true,
                GraphRelation::At => targets.binary_search(&id).is_ok(),
                GraphRelation::ChildOf => record
                    .parent_id
                    .is_some_and(|p| targets.binary_search(&p).is_ok()),
                GraphRelation::DescendantOf => {
                    while eligible.peek().is_some_and(|&target| target < id) {
                        descendant_end = descendant_end.max(self.0[eligible.next().unwrap()]);
                    }
                    id < descendant_end
                }
            };
            if related && record.kind == kind && predicate(record) {
                found.push(id);
            }
        }
        Ok(found)
    }
}
