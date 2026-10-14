//! Clock values and source-backed timestamp points from Scheme graph records.

use gerbil_parser_runtime::GraphRecord;
use gerbil_parser_runtime::TextRange;

use super::{
    Clock, GraphProjector, OrgDuration, Timestamp, project_timestamp, timestamp_point_ranges,
};

impl GraphProjector<'_> {
    pub(super) fn timestamp(&self, id: usize) -> Timestamp {
        let record = self.record(id);
        project_timestamp(record, self.raw(record.range))
    }

    pub(super) fn clock_first_point_range(&self, record: &GraphRecord) -> Option<TextRange> {
        record
            .child_ids
            .iter()
            .map(|&child| self.record(child))
            .find(|child| child.kind == "timestamp")
            .and_then(|child| timestamp_point_ranges(child).0)
    }

    pub(super) fn clock(&self, record: &GraphRecord) -> Clock {
        let duration = record.field("duration").map(str::to_owned);
        Clock {
            value: record
                .child_ids
                .iter()
                .copied()
                .find(|&child| self.record(child).kind == "timestamp")
                .map(|child| self.timestamp(child)),
            parsed_duration: duration.as_deref().and_then(OrgDuration::parse),
            duration,
            raw: self.raw(record.range).to_owned(),
        }
    }
}
