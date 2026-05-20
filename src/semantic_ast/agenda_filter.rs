//! Agenda query filtering and Org Agenda-style match expression evaluation.

use super::agenda_match::{
    AgendaMatchOperator, AgendaMatchPredicate, AgendaMatchQuery, AgendaMatchTerm, AgendaMatchValue,
};
use super::agenda_model::{AgendaCategory, AgendaQuery, is_done_keyword};
use super::model::Section;
use super::special_properties::{SpecialPropertyContext, special_property_value};

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
    if !query.include_archived && section.archive.archived {
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
    if let Some(match_query) = &query.match_query
        && !section_matches_agenda_match(
            section,
            category,
            query.source_file.as_deref(),
            match_query,
        )
    {
        return false;
    }
    true
}

pub(crate) fn section_matches_agenda_match<A>(
    section: &Section<A>,
    category: Option<&AgendaCategory>,
    source_file: Option<&str>,
    match_query: &AgendaMatchQuery,
) -> bool {
    match_query.clauses.iter().any(|clause| {
        clause
            .terms
            .iter()
            .all(|term| agenda_match_term_matches(section, category, source_file, term))
    })
}

fn agenda_match_term_matches<A>(
    section: &Section<A>,
    category: Option<&AgendaCategory>,
    source_file: Option<&str>,
    term: &AgendaMatchTerm,
) -> bool {
    let matched = match &term.predicate {
        AgendaMatchPredicate::Tag(tag) => has_tag(&section.effective_tags, tag),
        AgendaMatchPredicate::Property {
            key,
            operator,
            value,
        } => agenda_match_property_matches(section, category, source_file, key, *operator, value),
    };

    if term.positive { matched } else { !matched }
}

fn agenda_match_property_matches<A>(
    section: &Section<A>,
    category: Option<&AgendaCategory>,
    source_file: Option<&str>,
    key: &str,
    operator: AgendaMatchOperator,
    expected: &AgendaMatchValue,
) -> bool {
    let Some(actual) = agenda_match_property_value(section, category, source_file, key) else {
        return false;
    };
    compare_agenda_match_values(actual.as_str(), operator, expected)
}

fn agenda_match_property_value<A>(
    section: &Section<A>,
    category: Option<&AgendaCategory>,
    source_file: Option<&str>,
    key: &str,
) -> Option<String> {
    let context = SpecialPropertyContext::new(category.map(AgendaCategory::as_str), source_file);
    if let Some(value) = special_property_value(section, context, key) {
        return Some(value);
    }

    section
        .effective_properties
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

    if let (Some(left), Some(right)) = (
        super::special_properties::timestamp_sort_key(actual),
        super::special_properties::timestamp_sort_key(expected.as_str()),
    ) {
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
