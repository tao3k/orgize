//! Workspace-level Org Agenda command plans.

use std::collections::BTreeMap;

use super::agenda_filter::section_matches_agenda_match;
use super::agenda_model::is_done_keyword;
use super::{
    AgendaUrgencyIngredient, AgendaUrgencyIngredientKind, AgendaUrgencyScore, AgendaViewCard,
    AgendaWorkspaceCard, AgendaWorkspaceCardKind, AgendaWorkspaceCommandKind,
    AgendaWorkspaceCommandPlan, AgendaWorkspaceDocumentSummary, AgendaWorkspaceMatchCommand,
    AgendaWorkspacePlan, AgendaWorkspaceQuery, AgendaWorkspaceReceipt, AgendaWorkspaceReceiptKind,
    AgendaWorkspaceSkip, AgendaWorkspaceSkipReason, Document, ParsedAnnotation, Priority, Section,
    SectionIndexRecord, SectionIndexSource,
};

/// Builder for non-mutating workspace Agenda-style command plans.
#[derive(Clone, Debug, Default)]
pub struct AgendaWorkspaceBuilder<'a> {
    documents: Vec<AgendaWorkspaceInput<'a>>,
}

#[derive(Clone, Debug)]
struct AgendaWorkspaceInput<'a> {
    source_file: String,
    document: &'a Document<ParsedAnnotation>,
    sections: Vec<SectionIndexRecord>,
}

impl<'a> AgendaWorkspaceBuilder<'a> {
    /// Creates an empty workspace agenda builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one already-parsed document to the workspace command plan.
    pub fn add_document(
        &mut self,
        source_file: impl Into<String>,
        document: &'a Document<ParsedAnnotation>,
    ) -> &mut Self {
        let source_file = source_file.into();
        let sections = document.section_index_records_for_file(source_file.clone());
        self.documents.push(AgendaWorkspaceInput {
            source_file,
            document,
            sections,
        });
        self
    }

    /// Runs the query's named commands over all added documents.
    pub fn finish(&self, query: &AgendaWorkspaceQuery) -> AgendaWorkspacePlan {
        AgendaWorkspacePlan {
            documents: self.document_summaries(),
            commands: query
                .commands
                .iter()
                .map(|command| self.command_plan(command.name.clone(), &command.kind))
                .collect(),
        }
    }

    fn document_summaries(&self) -> Vec<AgendaWorkspaceDocumentSummary> {
        self.documents
            .iter()
            .map(|document| AgendaWorkspaceDocumentSummary {
                source_file: document.source_file.clone(),
                section_count: document.sections.len(),
            })
            .collect()
    }

    fn command_plan(
        &self,
        name: String,
        kind: &AgendaWorkspaceCommandKind,
    ) -> AgendaWorkspaceCommandPlan {
        let mut cards = Vec::new();
        let mut skipped = Vec::new();
        match kind {
            AgendaWorkspaceCommandKind::Agenda(query) => {
                for document in &self.documents {
                    let lookup = section_lookup(&document.sections);
                    let plan = document.document.agenda_view_plan(query);
                    cards.extend(
                        plan.cards
                            .into_iter()
                            .map(|card| agenda_card(document.source_file.as_str(), &lookup, card)),
                    );
                    skipped.extend(plan.skipped.into_iter().map(|skip| AgendaWorkspaceSkip {
                        source_file: document.source_file.clone(),
                        source: skip.source,
                        title: skip.title,
                        reason: AgendaWorkspaceSkipReason::AgendaViewLimit,
                        receipts: vec![AgendaWorkspaceReceipt {
                            kind: AgendaWorkspaceReceiptKind::AgendaViewSkipped,
                            message: skip.reason.as_str().to_string(),
                        }],
                    }));
                }
            }
            AgendaWorkspaceCommandKind::TodoList { include_done } => {
                for document in &self.documents {
                    cards.extend(document.sections.iter().filter_map(|section| {
                        todo_card(document.source_file.as_str(), section, *include_done)
                    }));
                }
            }
            AgendaWorkspaceCommandKind::Match(match_command) => {
                for document in &self.documents {
                    collect_match_cards(document, match_command, &mut cards);
                }
            }
            AgendaWorkspaceCommandKind::Search {
                needle,
                case_sensitive,
            } => {
                for document in &self.documents {
                    cards.extend(document.sections.iter().filter_map(|section| {
                        search_card(
                            document.source_file.as_str(),
                            section,
                            needle,
                            *case_sensitive,
                        )
                    }));
                }
            }
            AgendaWorkspaceCommandKind::StuckProjects { next_keywords } => {
                for document in &self.documents {
                    for section in &document.document.sections {
                        collect_stuck_cards(
                            document.source_file.as_str(),
                            section,
                            Vec::new(),
                            next_keywords,
                            &mut cards,
                        );
                    }
                }
            }
        }
        AgendaWorkspaceCommandPlan {
            name,
            kind: kind.label(),
            total_candidates: cards.len() + skipped.len(),
            cards,
            skipped,
        }
    }
}

