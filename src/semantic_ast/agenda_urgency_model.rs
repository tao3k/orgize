//! Agenda urgency scoring DTOs.

/// Explainable urgency score for one agenda or workspace agenda card.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AgendaUrgencyScore {
    pub total: i32,
    pub ingredients: Vec<AgendaUrgencyIngredient>,
}

impl AgendaUrgencyScore {
    /// Returns the score for a stable ingredient kind, or zero when absent.
    pub fn score_for(&self, kind: AgendaUrgencyIngredientKind) -> i32 {
        self.ingredients
            .iter()
            .find(|ingredient| ingredient.kind == kind)
            .map(|ingredient| ingredient.score)
            .unwrap_or(0)
    }
}

/// One named ingredient contributing to an urgency score.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaUrgencyIngredient {
    pub kind: AgendaUrgencyIngredientKind,
    pub score: i32,
    pub message: String,
}

/// Stable agenda urgency ingredient categories.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgendaUrgencyIngredientKind {
    Priority,
    Deadline,
    Scheduled,
    TodoState,
    TimeOfDay,
    Tags,
    Category,
    Occurrence,
}

impl AgendaUrgencyIngredientKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Priority => "priority",
            Self::Deadline => "deadline",
            Self::Scheduled => "scheduled",
            Self::TodoState => "todoState",
            Self::TimeOfDay => "timeOfDay",
            Self::Tags => "tags",
            Self::Category => "category",
            Self::Occurrence => "occurrence",
        }
    }
}
