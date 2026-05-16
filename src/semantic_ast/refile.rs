//! Document-local refile target discovery and non-executing plans.

use std::collections::BTreeMap;
use std::path::Path;

use super::{
    Document, ParsedAnnotation, RefileAction, RefileCreateParentNode, RefileCreateParentPlan,
    RefileOutlinePathMode, RefileParentCreationMode, RefilePlan, RefilePlanReceipt,
    RefilePlanReceiptKind, RefilePlanRequest, RefilePlanSection, RefileTarget, RefileTargetIndex,
    RefileTargetQuery, RefileTargetReceipt, RefileTargetSpec, RefileWarning, RefileWarningKind,
    SectionIndexRecord,
};

impl Document<ParsedAnnotation> {
    /// Builds a document-local refile target index without moving source.
    pub fn refile_target_index(&self, query: &RefileTargetQuery) -> RefileTargetIndex {
        let records = refile_records(self, query.source_file.as_deref());
        let specs = query.effective_specs();
        let mut warnings = unsupported_spec_warnings(&specs);
        let mut targets = records
            .iter()
            .filter_map(|record| refile_target(self, query, &specs, record))
            .collect::<Vec<_>>();
        warnings.extend(duplicate_display_warnings(&targets));

        targets.sort_by_key(|target| target.source.range_start);
        RefileTargetIndex {
            source_file: query.source_file.clone(),
            outline_path_mode: query.outline_path_mode,
            specs,
            targets,
            warnings,
        }
    }

    /// Resolves a non-mutating refile plan from one outline path to another.
    pub fn refile_plan(&self, request: &RefilePlanRequest) -> RefilePlan {
        let records = refile_records(self, request.source_file.as_deref());
        let source_matches = matching_outline_records(&records, &request.source_outline_path);
        let target_matches = matching_outline_records(&records, &request.target_outline_path);
        let mut warnings = Vec::new();
        let mut receipts = Vec::new();

        let source = resolve_plan_section(
            request.source_file.clone(),
            &source_matches,
            &request.source_outline_path,
            true,
            &mut warnings,
            &mut receipts,
        );
        let (target, created_target) = resolve_plan_target_or_create(
            self,
            request.source_file.clone(),
            &records,
            request,
            &target_matches,
            &mut warnings,
            &mut receipts,
        );

        receipts.push(RefilePlanReceipt {
            kind: RefilePlanReceiptKind::InsertPositionResolved,
            message: format!(
                "refile insertion intent is `{}` under the target heading",
                request.insert_position.as_str()
            ),
        });
        receipts.push(RefilePlanReceipt {
            kind: RefilePlanReceiptKind::NonMutating,
            message: "orgize reports a refile plan but does not move or rewrite Org source"
                .to_string(),
        });

        if let Some(source) = &source {
            if let Some(target) = &target {
                push_plan_shape_warnings(source, target, request.action, &mut warnings);
            }
            if let Some(created_target) = &created_target {
                push_creation_shape_warnings(source, created_target, request.action, &mut warnings);
            }
        }

        RefilePlan {
            source_file: request.source_file.clone(),
            action: request.action,
            insert_position: request.insert_position,
            parent_creation: request.parent_creation,
            source,
            target,
            created_target,
            receipts,
            warnings,
        }
    }
}

fn refile_records(
    document: &Document<ParsedAnnotation>,
    source_file: Option<&str>,
) -> Vec<SectionIndexRecord> {
    match source_file {
        Some(source_file) => document.section_index_records_for_file(source_file),
        None => document.section_index_records(),
    }
}

fn refile_target(
    document: &Document<ParsedAnnotation>,
    query: &RefileTargetQuery,
    specs: &[RefileTargetSpec],
    record: &SectionIndexRecord,
) -> Option<RefileTarget> {
    let receipts = specs
        .iter()
        .filter_map(|spec| refile_target_receipt(spec, record))
        .collect::<Vec<_>>();
    if receipts.is_empty() {
        return None;
    }
    Some(refile_target_from_record(document, query, record, receipts))
}

fn refile_target_from_record(
    document: &Document<ParsedAnnotation>,
    query: &RefileTargetQuery,
    record: &SectionIndexRecord,
    receipts: Vec<RefileTargetReceipt>,
) -> RefileTarget {
    RefileTarget {
        source_file: query.source_file.clone(),
        source: record.source.clone(),
        level: record.level,
        title: record.title.clone(),
        outline_path: record.outline_path.clone(),
        display: target_display(document, query, record),
        receipts,
    }
}

fn refile_target_receipt(
    spec: &RefileTargetSpec,
    record: &SectionIndexRecord,
) -> Option<RefileTargetReceipt> {
    if !spec_matches_record(spec, record) {
        return None;
    }
    Some(RefileTargetReceipt {
        spec: spec.clone(),
        message: refile_target_receipt_message(spec, record),
    })
}

