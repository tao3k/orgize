use crate::semantic_ast::support::assert_clean_projection;
use orgize::{Org, ast::ParsedAst};

const SOURCE: &str = include_str!("../../fixtures/semantic_ast/m25-runtime-metadata.org");

fn semantic_ast_projects_runtime_metadata_plan() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let plan = doc.runtime_metadata_plan();
    assert_eq!(plan.feeds.len(), 1);
    assert_eq!(plan.feeds[0].entry_count, 2);
    assert!(plan.feeds[0].readable);
    assert_eq!(plan.timers.len(), 3);
    assert_eq!(plan.mobile.readonly.len(), 1);
    assert_eq!(plan.mobile.readonly[0].source.range_start, 0);
    assert_eq!(plan.mobile.readonly[0].source.range_end, 11);
    assert_eq!(plan.mobile.all_priorities[0].values, ["A", "B", "C"]);
    assert_eq!(plan.mobile.index_links.len(), 2);
    assert_eq!(plan.mobile.flagged_sections.len(), 1);
    assert_eq!(plan.mobile.original_ids.len(), 1);
    assert_eq!(plan.boundaries.len(), 4);
    assert!(plan.warnings.is_empty());

    insta::assert_snapshot!(
        "semantic_ast__m25_runtime_metadata_plan",
        render_runtime_metadata_plan(&doc)
    );
}

fn mobile_markers_follow_aot_keywords_inside_sections_not_source_blocks() {
    let doc = Org::parse(
        "* Section\n#+READONLY\n#+ALLPRIORITIES: A B\n#+begin_src org\n#+READONLY\n#+end_src\n",
    )
    .document();
    assert_clean_projection(&doc);
    let plan = doc.runtime_metadata_plan();
    assert_eq!(plan.mobile.readonly.len(), 1);
    assert_eq!(plan.mobile.all_priorities.len(), 1);
    assert_eq!(plan.mobile.all_priorities[0].values, ["A", "B"]);
}

fn feed_status_body_uses_aot_drawer_bounds_with_crlf() {
    let source = "* Inbox\r\n:FEEDSTATUS:\r\n((\"guid\" t \"hash\"))\r\n:FEEDSTATUS:\r\n:END:\r\n";
    let doc = Org::parse(source).document();
    assert_clean_projection(&doc);
    let plan = doc.runtime_metadata_plan();
    assert_eq!(plan.feeds.len(), 1);
    assert_eq!(plan.feeds[0].raw, "((\"guid\" t \"hash\"))\n:FEEDSTATUS:");
    assert_eq!(plan.feeds[0].entry_count, 1);
}

fn render_runtime_metadata_plan(doc: &ParsedAst) -> String {
    let plan = doc.runtime_metadata_plan();
    let mut out = String::new();
    out.push_str(&format!(
        "feeds={} timers={} indexLinks={} flagged={} originalIds={} boundaries={} warnings={}\n",
        plan.feeds.len(),
        plan.timers.len(),
        plan.mobile.index_links.len(),
        plan.mobile.flagged_sections.len(),
        plan.mobile.original_ids.len(),
        plan.boundaries.len(),
        plan.warnings.len()
    ));
    for feed in &plan.feeds {
        out.push_str(&format!(
            "feed section={} drawer={} entries={} readable={}\n",
            feed.section_title,
            feed.drawer.as_str(),
            feed.entry_count,
            feed.readable
        ));
    }
    for timer in &plan.timers {
        out.push_str(&format!(
            "timer {} raw={} seconds={} path={}\n",
            timer.context.as_str(),
            timer.raw,
            timer.total_seconds,
            timer.outline_path.join(" > ")
        ));
    }
    for priorities in &plan.mobile.all_priorities {
        out.push_str(&format!(
            "mobile all-priorities {}\n",
            priorities.values.join(",")
        ));
    }
    for link in &plan.mobile.index_links {
        out.push_str(&format!(
            "mobile index file={} desc={} title={}\n",
            link.file, link.description, link.title
        ));
    }
    for flagged in &plan.mobile.flagged_sections {
        out.push_str(&format!(
            "mobile flagged title={} original={} props={}\n",
            flagged.title,
            flagged.original_id.as_deref().unwrap_or("none"),
            flagged
                .mobile_properties
                .iter()
                .map(|property| format!("{}={}", property.key, property.value))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    for boundary in &plan.boundaries {
        out.push_str(&format!("boundary {}\n", boundary.kind.as_str()));
    }
    for warning in &plan.warnings {
        out.push_str(&format!("warning {}\n", warning.kind.as_str()));
    }
    out
}

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[
    (
        "semantic_ast::semantic_ast_projects_runtime_metadata::semantic_ast_projects_runtime_metadata_plan",
        semantic_ast_projects_runtime_metadata_plan,
    ),
    (
        "semantic_ast::semantic_ast_projects_runtime_metadata::mobile_markers_follow_aot_keywords_inside_sections_not_source_blocks",
        mobile_markers_follow_aot_keywords_inside_sections_not_source_blocks,
    ),
    (
        "semantic_ast::semantic_ast_projects_runtime_metadata::feed_status_body_uses_aot_drawer_bounds_with_crlf",
        feed_status_body_uses_aot_drawer_bounds_with_crlf,
    ),
];
