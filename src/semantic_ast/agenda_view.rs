//! Explainable agenda view plans over existing agenda rows.

use std::cmp::Ordering;

use super::{
    AgendaBlockSectionPlan, AgendaBlockViewPlan, AgendaBlockViewQuery, AgendaEntry,
    AgendaEntryKind, AgendaViewCard, AgendaViewPlan, AgendaViewQuery, AgendaViewReceipt,
    AgendaViewReceiptKind, AgendaViewSkip, AgendaViewSkipReason, AgendaViewSortDirection,
    AgendaViewSortKey, AgendaViewSortSpec, AgendaViewSortValue, Document, ParsedAnnotation,
    SectionIndexSource, TaskBlockerRecord,
    agenda_urgency::agenda_urgency_score,
    agenda_view_model::{compact_sort_strategy, format_date, format_time},
    task_blockers::{blockers_by_blocked_source, blockers_for_source},
};

impl Document<ParsedAnnotation> {
    /// Projects agenda rows into an explainable sorted and limited view plan.
    ///
    /// This API does not change agenda matching semantics. It records why rows
    /// were accepted or skipped after the existing agenda query produced its
    /// sorted candidate list.
    pub fn agenda_view_plan(&self, query: &AgendaViewQuery) -> AgendaViewPlan {
        let candidates = sort_candidates(self.agenda_entries(&query.agenda), &query.sort_strategy);
        let blockers_by_source = blockers_by_blocked_source(self.task_blocker_records());
        let total_candidates = candidates.len();
        let mut cards = Vec::new();
        let mut skipped = Vec::new();

        for (index, entry) in candidates.into_iter().enumerate() {
            let sorted_position = index + 1;
            let source = SectionIndexSource::from_annotation(&entry.ann);
            let blockers = blockers_for_source(&blockers_by_source, &source);
            if query.limit.is_some_and(|limit| sorted_position > limit) {
                skipped.push(skipped_entry(
                    entry,
                    source,
                    sorted_position,
                    query.limit.unwrap(),
                    blockers,
                    &query.sort_strategy,
                ));
            } else {
                cards.push(accepted_card(
                    entry,
                    source,
                    sorted_position,
                    query.limit,
                    blockers,
                    &query.sort_strategy,
                ));
            }
        }

        AgendaViewPlan {
            total_candidates,
            limit: query.limit,
            sort_strategy: query.sort_strategy.clone(),
            cards,
            skipped,
        }
    }

    /// Projects multiple named agenda views into a block-agenda style plan.
    ///
    /// Each section has its own query and sort receipts. The block plan is a
    /// read-only projection intended for Agent Search / Memory consumers.
    pub fn agenda_block_view_plan(&self, query: &AgendaBlockViewQuery) -> AgendaBlockViewPlan {
        let sections = query
            .sections
            .iter()
            .enumerate()
            .map(|(index, section)| AgendaBlockSectionPlan {
                index: index + 1,
                name: section.name.clone(),
                plan: self.agenda_view_plan(&section.query),
            })
            .collect::<Vec<_>>();
        let total_candidates = sections
            .iter()
            .map(|section| section.plan.total_candidates)
            .sum();
        AgendaBlockViewPlan {
            title: query.title.clone(),
            total_candidates,
            sections,
        }
    }
}

fn accepted_card(
    entry: AgendaEntry<ParsedAnnotation>,
    source: SectionIndexSource,
    sorted_position: usize,
    limit: Option<usize>,
    blockers: Vec<TaskBlockerRecord>,
    strategy: &[AgendaViewSortSpec],
) -> AgendaViewCard {
    let sort_keys = sort_keys(&entry);
    let urgency = agenda_urgency_score(&entry);
    let receipts = accepted_receipts(sorted_position, limit, blockers.len(), &sort_keys, strategy);
    AgendaViewCard {
        source,
        sorted_position,
        kind: entry.kind,
        display_date: entry.display_date,
        target_date: entry.target_date,
        target_end_date: entry.target_end_date,
        time: entry.time,
        end_time: entry.end_time,
        title: entry.raw_title,
        category: entry.category,
        todo: entry.todo,
        effective_tags: entry.effective_tags,
        urgency,
        blockers,
        sort_keys,
        receipts,
    }
}

fn skipped_entry(
    entry: AgendaEntry<ParsedAnnotation>,
    source: SectionIndexSource,
    sorted_position: usize,
    limit: usize,
    blockers: Vec<TaskBlockerRecord>,
    strategy: &[AgendaViewSortSpec],
) -> AgendaViewSkip {
    let sort_keys = sort_keys(&entry);
    let urgency = agenda_urgency_score(&entry);
    AgendaViewSkip {
        source,
        sorted_position,
        title: entry.raw_title,
        reason: AgendaViewSkipReason::Limit { limit },
        urgency,
        receipts: skipped_receipts(sorted_position, limit, blockers.len(), &sort_keys, strategy),
        blockers,
        sort_keys,
    }
}