fn spec_matches_record(spec: &RefileTargetSpec, record: &SectionIndexRecord) -> bool {
    match spec {
        RefileTargetSpec::All => true,
        RefileTargetSpec::Tag(tag) => record.tags.iter().any(|entry| entry == tag),
        RefileTargetSpec::Todo(keyword) => record
            .todo
            .as_ref()
            .is_some_and(|todo| todo.name == *keyword),
        RefileTargetSpec::Level(level) => record.level == *level,
        RefileTargetSpec::MaxLevel(level) => record.level <= *level,
        RefileTargetSpec::Regexp(_) => false,
    }
}

fn refile_target_receipt_message(spec: &RefileTargetSpec, record: &SectionIndexRecord) -> String {
    match spec {
        RefileTargetSpec::All => "all headlines are accepted as refile targets".to_string(),
        RefileTargetSpec::Tag(tag) => {
            format!("headline has local refile target tag `{tag}`")
        }
        RefileTargetSpec::Todo(keyword) => {
            format!("headline has TODO keyword `{keyword}`")
        }
        RefileTargetSpec::Level(level) => {
            format!(
                "headline level {} matches target level {level}",
                record.level
            )
        }
        RefileTargetSpec::MaxLevel(level) => {
            format!(
                "headline level {} is within max level {level}",
                record.level
            )
        }
        RefileTargetSpec::Regexp(pattern) => {
            format!("regexp target `{pattern}` is preserved but not approximated")
        }
    }
}

fn target_display(
    document: &Document<ParsedAnnotation>,
    query: &RefileTargetQuery,
    record: &SectionIndexRecord,
) -> String {
    let outline = record
        .outline_path
        .iter()
        .map(|entry| entry.replace('/', "\\/"))
        .collect::<Vec<_>>();
    match query.outline_path_mode {
        RefileOutlinePathMode::None => record.title.clone(),
        RefileOutlinePathMode::Outline => outline.join("/"),
        RefileOutlinePathMode::File => prefixed_display(source_file_name(query), &outline),
        RefileOutlinePathMode::FullFilePath => {
            prefixed_display(query.source_file.clone(), &outline)
        }
        RefileOutlinePathMode::BufferName => prefixed_display(source_file_name(query), &outline),
        RefileOutlinePathMode::Title => prefixed_display(
            document_title(document).or_else(|| source_file_name(query)),
            &outline,
        ),
    }
}

fn prefixed_display(prefix: Option<String>, outline: &[String]) -> String {
    let mut parts = Vec::new();
    if let Some(prefix) = prefix.filter(|prefix| !prefix.trim().is_empty()) {
        parts.push(prefix);
    }
    parts.extend(outline.iter().cloned());
    parts.join("/")
}

fn source_file_name(query: &RefileTargetQuery) -> Option<String> {
    query.source_file.as_ref().map(|source_file| {
        Path::new(source_file)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(source_file)
            .to_string()
    })
}

fn document_title(document: &Document<ParsedAnnotation>) -> Option<String> {
    let titles = document
        .metadata
        .iter()
        .filter(|keyword| keyword.key.eq_ignore_ascii_case("TITLE"))
        .map(|keyword| keyword.value.trim())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    (!titles.is_empty()).then(|| titles.join(" "))
}

fn unsupported_spec_warnings(specs: &[RefileTargetSpec]) -> Vec<RefileWarning> {
    specs
        .iter()
        .filter_map(|spec| match spec {
            RefileTargetSpec::Regexp(pattern) => Some(RefileWarning {
                kind: RefileWarningKind::UnsupportedRegexp,
                message: format!(
                    "regexp refile target `{pattern}` is preserved but not evaluated; orgize avoids approximating Emacs regexp semantics"
                ),
            }),
            _ => None,
        })
        .collect()
}

fn duplicate_display_warnings(targets: &[RefileTarget]) -> Vec<RefileWarning> {
    let mut by_display = BTreeMap::<&str, usize>::new();
    for target in targets {
        *by_display.entry(target.display.as_str()).or_default() += 1;
    }
    by_display
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(display, count)| RefileWarning {
            kind: RefileWarningKind::DuplicateDisplay,
            message: format!(
                "refile target display `{display}` appears {count} times; use outline/source evidence to disambiguate"
            ),
        })
        .collect()
}

fn matching_outline_records<'a>(
    records: &'a [SectionIndexRecord],
    outline_path: &[String],
) -> Vec<&'a SectionIndexRecord> {
    records
        .iter()
        .filter(|record| record.outline_path == outline_path)
        .collect()
}

