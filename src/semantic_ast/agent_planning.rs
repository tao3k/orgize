//! Agent-facing planning snapshots derived from Org agenda rows.

use super::agent_planning_model::{AgentPlanningCard, AgentPlanningQuery, AgentPlanningSnapshot};
use super::model::{Document, ParsedAnnotation};
use super::section_index_model::SectionIndexSource;
use super::task_blockers::{blockers_by_blocked_source, blockers_for_source};

impl Document<ParsedAnnotation> {
    /// Projects Org agenda rows into compact agent-facing planning cards.
    ///
    /// This is a projection over existing Org agenda semantics. It does not
    /// introduce new source syntax or mutate the parsed document.
    pub fn agent_planning_snapshot(&self, query: &AgentPlanningQuery) -> AgentPlanningSnapshot {
        let blockers_by_source = blockers_by_blocked_source(self.task_blocker_records());
        let cards = self
            .agenda_entries(&query.agenda)
            .into_iter()
            .map(|entry| {
                let source = SectionIndexSource::from_annotation(&entry.ann);
                AgentPlanningCard::from_agenda_entry(
                    entry,
                    blockers_for_source(&blockers_by_source, &source),
                )
            })
            .collect();

        AgentPlanningSnapshot { cards }
    }
}