fn agenda_card(
    source_file: &str,
    lookup: &BTreeMap<(u32, u32), &SectionIndexRecord>,
    card: AgendaViewCard,
) -> AgendaWorkspaceCard {
    let section = lookup.get(&(card.source.range_start, card.source.range_end));
    AgendaWorkspaceCard {
        source_file: source_file.to_string(),
        source: card.source,
        title: card.title,
        outline_path: section
            .map(|section| section.outline_path.clone())
            .unwrap_or_default(),
        level: section.map(|section| section.level).unwrap_or_default(),
        kind: AgendaWorkspaceCardKind::Agenda,
        todo: card.todo,
        time: card.time,
        urgency: card.urgency,
        receipts: vec![
            document_receipt(source_file),
            AgendaWorkspaceReceipt {
                kind: AgendaWorkspaceReceiptKind::QueryMatched,
                message: "workspace agenda command accepted this agenda row".to_string(),
            },
        ],
    }
}

fn todo_card(
    source_file: &str,
    section: &SectionIndexRecord,
    include_done: bool,
) -> Option<AgendaWorkspaceCard> {
    let todo = section.todo.as_ref()?;
    if !include_done && matches!(todo.state, super::TodoState::Done) {
        return None;
    }
    Some(section_card(
        source_file,
        section,
        AgendaWorkspaceCardKind::Todo,
        vec![
            document_receipt(source_file),
            AgendaWorkspaceReceipt {
                kind: AgendaWorkspaceReceiptKind::TodoMatched,
                message: format!("matched TODO keyword {}", todo.name),
            },
        ],
    ))
}

fn collect_match_cards(
    document: &AgendaWorkspaceInput<'_>,
    command: &AgendaWorkspaceMatchCommand,
    cards: &mut Vec<AgendaWorkspaceCard>,
) {
    for section in &document.document.sections {
        collect_match_cards_from_section(
            document.source_file.as_str(),
            section,
            Vec::new(),
            &command.query,
            command.include_done,
            command.include_archived,
            cards,
        );
    }
}

fn collect_match_cards_from_section(
    source_file: &str,
    section: &Section<ParsedAnnotation>,
    parent_outline_path: Vec<String>,
    query: &super::AgendaMatchQuery,
    include_done: bool,
    include_archived: bool,
    cards: &mut Vec<AgendaWorkspaceCard>,
) {
    let current_outline_path = outline_path(parent_outline_path, section);
    if (include_done || !is_done_keyword(&section.todo))
        && (include_archived || !section.archive.archived)
        && section_matches_agenda_match(section, None, Some(source_file), query)
    {
        cards.push(section_ast_card(
            source_file,
            section,
            current_outline_path.clone(),
            AgendaWorkspaceCardKind::Match,
            vec![
                document_receipt(source_file),
                AgendaWorkspaceReceipt {
                    kind: AgendaWorkspaceReceiptKind::QueryMatched,
                    message: format!("matched {}", query.expression()),
                },
            ],
        ));
    }
    for subsection in &section.subsections {
        collect_match_cards_from_section(
            source_file,
            subsection,
            current_outline_path.clone(),
            query,
            include_done,
            include_archived,
            cards,
        );
    }
}

fn search_card(
    source_file: &str,
    section: &SectionIndexRecord,
    needle: &str,
    case_sensitive: bool,
) -> Option<AgendaWorkspaceCard> {
    let matched = searchable_text(section)
        .into_iter()
        .any(|text| contains_text(text.as_str(), needle, case_sensitive));
    matched.then(|| {
        section_card(
            source_file,
            section,
            AgendaWorkspaceCardKind::Search,
            vec![
                document_receipt(source_file),
                AgendaWorkspaceReceipt {
                    kind: AgendaWorkspaceReceiptKind::SearchMatched,
                    message: format!("matched text `{needle}`"),
                },
            ],
        )
    })
}

fn collect_stuck_cards(
    source_file: &str,
    section: &Section<ParsedAnnotation>,
    parent_outline_path: Vec<String>,
    next_keywords: &[String],
    cards: &mut Vec<AgendaWorkspaceCard>,
) {
    let current_outline_path = outline_path(parent_outline_path, section);
    if is_project(section) && !has_next_action(section, next_keywords) {
        cards.push(section_ast_card(
            source_file,
            section,
            current_outline_path.clone(),
            AgendaWorkspaceCardKind::StuckProject,
            vec![
                document_receipt(source_file),
                AgendaWorkspaceReceipt {
                    kind: AgendaWorkspaceReceiptKind::StuckProjectMatched,
                    message: "TODO project has no NEXT-style descendant action".to_string(),
                },
            ],
        ));
    }
    for subsection in &section.subsections {
        collect_stuck_cards(
            source_file,
            subsection,
            current_outline_path.clone(),
            next_keywords,
            cards,
        );
    }
}