fn resolve_plan_section(
    source_file: Option<String>,
    matches: &[&SectionIndexRecord],
    outline_path: &[String],
    is_source: bool,
    warnings: &mut Vec<RefileWarning>,
    receipts: &mut Vec<RefilePlanReceipt>,
) -> Option<RefilePlanSection> {
    match matches {
        [] => {
            warnings.push(RefileWarning {
                kind: if is_source {
                    RefileWarningKind::SourceNotFound
                } else {
                    RefileWarningKind::TargetNotFound
                },
                message: format!(
                    "{} outline `{}` was not found in the parsed document",
                    if is_source { "source" } else { "target" },
                    outline_path.join(" / ")
                ),
            });
            None
        }
        [record] => {
            receipts.push(RefilePlanReceipt {
                kind: if is_source {
                    RefilePlanReceiptKind::SourceResolved
                } else {
                    RefilePlanReceiptKind::TargetResolved
                },
                message: format!(
                    "{} outline `{}` resolved to line {}",
                    if is_source { "source" } else { "target" },
                    outline_path.join(" / "),
                    record.source.start.line
                ),
            });
            Some(RefilePlanSection {
                source_file,
                source: record.source.clone(),
                level: record.level,
                title: record.title.clone(),
                outline_path: record.outline_path.clone(),
                local_ids: local_identity_values(record),
            })
        }
        _ => {
            warnings.push(RefileWarning {
                kind: if is_source {
                    RefileWarningKind::AmbiguousSource
                } else {
                    RefileWarningKind::AmbiguousTarget
                },
                message: format!(
                    "{} outline `{}` matched {} sections",
                    if is_source { "source" } else { "target" },
                    outline_path.join(" / "),
                    matches.len()
                ),
            });
            None
        }
    }
}

fn resolve_plan_target(
    document: &Document<ParsedAnnotation>,
    source_file: Option<String>,
    matches: &[&SectionIndexRecord],
    outline_path: &[String],
    warnings: &mut Vec<RefileWarning>,
    receipts: &mut Vec<RefilePlanReceipt>,
) -> Option<RefileTarget> {
    let section = resolve_plan_section(
        source_file.clone(),
        matches,
        outline_path,
        false,
        warnings,
        receipts,
    )?;
    let mut query = RefileTargetQuery::new().outline_path_mode(RefileOutlinePathMode::Outline);
    if let Some(source_file) = source_file {
        query = query.source_file(source_file);
    }
    let target_display_text = target_display(
        document,
        &query,
        matches.first().expect("target match should exist"),
    );
    Some(RefileTarget {
        source_file: section.source_file,
        source: section.source,
        level: section.level,
        title: section.title,
        outline_path: section.outline_path.clone(),
        display: prefixed_display(None, &section.outline_path),
        receipts: vec![RefileTargetReceipt {
            spec: RefileTargetSpec::All,
            message: format!(
                "target outline `{}` resolved for the refile plan",
                target_display_text
            ),
        }],
    })
}

fn resolve_plan_target_or_create(
    document: &Document<ParsedAnnotation>,
    source_file: Option<String>,
    records: &[SectionIndexRecord],
    request: &RefilePlanRequest,
    target_matches: &[&SectionIndexRecord],
    warnings: &mut Vec<RefileWarning>,
    receipts: &mut Vec<RefilePlanReceipt>,
) -> (Option<RefileTarget>, Option<RefileCreateParentPlan>) {
    if !target_matches.is_empty() || request.parent_creation == RefileParentCreationMode::Never {
        return (
            resolve_plan_target(
                document,
                source_file,
                target_matches,
                &request.target_outline_path,
                warnings,
                receipts,
            ),
            None,
        );
    }

    let created_target = resolve_created_target_plan(
        document,
        source_file,
        records,
        request.parent_creation,
        &request.target_outline_path,
        warnings,
        receipts,
    );
    (None, created_target)
}

