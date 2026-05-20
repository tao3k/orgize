//! Document-local sparse-tree projections over existing section index records.

use super::agenda_match::{
    AgendaMatchClause, AgendaMatchOperator, AgendaMatchPredicate, AgendaMatchQuery,
    AgendaMatchTerm, AgendaMatchValue,
};
use super::sparse_tree_model::{
    SparseTreeCard, SparseTreeMatch, SparseTreeMatchKind, SparseTreeProjection, SparseTreeQuery,
    SparseTreeReceipt, SparseTreeReceiptKind, SparseTreeSkip, SparseTreeSkipReason, truncate_text,
};
use super::tag_vocabulary::TagMatcher;
use super::{
    Document, ParsedAnnotation, SectionIndexProperty, SectionIndexRecord, SectionIndexSource,
    SectionIndexSpecialProperty,
};

const PREVIEW_CHARS: usize = 240;

impl Document<ParsedAnnotation> {
    /// Projects one parsed document into sparse-tree cards for agent/search
    /// consumers.
    ///
    /// This API is document-local. It does not rank, persist, or resolve
    /// cross-file targets.
    pub fn sparse_tree_projection(&self, query: &SparseTreeQuery) -> SparseTreeProjection {
        let records = match query.source_file.as_deref() {
            Some(source_file) => self.section_index_records_for_file(source_file),
            None => self.section_index_records(),
        };
        let tag_matcher = TagMatcher::new(&self.tag_definitions);
        let total_candidates = records.len();
        let (cards, skipped) = records.iter().fold(
            (Vec::new(), Vec::new()),
            |(mut cards, mut skipped), record| {
                match sparse_tree_decision(record, query, tag_matcher) {
                    SparseTreeDecision::Accept(card) => cards.push(*card),
                    SparseTreeDecision::Skip(skip) if query.explain_skips => skipped.push(skip),
                    SparseTreeDecision::Skip(_) => {}
                }
                (cards, skipped)
            },
        );
        SparseTreeProjection {
            total_candidates,
            cards,
            skipped,
        }
    }
}

enum SparseTreeDecision {
    Accept(Box<SparseTreeCard>),
    Skip(SparseTreeSkip),
}

fn sparse_tree_decision(
    record: &SectionIndexRecord,
    query: &SparseTreeQuery,
    tag_matcher: TagMatcher<'_>,
) -> SparseTreeDecision {
    let mut receipts = vec![SparseTreeReceipt {
        kind: SparseTreeReceiptKind::Candidate,
        message: "section entered the sparse-tree candidate set".to_string(),
    }];

    if record.is_comment && !query.include_comments {
        return skipped_record(record, SparseTreeSkipReason::Comment, receipts);
    }
    if record.archive.archived && !query.include_archived {
        return skipped_record(record, SparseTreeSkipReason::Archived, receipts);
    }
    if record
        .todo
        .as_ref()
        .is_some_and(|todo| matches!(todo.state, super::TodoState::Done) && !query.include_done)
    {
        return skipped_record(record, SparseTreeSkipReason::Done, receipts);
    }
    receipts.push(SparseTreeReceipt {
        kind: SparseTreeReceiptKind::VisibilityFilterPassed,
        message: "section passed comment, archive, and done-state visibility filters".to_string(),
    });

    let mut matches = Vec::new();
    if let Some(match_query) = &query.match_query {
        let Some(match_reasons) = match_query_matches_record(record, match_query, tag_matcher)
        else {
            return skipped_record(record, SparseTreeSkipReason::MatchExpression, receipts);
        };
        receipts.push(SparseTreeReceipt {
            kind: SparseTreeReceiptKind::MatchExpressionMatched,
            message: format!(
                "agenda-style match expression `{}` accepted this section",
                match_query.expression()
            ),
        });
        matches.extend(match_reasons);
    }
    if let Some(text) = &query.text {
        let text_matches = text_matches_record(record, text);
        if text_matches.is_empty() {
            return skipped_record(record, SparseTreeSkipReason::Text, receipts);
        }
        receipts.push(SparseTreeReceipt {
            kind: SparseTreeReceiptKind::TextMatched,
            message: format!("document-local text predicate `{text}` matched this section"),
        });
        matches.extend(text_matches);
    }
    if !query.has_predicate() {
        receipts.push(SparseTreeReceipt {
            kind: SparseTreeReceiptKind::DefaultAllMatched,
            message: "query has no predicate, so visible sections are accepted".to_string(),
        });
        matches.push(SparseTreeMatch {
            source: record.source.clone(),
            kind: SparseTreeMatchKind::Query,
            key: None,
            value: "all".to_string(),
        });
    }
    receipts.push(SparseTreeReceipt {
        kind: SparseTreeReceiptKind::Accepted,
        message: "section accepted into sparse-tree projection".to_string(),
    });

    SparseTreeDecision::Accept(Box::new(SparseTreeCard {
        source: record.source.clone(),
        outline_path: record.outline_path.clone(),
        level: record.level,
        title: record.title.clone(),
        matches,
        receipts,
        preview: preview(record),
        todo: record.todo.clone(),
        priority: record.priority.clone(),
        category: record.category.clone(),
        tags: record.tags.clone(),
        effective_tags: record.effective_tags.clone(),
        properties: record.properties.clone(),
        special_properties: record.special_properties.clone(),
        planning: record.planning.clone(),
        archive: record.archive.clone(),
        attachment: record.attachment.clone(),
        links: record.links.clone(),
        targets: record.targets.clone(),
        lifecycle: record.lifecycle.clone(),
    }))
}