fn accepted_receipts(
    sorted_position: usize,
    limit: Option<usize>,
    blocker_count: usize,
    sort_keys: &[AgendaViewSortValue],
    strategy: &[AgendaViewSortSpec],
) -> Vec<AgendaViewReceipt> {
    let mut receipts = vec![
        AgendaViewReceipt {
            kind: AgendaViewReceiptKind::QueryMatched,
            message: "agenda query produced this visible candidate row".to_string(),
        },
        AgendaViewReceipt {
            kind: AgendaViewReceiptKind::Sorted,
            message: sort_receipt_message(sort_keys, strategy),
        },
    ];
    push_blocker_receipt(&mut receipts, blocker_count);
    receipts.push(AgendaViewReceipt {
        kind: AgendaViewReceiptKind::Accepted,
        message: match limit {
            Some(limit) => format!(
                "accepted because sorted position {sorted_position} is within limit {limit}"
            ),
            None => "accepted because no view limit was configured".to_string(),
        },
    });
    receipts
}

fn skipped_receipts(
    sorted_position: usize,
    limit: usize,
    blocker_count: usize,
    sort_keys: &[AgendaViewSortValue],
    strategy: &[AgendaViewSortSpec],
) -> Vec<AgendaViewReceipt> {
    let mut receipts = vec![
        AgendaViewReceipt {
            kind: AgendaViewReceiptKind::QueryMatched,
            message: "agenda query produced this visible candidate row".to_string(),
        },
        AgendaViewReceipt {
            kind: AgendaViewReceiptKind::Sorted,
            message: sort_receipt_message(sort_keys, strategy),
        },
        AgendaViewReceipt {
            kind: AgendaViewReceiptKind::SkippedLimit,
            message: format!(
                "skipped because sorted position {sorted_position} exceeds limit {limit}"
            ),
        },
    ];
    push_blocker_receipt(&mut receipts, blocker_count);
    receipts
}

fn push_blocker_receipt(receipts: &mut Vec<AgendaViewReceipt>, blocker_count: usize) {
    if blocker_count == 0 {
        return;
    }

    receipts.push(AgendaViewReceipt {
        kind: AgendaViewReceiptKind::BlockedByOrderedSibling,
        message: format!("{blocker_count} local ORDERED previous-sibling blocker edge(s) attached"),
    });
}

fn sort_keys(entry: &AgendaEntry<ParsedAnnotation>) -> Vec<AgendaViewSortValue> {
    vec![
        AgendaViewSortValue {
            key: AgendaViewSortKey::DisplayDate,
            value: format_date(entry.display_date),
        },
        AgendaViewSortValue {
            key: AgendaViewSortKey::Time,
            value: format_time(entry.time),
        },
        AgendaViewSortValue {
            key: AgendaViewSortKey::Kind,
            value: entry.kind.as_str().to_string(),
        },
        AgendaViewSortValue {
            key: AgendaViewSortKey::Level,
            value: entry.level.to_string(),
        },
        AgendaViewSortValue {
            key: AgendaViewSortKey::Title,
            value: entry.raw_title.clone(),
        },
        AgendaViewSortValue {
            key: AgendaViewSortKey::TargetDate,
            value: format_date(entry.target_date),
        },
        AgendaViewSortValue {
            key: AgendaViewSortKey::ScheduledDate,
            value: match entry.kind {
                AgendaEntryKind::Scheduled => format_date(entry.target_date),
                _ => "none".to_string(),
            },
        },
        AgendaViewSortValue {
            key: AgendaViewSortKey::DeadlineDate,
            value: match entry.kind {
                AgendaEntryKind::Deadline => format_date(entry.target_date),
                _ => "none".to_string(),
            },
        },
        AgendaViewSortValue {
            key: AgendaViewSortKey::Priority,
            value: entry.priority.effective_text(),
        },
        AgendaViewSortValue {
            key: AgendaViewSortKey::Category,
            value: entry
                .category
                .as_ref()
                .map(|category| category.as_str().to_string())
                .unwrap_or_else(|| "none".to_string()),
        },
        AgendaViewSortValue {
            key: AgendaViewSortKey::TodoState,
            value: entry
                .todo
                .as_ref()
                .map(|todo| todo.name.clone())
                .unwrap_or_else(|| "none".to_string()),
        },
    ]
}

