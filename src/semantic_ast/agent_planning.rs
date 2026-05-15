//! Agent-facing planning snapshots derived from Org agenda rows.

use super::agent_planning_model::{AgentPlanningCard, AgentPlanningQuery, AgentPlanningSnapshot};
use super::model::{Document, ParsedAnnotation};

impl Document<ParsedAnnotation> {
    /// Projects Org agenda rows into compact agent-facing planning cards.
    ///
    /// This is a projection over existing Org agenda semantics. It does not
    /// introduce new source syntax or mutate the parsed document.
    pub fn agent_planning_snapshot(&self, query: &AgentPlanningQuery) -> AgentPlanningSnapshot {
        let cards = self
            .agenda_entries(&query.agenda)
            .into_iter()
            .map(AgentPlanningCard::from_agenda_entry)
            .collect();

        AgentPlanningSnapshot { cards }
    }
}