fn skipped_record(
    record: &SectionIndexRecord,
    reason: SparseTreeSkipReason,
    mut receipts: Vec<SparseTreeReceipt>,
) -> SparseTreeDecision {
    receipts.push(SparseTreeReceipt {
        kind: skipped_receipt_kind(reason),
        message: skipped_message(reason),
    });
    SparseTreeDecision::Skip(SparseTreeSkip {
        source: record.source.clone(),
        outline_path: record.outline_path.clone(),
        level: record.level,
        title: record.title.clone(),
        reason,
        receipts,
    })
}

fn skipped_receipt_kind(reason: SparseTreeSkipReason) -> SparseTreeReceiptKind {
    match reason {
        SparseTreeSkipReason::Comment => SparseTreeReceiptKind::SkippedComment,
        SparseTreeSkipReason::Archived => SparseTreeReceiptKind::SkippedArchived,
        SparseTreeSkipReason::Done => SparseTreeReceiptKind::SkippedDone,
        SparseTreeSkipReason::MatchExpression => SparseTreeReceiptKind::SkippedMatchExpression,
        SparseTreeSkipReason::Text => SparseTreeReceiptKind::SkippedText,
    }
}

fn skipped_message(reason: SparseTreeSkipReason) -> String {
    match reason {
        SparseTreeSkipReason::Comment => {
            "skipped because COMMENT headlines are excluded".to_string()
        }
        SparseTreeSkipReason::Archived => {
            "skipped because archived sections are excluded".to_string()
        }
        SparseTreeSkipReason::Done => {
            "skipped because DONE-state sections are excluded".to_string()
        }
        SparseTreeSkipReason::MatchExpression => {
            "skipped because the agenda-style match expression did not match".to_string()
        }
        SparseTreeSkipReason::Text => {
            "skipped because the document-local text predicate did not match".to_string()
        }
    }
}

fn match_query_matches_record(
    record: &SectionIndexRecord,
    query: &AgendaMatchQuery,
    tag_matcher: TagMatcher<'_>,
) -> Option<Vec<SparseTreeMatch>> {
    query
        .clauses
        .iter()
        .find_map(|clause| match_clause_reasons(record, query, clause, tag_matcher))
}

