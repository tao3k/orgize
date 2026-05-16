//! Date tree projection records.

use super::{AgendaDate, SectionIndexSource};

/// One date-organized headline recognized from an Org datetree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateTreeEntry {
    pub source: SectionIndexSource,
    pub date: AgendaDate,
    pub year_title: String,
    pub month_title: String,
    pub day_title: String,
    pub outline_path: Vec<String>,
}
