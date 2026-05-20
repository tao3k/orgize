use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{
        AgendaDate, AgendaEntryKind, AgendaQuery, IncludeExpansionMode, IncludeExpansionOptions,
        IncludeExpansionPlan, IncludeLineSelection, TimestampKind,
    },
};

const SOURCE: &str = include_str!("../../fixtures/semantic_ast/include-datetree-agenda-extras.org");

#[test]
fn semantic_ast_projects_include_dated_and_agenda_extras() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let include_plan = doc.include_expansion_plan(&IncludeExpansionOptions::with_base_dir("/site"));
    assert_eq!(include_plan.entries.len(), 3);
    assert_eq!(
        include_plan.entries[0].resolved_path.as_deref(),
        Some("/site/partials/header.org")
    );
    assert!(matches!(
        include_plan.entries[0].line_selection,
        IncludeLineSelection::Range {
            start: Some(2),
            end: Some(5),
            ..
        }
    ));
    assert_eq!(include_plan.entries[0].min_level, Some(2));
    assert_eq!(
        include_plan.entries[1].mode,
        IncludeExpansionMode::Source {
            language: Some("rust".to_string())
        }
    );
    assert!(matches!(
        include_plan.entries[2].line_selection,
        IncludeLineSelection::Invalid { .. }
    ));

    let datetree = doc.datetree_entries();
    assert_eq!(datetree.len(), 1);
    assert_eq!(datetree[0].date, AgendaDate::new(2026, 5, 15));
    assert_eq!(
        datetree[0].outline_path,
        ["2026", "2026-05 May", "2026-05-15 Friday"]
    );

    let default_entries = doc
        .to_bare()
        .agenda_entries(&AgendaQuery::single_day(AgendaDate::new(2026, 5, 15)));
    assert_eq!(default_entries.len(), 1);
    assert_eq!(default_entries[0].timestamp.kind, TimestampKind::Active);

    let opt_in_entries = doc.to_bare().agenda_entries(
        &AgendaQuery::single_day(AgendaDate::new(2026, 5, 15))
            .include_inactive_timestamps(true)
            .include_diary_timestamps(true),
    );
    assert_eq!(opt_in_entries.len(), 3);
    assert!(opt_in_entries.iter().any(|entry| {
        entry.kind == AgendaEntryKind::Diary && entry.timestamp.kind == TimestampKind::Diary
    }));
    assert!(
        opt_in_entries
            .iter()
            .any(|entry| entry.timestamp.kind == TimestampKind::Inactive)
    );

    insta::assert_debug_snapshot!(
        "semantic_ast__semantic_include_expansion_plan",
        include_plan_without_annotations(include_plan)
    );
    insta::assert_debug_snapshot!("semantic_ast__semantic_datetree_entries", datetree);
    insta::assert_debug_snapshot!(
        "semantic_ast__semantic_agenda_inactive_and_diary_entries",
        opt_in_entries
    );
}

fn include_plan_without_annotations(
    plan: IncludeExpansionPlan<orgize::ast::ParsedAnnotation>,
) -> IncludeExpansionPlan<()> {
    IncludeExpansionPlan {
        entries: plan
            .entries
            .into_iter()
            .map(|entry| orgize::ast::IncludeExpansionEntry {
                directive: orgize::ast::IncludeDirective {
                    ann: (),
                    path: entry.directive.path,
                    raw_path: entry.directive.raw_path,
                    arguments: entry.directive.arguments,
                    options: entry.directive.options,
                    raw_value: entry.directive.raw_value,
                },
                resolved_path: entry.resolved_path,
                line_selection: entry.line_selection,
                min_level: entry.min_level,
                mode: entry.mode,
                options: entry.options,
            })
            .collect(),
    }
}