fn match_clause_reasons(
    record: &SectionIndexRecord,
    query: &AgendaMatchQuery,
    clause: &AgendaMatchClause,
    tag_matcher: TagMatcher<'_>,
) -> Option<Vec<SparseTreeMatch>> {
    let mut reasons = Vec::new();
    for term in &clause.terms {
        if !collect_accepted_term_reasons(record, term, &mut reasons, tag_matcher) {
            return None;
        }
    }
    if reasons.is_empty() {
        reasons.push(SparseTreeMatch {
            source: record.source.clone(),
            kind: SparseTreeMatchKind::Query,
            key: None,
            value: query.expression().to_string(),
        });
    }
    Some(reasons)
}

fn collect_accepted_term_reasons(
    record: &SectionIndexRecord,
    term: &AgendaMatchTerm,
    reasons: &mut Vec<SparseTreeMatch>,
    tag_matcher: TagMatcher<'_>,
) -> bool {
    let term_reasons = match_term_reasons(record, term, tag_matcher);
    let term_matched = !term_reasons.is_empty();
    if term.positive && term_matched {
        reasons.extend(term_reasons);
        true
    } else if term.positive {
        false
    } else {
        !term_matched
    }
}

fn match_term_reasons(
    record: &SectionIndexRecord,
    term: &AgendaMatchTerm,
    tag_matcher: TagMatcher<'_>,
) -> Vec<SparseTreeMatch> {
    match &term.predicate {
        AgendaMatchPredicate::Tag(tag) => tag_match_reason(record, tag, tag_matcher)
            .into_iter()
            .collect(),
        AgendaMatchPredicate::Property {
            key,
            operator,
            value,
        } => property_match_reasons(record, key, *operator, value, tag_matcher),
    }
}

fn tag_match_reason(
    record: &SectionIndexRecord,
    tag: &str,
    tag_matcher: TagMatcher<'_>,
) -> Option<SparseTreeMatch> {
    tag_matcher
        .matched_tag(&record.effective_tags, tag)
        .map(|actual| SparseTreeMatch {
            source: record.source.clone(),
            kind: SparseTreeMatchKind::Tag,
            key: Some("tag".to_string()),
            value: actual.clone(),
        })
}

fn property_match_reasons(
    record: &SectionIndexRecord,
    key: &str,
    operator: AgendaMatchOperator,
    expected: &AgendaMatchValue,
    tag_matcher: TagMatcher<'_>,
) -> Vec<SparseTreeMatch> {
    let mut reasons = Vec::new();
    let tag_property_match =
        tag_property_match_result(record, key, operator, expected, tag_matcher);
    for property in &record.special_properties {
        if !property.name.eq_ignore_ascii_case(key) {
            continue;
        }
        let matched = tag_property_match
            .unwrap_or_else(|| compare_match_values(&property.value, operator, expected));
        if matched {
            reasons.push(special_property_match(property));
        }
    }
    if tag_property_match.is_some() {
        return reasons;
    }
    for property in &record.effective_properties {
        if property.key.eq_ignore_ascii_case(key)
            && compare_match_values(&property.value, operator, expected)
        {
            reasons.push(property_match(property));
        }
    }
    reasons
}

fn tag_property_match_result(
    record: &SectionIndexRecord,
    key: &str,
    operator: AgendaMatchOperator,
    expected: &AgendaMatchValue,
    tag_matcher: TagMatcher<'_>,
) -> Option<bool> {
    let tags = if key.eq_ignore_ascii_case("TAGS") {
        &record.tags
    } else if key.eq_ignore_ascii_case("ALLTAGS") {
        &record.effective_tags
    } else {
        return None;
    };

    match operator {
        AgendaMatchOperator::Equal | AgendaMatchOperator::NotEqual => {
            let matched = sparse_tag_property_value_matches(tags, expected, tag_matcher);
            Some(if operator == AgendaMatchOperator::Equal {
                matched
            } else {
                !matched
            })
        }
        AgendaMatchOperator::Less
        | AgendaMatchOperator::LessOrEqual
        | AgendaMatchOperator::Greater
        | AgendaMatchOperator::GreaterOrEqual => None,
    }
}

