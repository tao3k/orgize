//! Agenda query filtering and Org Agenda-style match expression evaluation.

use super::agenda_match::{
    AgendaMatchOperator, AgendaMatchPredicate, AgendaMatchQuery, AgendaMatchTerm, AgendaMatchValue,
};
use super::agenda_model::{is_done_keyword, AgendaCategory, AgendaQuery};
use super::model::Section;

pub(crate) fn section_matches_query<A>(
    section: &Section<A>,
    query: &AgendaQuery,
    category: Option<&AgendaCategory>,
) -> bool {
    if section.is_comment && !query.include_comments {
        return false;
    }
    if !query.include_done && is_done_keyword(&section.todo) {
        return false;
    }
    if !query.include_archived && has_tag(&section.effective_tags, "ARCHIVE") {
        return false;
    }
    if query
        .required_tags
        .iter()
        .any(|required| !has_tag(&section.effective_tags, required))
    {
        return false;
    }
    if query
        .excluded_tags
        .iter()
        .any(|excluded| has_tag(&section.effective_tags, excluded))
    {
        return false;
    }
    if let Some(match_query) = &query.match_query {
        if !agenda_match_query_matches(section, category, match_query) {
            return false;
        }
    }
    true
}

fn agenda_match_query_matches<A>(
    section: &Section<A>,
    category: Option<&AgendaCategory>,
    match_query: &AgendaMatchQuery,
) -> bool {
    match_query.clauses.iter().any(|clause| {
        clause
            .terms
            .iter()
            .all(|term| agenda_match_term_matches(section, category, term))
    })
}

fn agenda_match_term_matches<A>(
    section: &Section<A>,
    category: Option<&AgendaCategory>,
    term: &AgendaMatchTerm,
) -> bool {
    let matched = match &term.predicate {
        AgendaMatchPredicate::Tag(tag) => has_tag(&section.effective_tags, tag),
        AgendaMatchPredicate::Property {
            key,
            operator,
            value,
        } => agenda_match_property_matches(section, category, key, *operator, value),
    };

    if term.positive {
        matched
    } else {
        !matched
    }
}

fn agenda_match_property_matches<A>(
    section: &Section<A>,
    category: Option<&AgendaCategory>,
    key: &str,
    operator: AgendaMatchOperator,
    expected: &AgendaMatchValue,
) -> bool {
    let Some(actual) = agenda_match_property_value(section, category, key) else {
        return false;
    };
    compare_agenda_match_values(actual.as_str(), operator, expected)
}

fn agenda_match_property_value<A>(
    section: &Section<A>,
    category: Option<&AgendaCategory>,
    key: &str,
) -> Option<String> {
    if key.eq_ignore_ascii_case("TODO") {
        return section.todo.as_ref().map(|todo| todo.name.clone());
    }
    if key.eq_ignore_ascii_case("LEVEL") {
        return Some(section.level.to_string());
    }
    if key.eq_ignore_ascii_case("PRIORITY") {
        return section.priority.clone();
    }
    if key.eq_ignore_ascii_case("CATEGORY") {
        return category.map(|category| category.as_str().to_string());
    }
    if key.eq_ignore_ascii_case("SCHEDULED") {
        return section
            .planning
            .scheduled
            .as_ref()
            .map(|timestamp| timestamp.raw.clone());
    }
    if key.eq_ignore_ascii_case("DEADLINE") {
        return section
            .planning
            .deadline
            .as_ref()
            .map(|timestamp| timestamp.raw.clone());
    }
    if key.eq_ignore_ascii_case("CLOSED") {
        return section
            .planning
            .closed
            .as_ref()
            .map(|timestamp| timestamp.raw.clone());
    }
    if key.eq_ignore_ascii_case("TAGS") {
        return Some(section.effective_tags.join(":"));
    }

    section
        .properties
        .iter()
        .find(|property| property.key.eq_ignore_ascii_case(key))
        .map(|property| property.value.clone())
}

fn compare_agenda_match_values(
    actual: &str,
    operator: AgendaMatchOperator,
    expected: &AgendaMatchValue,
) -> bool {
    if expected.is_pattern() {
        return match operator {
            AgendaMatchOperator::Equal => actual.contains(expected.as_str()),
            AgendaMatchOperator::NotEqual => !actual.contains(expected.as_str()),
            AgendaMatchOperator::Less
            | AgendaMatchOperator::LessOrEqual
            | AgendaMatchOperator::Greater
            | AgendaMatchOperator::GreaterOrEqual => false,
        };
    }

    if let (Ok(left), Ok(right)) = (actual.parse::<f64>(), expected.as_str().parse::<f64>()) {
        return match operator {
            AgendaMatchOperator::Equal => left == right,
            AgendaMatchOperator::NotEqual => left != right,
            AgendaMatchOperator::Less => left < right,
            AgendaMatchOperator::LessOrEqual => left <= right,
            AgendaMatchOperator::Greater => left > right,
            AgendaMatchOperator::GreaterOrEqual => left >= right,
        };
    }

    match operator {
        AgendaMatchOperator::Equal => actual == expected.as_str(),
        AgendaMatchOperator::NotEqual => actual != expected.as_str(),
        AgendaMatchOperator::Less => actual < expected.as_str(),
        AgendaMatchOperator::LessOrEqual => actual <= expected.as_str(),
        AgendaMatchOperator::Greater => actual > expected.as_str(),
        AgendaMatchOperator::GreaterOrEqual => actual >= expected.as_str(),
    }
}

fn has_tag(tags: &[String], needle: &str) -> bool {
    tags.iter().any(|tag| tag.eq_ignore_ascii_case(needle))
}
