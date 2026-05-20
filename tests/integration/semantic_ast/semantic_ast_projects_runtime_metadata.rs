use crate::semantic_ast::support::assert_clean_projection;
use orgize::{Org, ast::ParsedAst};

const SOURCE: &str = include_str!("../../fixtures/semantic_ast/m25-runtime-metadata.org");

#[test]
fn semantic_ast_projects_runtime_metadata_plan() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let plan = doc.runtime_metadata_plan();
    assert_eq!(plan.feeds.len(), 1);
    assert_eq!(plan.feeds[0].entry_count, 2);
    assert!(plan.feeds[0].readable);
    assert_eq!(plan.timers.len(), 3);
    assert_eq!(plan.mobile.readonly.len(), 1);
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