fn resolve_created_target_plan(
    document: &Document<ParsedAnnotation>,
    source_file: Option<String>,
    records: &[SectionIndexRecord],
    mode: RefileParentCreationMode,
    target_outline_path: &[String],
    warnings: &mut Vec<RefileWarning>,
    receipts: &mut Vec<RefilePlanReceipt>,
) -> Option<RefileCreateParentPlan> {
    let (parent_outline_path, child_title) = match target_outline_path
        .split_last()
        .filter(|(_, parent)| !parent.is_empty())
    {
        Some((child_title, parent_outline_path)) => (parent_outline_path, child_title),
        None => {
            warnings.push(RefileWarning {
                kind: RefileWarningKind::ParentNotFound,
                message: format!(
                    "target outline `{}` cannot be created because it has no existing parent path",
                    target_outline_path.join(" / ")
                ),
            });
            return None;
        }
    };
    let parent_matches = matching_outline_records(records, parent_outline_path);
    let parent = resolve_creation_parent(
        document,
        source_file.clone(),
        &parent_matches,
        parent_outline_path,
        warnings,
    )?;
    let node = RefileCreateParentNode {
        title: child_title.clone(),
        level: parent.level + 1,
        outline_path: target_outline_path.to_vec(),
        display: prefixed_display(None, target_outline_path),
    };
    receipts.push(RefilePlanReceipt {
        kind: RefilePlanReceiptKind::ParentCreationPlanned,
        message: format!(
            "missing target outline `{}` is planned as a new child under `{}`",
            target_outline_path.join(" / "),
            parent.outline_path.join(" / ")
        ),
    });
    if mode == RefileParentCreationMode::Confirm {
        receipts.push(RefilePlanReceipt {
            kind: RefilePlanReceiptKind::ParentCreationRequiresConfirmation,
            message: "new parent creation mirrors Org's `confirm` mode; downstream must ask before editing source"
                .to_string(),
        });
    }
    Some(RefileCreateParentPlan {
        source_file,
        existing_parent: parent,
        target_outline_path: target_outline_path.to_vec(),
        nodes: vec![node],
        requires_confirmation: mode == RefileParentCreationMode::Confirm,
    })
}

fn resolve_creation_parent(
    document: &Document<ParsedAnnotation>,
    source_file: Option<String>,
    matches: &[&SectionIndexRecord],
    parent_outline_path: &[String],
    warnings: &mut Vec<RefileWarning>,
) -> Option<RefileTarget> {
    match matches {
        [] => {
            warnings.push(RefileWarning {
                kind: RefileWarningKind::ParentNotFound,
                message: format!(
                    "new refile target parent `{}` was not found in the parsed document",
                    parent_outline_path.join(" / ")
                ),
            });
            None
        }
        [record] => {
            let mut query =
                RefileTargetQuery::new().outline_path_mode(RefileOutlinePathMode::Outline);
            if let Some(source_file) = source_file {
                query = query.source_file(source_file);
            }
            Some(refile_target_from_record(
                document,
                &query,
                record,
                vec![RefileTargetReceipt {
                    spec: RefileTargetSpec::All,
                    message: format!(
                        "existing parent outline `{}` resolved for missing target creation",
                        parent_outline_path.join(" / ")
                    ),
                }],
            ))
        }
        _ => {
            warnings.push(RefileWarning {
                kind: RefileWarningKind::AmbiguousParent,
                message: format!(
                    "new refile target parent `{}` matched {} sections",
                    parent_outline_path.join(" / "),
                    matches.len()
                ),
            });
            None
        }
    }
}

fn local_identity_values(record: &SectionIndexRecord) -> Vec<String> {
    record
        .properties
        .iter()
        .filter(|property| {
            property.key.eq_ignore_ascii_case("ID")
                || property.key.eq_ignore_ascii_case("CUSTOM_ID")
        })
        .map(|property| property.value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn push_plan_shape_warnings(
    source: &RefilePlanSection,
    target: &RefileTarget,
    action: RefileAction,
    warnings: &mut Vec<RefileWarning>,
) {
    if source.source.range_start == target.source.range_start
        && source.source.range_end == target.source.range_end
    {
        warnings.push(RefileWarning {
            kind: RefileWarningKind::SameSourceAndTarget,
            message: "source and target resolve to the same heading; org-refile would not move into itself"
                .to_string(),
        });
    } else if target.source.range_start > source.source.range_start
        && target.source.range_end <= source.source.range_end
    {
        warnings.push(RefileWarning {
            kind: RefileWarningKind::TargetInsideSource,
            message: "target is inside the source subtree; org-refile rejects moving a subtree into itself"
                .to_string(),
        });
    }

    if action == RefileAction::Copy && !source.local_ids.is_empty() {
        push_copy_identity_warning(source, warnings);
    }
}

fn push_creation_shape_warnings(
    source: &RefilePlanSection,
    created_target: &RefileCreateParentPlan,
    action: RefileAction,
    warnings: &mut Vec<RefileWarning>,
) {
    let parent = &created_target.existing_parent;
    if parent.source.range_start >= source.source.range_start
        && parent.source.range_end <= source.source.range_end
    {
        warnings.push(RefileWarning {
            kind: RefileWarningKind::TargetInsideSource,
            message: "planned new target would be created inside the source subtree; org-refile rejects moving a subtree into itself"
                .to_string(),
        });
    }

    if action == RefileAction::Copy && !source.local_ids.is_empty() {
        push_copy_identity_warning(source, warnings);
    }
}

fn push_copy_identity_warning(source: &RefilePlanSection, warnings: &mut Vec<RefileWarning>) {
    warnings.push(RefileWarning {
        kind: RefileWarningKind::CopyMayDuplicateId,
        message: format!(
            "copying this subtree may duplicate local ID/CUSTOM_ID values: {}",
            source.local_ids.join(", ")
        ),
    });
}