fn section_card(
    source_file: &str,
    section: &SectionIndexRecord,
    kind: AgendaWorkspaceCardKind,
    receipts: Vec<AgendaWorkspaceReceipt>,
) -> AgendaWorkspaceCard {
    AgendaWorkspaceCard {
        source_file: source_file.to_string(),
        source: section.source.clone(),
        title: section.title.clone(),
        outline_path: section.outline_path.clone(),
        level: section.level,
        kind,
        todo: section.todo.clone(),
        time: None,
        urgency: section_urgency(
            &section.priority,
            section.todo.as_ref(),
            &section.effective_tags,
        ),
        receipts,
    }
}

fn section_ast_card(
    source_file: &str,
    section: &Section<ParsedAnnotation>,
    outline_path: Vec<String>,
    kind: AgendaWorkspaceCardKind,
    receipts: Vec<AgendaWorkspaceReceipt>,
) -> AgendaWorkspaceCard {
    AgendaWorkspaceCard {
        source_file: source_file.to_string(),
        source: SectionIndexSource::from_annotation(&section.ann),
        title: section.raw_title.trim().to_string(),
        outline_path,
        level: section.level,
        kind,
        todo: section.todo.clone(),
        time: None,
        urgency: section_urgency(
            &section.priority,
            section.todo.as_ref(),
            &section.effective_tags,
        ),
        receipts,
    }
}

fn outline_path(
    mut parent_outline_path: Vec<String>,
    section: &Section<ParsedAnnotation>,
) -> Vec<String> {
    parent_outline_path.push(section.raw_title.trim().to_string());
    parent_outline_path
}

fn section_urgency(
    priority: &Priority,
    todo: Option<&super::TodoKeyword>,
    tags: &[String],
) -> AgendaUrgencyScore {
    let mut ingredients = vec![
        AgendaUrgencyIngredient {
            kind: AgendaUrgencyIngredientKind::Priority,
            score: priority.org_priority_score().unwrap_or(0),
            message: format!("priority {}", priority.effective_text()),
        },
        AgendaUrgencyIngredient {
            kind: AgendaUrgencyIngredientKind::TodoState,
            score: todo.map(|_| 250).unwrap_or(0),
            message: todo
                .map(|todo| format!("todo keyword {}", todo.name))
                .unwrap_or_else(|| "no todo keyword".to_string()),
        },
    ];
    let urgent_tags = tags
        .iter()
        .filter(|tag| tag.eq_ignore_ascii_case("urgent") || tag.eq_ignore_ascii_case("now"))
        .count();
    ingredients.push(AgendaUrgencyIngredient {
        kind: AgendaUrgencyIngredientKind::Tags,
        score: i32::try_from(urgent_tags).unwrap_or(0) * 200,
        message: format!("{urgent_tags} urgency tag(s)"),
    });
    let total = ingredients.iter().map(|ingredient| ingredient.score).sum();
    AgendaUrgencyScore { total, ingredients }
}

fn document_receipt(source_file: &str) -> AgendaWorkspaceReceipt {
    AgendaWorkspaceReceipt {
        kind: AgendaWorkspaceReceiptKind::DocumentAccepted,
        message: format!("from workspace document {source_file}"),
    }
}

fn section_lookup(sections: &[SectionIndexRecord]) -> BTreeMap<(u32, u32), &SectionIndexRecord> {
    sections
        .iter()
        .map(|section| {
            (
                (section.source.range_start, section.source.range_end),
                section,
            )
        })
        .collect()
}

fn searchable_text(section: &SectionIndexRecord) -> Vec<String> {
    let mut values = vec![section.title.clone()];
    values.extend(section.body.iter().map(|body| body.text.clone()));
    values.extend(
        section
            .properties
            .iter()
            .map(|property| format!("{} {}", property.key, property.value)),
    );
    values
}

fn contains_text(haystack: &str, needle: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        haystack.contains(needle)
    } else {
        haystack
            .to_ascii_lowercase()
            .contains(&needle.to_ascii_lowercase())
    }
}

fn is_project(section: &Section<ParsedAnnotation>) -> bool {
    section.todo.is_some() && section.subsections.iter().any(has_open_todo)
}

fn has_open_todo(section: &Section<ParsedAnnotation>) -> bool {
    section.todo.is_some() && !is_done_keyword(&section.todo)
        || section.subsections.iter().any(has_open_todo)
}

fn has_next_action(section: &Section<ParsedAnnotation>, next_keywords: &[String]) -> bool {
    section.subsections.iter().any(|subsection| {
        is_next_section(subsection, next_keywords) || has_next_action(subsection, next_keywords)
    })
}

fn is_next_section(section: &Section<ParsedAnnotation>, next_keywords: &[String]) -> bool {
    let todo_matches = section.todo.as_ref().is_some_and(|todo| {
        next_keywords
            .iter()
            .any(|keyword| keyword.eq_ignore_ascii_case(&todo.name))
    });
    todo_matches
        || section
            .effective_tags
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case("next"))
}