fn sparse_tag_property_value_matches(
    tags: &[String],
    expected: &AgendaMatchValue,
    tag_matcher: TagMatcher<'_>,
) -> bool {
    if compare_match_values(
        super::special_properties::tag_string(tags).as_str(),
        AgendaMatchOperator::Equal,
        expected,
    ) {
        return true;
    }

    tag_matcher.has_tag_value(tags, expected.as_str())
}

fn special_property_match(property: &SectionIndexSpecialProperty) -> SparseTreeMatch {
    SparseTreeMatch {
        source: property.source.clone(),
        kind: special_property_kind(&property.name),
        key: Some(property.name.clone()),
        value: property.value.clone(),
    }
}

fn special_property_kind(name: &str) -> SparseTreeMatchKind {
    if name.eq_ignore_ascii_case("PRIORITY") {
        SparseTreeMatchKind::Priority
    } else if name.eq_ignore_ascii_case("SCHEDULED")
        || name.eq_ignore_ascii_case("DEADLINE")
        || name.eq_ignore_ascii_case("CLOSED")
    {
        SparseTreeMatchKind::Planning
    } else {
        SparseTreeMatchKind::SpecialProperty
    }
}

fn property_match(property: &SectionIndexProperty) -> SparseTreeMatch {
    SparseTreeMatch {
        source: property.source.clone(),
        kind: SparseTreeMatchKind::Property,
        key: Some(property.key.clone()),
        value: property.value.clone(),
    }
}

fn text_matches_record(record: &SectionIndexRecord, text: &str) -> Vec<SparseTreeMatch> {
    let needle = text.to_ascii_lowercase();
    let mut matches = Vec::new();

    push_text_match(
        &mut matches,
        &record.source,
        SparseTreeMatchKind::Title,
        Some("title"),
        &record.title,
        &needle,
    );
    for slice in &record.body {
        push_text_match(
            &mut matches,
            &slice.source,
            SparseTreeMatchKind::Body,
            None,
            &slice.text,
            &needle,
        );
    }
    for property in &record.properties {
        push_text_match(
            &mut matches,
            &property.source,
            SparseTreeMatchKind::Property,
            Some(&property.key),
            &property.value,
            &needle,
        );
    }
    for property in &record.special_properties {
        push_text_match(
            &mut matches,
            &property.source,
            special_property_kind(&property.name),
            Some(&property.name),
            &property.value,
            &needle,
        );
    }
    for link in &record.links {
        push_text_match(
            &mut matches,
            &link.source,
            SparseTreeMatchKind::Link,
            Some("path"),
            &link.path,
            &needle,
        );
        push_text_match(
            &mut matches,
            &link.source,
            SparseTreeMatchKind::Link,
            Some("description"),
            &link.description,
            &needle,
        );
    }
    for target in &record.targets {
        push_text_match(
            &mut matches,
            &target.source,
            SparseTreeMatchKind::Target,
            Some(&target.key),
            &target.value,
            &needle,
        );
    }

    matches
}

fn push_text_match(
    matches: &mut Vec<SparseTreeMatch>,
    source: &SectionIndexSource,
    kind: SparseTreeMatchKind,
    key: Option<&str>,
    value: &str,
    needle: &str,
) {
    if value.to_ascii_lowercase().contains(needle) {
        matches.push(SparseTreeMatch {
            source: source.clone(),
            kind,
            key: key.map(ToOwned::to_owned),
            value: value.to_string(),
        });
    }
}

fn preview(record: &SectionIndexRecord) -> Option<String> {
    let text = record
        .body
        .iter()
        .map(|slice| slice.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    (!normalized.is_empty()).then(|| truncate_text(&normalized, PREVIEW_CHARS))
}

fn compare_match_values(
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