fn sort_candidates(
    entries: Vec<AgendaEntry<ParsedAnnotation>>,
    strategy: &[AgendaViewSortSpec],
) -> Vec<AgendaEntry<ParsedAnnotation>> {
    if strategy.is_empty() {
        return entries;
    }

    let mut indexed_entries = entries.into_iter().enumerate().collect::<Vec<_>>();
    indexed_entries.sort_by(|left, right| {
        compare_by_strategy(&left.1, &right.1, strategy).then_with(|| left.0.cmp(&right.0))
    });
    indexed_entries
        .into_iter()
        .map(|(_, entry)| entry)
        .collect()
}

fn compare_by_strategy(
    left: &AgendaEntry<ParsedAnnotation>,
    right: &AgendaEntry<ParsedAnnotation>,
    strategy: &[AgendaViewSortSpec],
) -> Ordering {
    strategy
        .iter()
        .filter(|spec| spec.direction != AgendaViewSortDirection::Keep)
        .map(|spec| compare_sort_spec(left, right, *spec))
        .find(|ordering| *ordering != Ordering::Equal)
        .unwrap_or(Ordering::Equal)
}

fn compare_sort_spec(
    left: &AgendaEntry<ParsedAnnotation>,
    right: &AgendaEntry<ParsedAnnotation>,
    spec: AgendaViewSortSpec,
) -> Ordering {
    let ordering = match spec.key {
        AgendaViewSortKey::DisplayDate => left.display_date.cmp(&right.display_date),
        AgendaViewSortKey::Time => cmp_optional_late(left.time, right.time),
        AgendaViewSortKey::Kind => kind_order(left.kind).cmp(&kind_order(right.kind)),
        AgendaViewSortKey::Level => left.level.cmp(&right.level),
        AgendaViewSortKey::Title => left.raw_title.cmp(&right.raw_title),
        AgendaViewSortKey::TargetDate => left.target_date.cmp(&right.target_date),
        AgendaViewSortKey::ScheduledDate => cmp_optional_late(
            date_for_kind(left, AgendaEntryKind::Scheduled),
            date_for_kind(right, AgendaEntryKind::Scheduled),
        ),
        AgendaViewSortKey::DeadlineDate => cmp_optional_late(
            date_for_kind(left, AgendaEntryKind::Deadline),
            date_for_kind(right, AgendaEntryKind::Deadline),
        ),
        AgendaViewSortKey::Priority => cmp_optional_late(
            left.priority.org_priority_score(),
            right.priority.org_priority_score(),
        ),
        AgendaViewSortKey::Category => cmp_optional_str_late(
            left.category.as_ref().map(|category| category.as_str()),
            right.category.as_ref().map(|category| category.as_str()),
        ),
        AgendaViewSortKey::TodoState => cmp_optional_str_late(
            left.todo.as_ref().map(|todo| todo.name.as_str()),
            right.todo.as_ref().map(|todo| todo.name.as_str()),
        ),
    };

    match spec.direction {
        AgendaViewSortDirection::Up | AgendaViewSortDirection::Keep => ordering,
        AgendaViewSortDirection::Down => ordering.reverse(),
    }
}

fn cmp_optional_late<T: Ord>(left: Option<T>, right: Option<T>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn cmp_optional_str_late(left: Option<&str>, right: Option<&str>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn date_for_kind(
    entry: &AgendaEntry<ParsedAnnotation>,
    kind: AgendaEntryKind,
) -> Option<super::AgendaDate> {
    (entry.kind == kind).then_some(entry.target_date)
}

fn kind_order(kind: AgendaEntryKind) -> u8 {
    match kind {
        AgendaEntryKind::Deadline => 0,
        AgendaEntryKind::Scheduled => 1,
        AgendaEntryKind::Timestamp => 2,
        AgendaEntryKind::Diary => 3,
        AgendaEntryKind::Closed => 4,
    }
}

fn sort_receipt_message(
    sort_keys: &[AgendaViewSortValue],
    strategy: &[AgendaViewSortSpec],
) -> String {
    let sort_key_text = sort_key_text(sort_keys);
    if strategy.is_empty() {
        format!("default agenda order: {sort_key_text}")
    } else {
        format!(
            "agenda sort strategy: {}; keys: {}",
            compact_sort_strategy(strategy),
            sort_key_text
        )
    }
}

fn sort_key_text(sort_keys: &[AgendaViewSortValue]) -> String {
    sort_keys
        .iter()
        .map(|sort_key| format!("{}={}", sort_key.key.as_str(), sort_key.value))
        .collect::<Vec<_>>()
        .join(", ")
}
