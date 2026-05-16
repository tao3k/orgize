//! Explainable Org Agenda-style urgency scoring.

use super::{
    AgendaDeadlineState, AgendaEntry, AgendaOccurrence, AgendaScheduleState,
    AgendaUrgencyIngredient, AgendaUrgencyIngredientKind, AgendaUrgencyScore,
};

pub(crate) fn agenda_urgency_score<A>(entry: &AgendaEntry<A>) -> AgendaUrgencyScore {
    let mut ingredients = Vec::new();
    push_priority(entry, &mut ingredients);
    push_deadline(entry, &mut ingredients);
    push_scheduled(entry, &mut ingredients);
    push_todo(entry, &mut ingredients);
    push_time(entry, &mut ingredients);
    push_tags(entry, &mut ingredients);
    push_category(entry, &mut ingredients);
    push_occurrence(entry, &mut ingredients);
    let total = ingredients.iter().map(|ingredient| ingredient.score).sum();
    AgendaUrgencyScore { total, ingredients }
}

fn push_priority<A>(entry: &AgendaEntry<A>, ingredients: &mut Vec<AgendaUrgencyIngredient>) {
    let score = entry.priority.org_priority_score().unwrap_or(0);
    ingredients.push(AgendaUrgencyIngredient {
        kind: AgendaUrgencyIngredientKind::Priority,
        score,
        message: format!("priority {}", entry.priority.effective_text()),
    });
}

fn push_deadline<A>(entry: &AgendaEntry<A>, ingredients: &mut Vec<AgendaUrgencyIngredient>) {
    let (score, message) = match entry.deadline {
        Some(AgendaDeadlineState::Overdue { days_overdue }) => (
            4000 + i32::try_from(days_overdue).unwrap_or(i32::MAX / 2) * 10,
            format!("deadline overdue by {days_overdue} day(s)"),
        ),
        Some(AgendaDeadlineState::Due) => (3000, "deadline due today".to_string()),
        Some(AgendaDeadlineState::Warning { days_until }) => (
            1000 - i32::try_from(days_until).unwrap_or(1000).min(1000),
            format!("deadline warning with {days_until} day(s) remaining"),
        ),
        None => (0, "no deadline urgency".to_string()),
    };
    ingredients.push(AgendaUrgencyIngredient {
        kind: AgendaUrgencyIngredientKind::Deadline,
        score,
        message,
    });
}

fn push_scheduled<A>(entry: &AgendaEntry<A>, ingredients: &mut Vec<AgendaUrgencyIngredient>) {
    let (score, message) = match entry.scheduled {
        Some(AgendaScheduleState::PastDue { days_overdue }) => (
            1500 + i32::try_from(days_overdue).unwrap_or(i32::MAX / 2) * 5,
            format!("scheduled date passed by {days_overdue} day(s)"),
        ),
        Some(AgendaScheduleState::OnDate) => (500, "scheduled today".to_string()),
        Some(AgendaScheduleState::Delayed { days_delayed }) => (
            -i32::try_from(days_delayed).unwrap_or(0).min(500),
            format!("scheduled is delayed by {days_delayed} day(s)"),
        ),
        None => (0, "no scheduled urgency".to_string()),
    };
    ingredients.push(AgendaUrgencyIngredient {
        kind: AgendaUrgencyIngredientKind::Scheduled,
        score,
        message,
    });
}

fn push_todo<A>(entry: &AgendaEntry<A>, ingredients: &mut Vec<AgendaUrgencyIngredient>) {
    let (score, message) = entry
        .todo
        .as_ref()
        .map(|todo| (250, format!("todo keyword {}", todo.name)))
        .unwrap_or_else(|| (0, "no todo keyword".to_string()));
    ingredients.push(AgendaUrgencyIngredient {
        kind: AgendaUrgencyIngredientKind::TodoState,
        score,
        message,
    });
}

fn push_time<A>(entry: &AgendaEntry<A>, ingredients: &mut Vec<AgendaUrgencyIngredient>) {
    let (score, message) = entry
        .time
        .map(|time| {
            let minute_of_day = i32::from(time.hour) * 60 + i32::from(time.minute);
            (
                300 - minute_of_day / 12,
                format!("time {:02}:{:02}", time.hour, time.minute),
            )
        })
        .unwrap_or_else(|| (0, "untimed".to_string()));
    ingredients.push(AgendaUrgencyIngredient {
        kind: AgendaUrgencyIngredientKind::TimeOfDay,
        score,
        message,
    });
}

fn push_tags<A>(entry: &AgendaEntry<A>, ingredients: &mut Vec<AgendaUrgencyIngredient>) {
    let urgent_tags = entry
        .effective_tags
        .iter()
        .filter(|tag| {
            tag.eq_ignore_ascii_case("urgent")
                || tag.eq_ignore_ascii_case("now")
                || tag.eq_ignore_ascii_case("important")
        })
        .count();
    let score = i32::try_from(urgent_tags).unwrap_or(0) * 200;
    ingredients.push(AgendaUrgencyIngredient {
        kind: AgendaUrgencyIngredientKind::Tags,
        score,
        message: if urgent_tags == 0 {
            "no urgency tags".to_string()
        } else {
            format!("{urgent_tags} urgency tag(s)")
        },
    });
}

fn push_category<A>(entry: &AgendaEntry<A>, ingredients: &mut Vec<AgendaUrgencyIngredient>) {
    ingredients.push(AgendaUrgencyIngredient {
        kind: AgendaUrgencyIngredientKind::Category,
        score: 0,
        message: entry
            .category
            .as_ref()
            .map(|category| format!("category {}", category.as_str()))
            .unwrap_or_else(|| "no category".to_string()),
    });
}

fn push_occurrence<A>(entry: &AgendaEntry<A>, ingredients: &mut Vec<AgendaUrgencyIngredient>) {
    let (score, message) = match entry.occurrence {
        AgendaOccurrence::Source => (0, "source occurrence".to_string()),
        AgendaOccurrence::Repeater { index } => (
            -i32::try_from(index).unwrap_or(0).min(1000),
            format!("repeater occurrence {index}"),
        ),
    };
    ingredients.push(AgendaUrgencyIngredient {
        kind: AgendaUrgencyIngredientKind::Occurrence,
        score,
        message,
    });
}
