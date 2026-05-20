use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{
        AgendaDate, AgendaMatchQuery, AgendaTime, AgendaUrgencyIngredientKind, AgendaViewQuery,
        AgendaViewSortKey, AgendaViewSortSpec, AgendaWorkspaceBuilder, AgendaWorkspaceCardKind,
        AgendaWorkspaceCommandKind, AgendaWorkspaceMatchCommand, AgendaWorkspacePlan,
        AgendaWorkspaceQuery, AgentCaptureKind, AgentCaptureMemoryPolicy, AgentCapturePlan,
        AgentCaptureRequest, AgentCaptureSource, AgentCaptureSourceKind, AgentCaptureTarget,
        AgentCaptureTimestamp, AttachmentInventoryOptions, AttachmentVcsStatus, CitationExportPlan,
        ExportDependencyDiagnosticKind, ExportDependencyEdgeKind, ExportDependencyGraph,
        ExportDependencyGraphOptions, PublishingProjectConfig, PublishingProjectPlan,
        agent_capture_plan, export_dependency_graph, publishing_project_plan,
    },
};

const WORKSPACE_A: &str = include_str!("../../fixtures/semantic_ast/m25-workspace-a.org");
const WORKSPACE_B: &str = include_str!("../../fixtures/semantic_ast/m25-workspace-b.org");
const DEPENDENCY_A: &str = include_str!("../../fixtures/semantic_ast/m25-dependency-a.org");
const DEPENDENCY_B: &str = include_str!("../../fixtures/semantic_ast/m25-dependency-b.org");

#[test]
fn semantic_ast_projects_m25_workspace_agenda_and_workflow_plans() {
    let doc_a = Org::parse(WORKSPACE_A).document();
    let doc_b = Org::parse(WORKSPACE_B).document();
    assert_clean_projection(&doc_a);
    assert_clean_projection(&doc_b);

    let mut builder = AgendaWorkspaceBuilder::new();
    builder
        .add_document("m25-workspace-a.org", &doc_a)
        .add_document("m25-workspace-b.org", &doc_b);

    let query = AgendaWorkspaceQuery::new()
        .command(
            "daily",
            AgendaWorkspaceCommandKind::Agenda(
                AgendaViewQuery::single_day(AgendaDate::new(2026, 5, 16))
                    .sort_by(AgendaViewSortSpec::down(AgendaViewSortKey::Priority))
                    .limit(3),
            ),
        )
        .command(
            "open-todos",
            AgendaWorkspaceCommandKind::TodoList {
                include_done: false,
            },
        )
        .command(
            "work-match",
            AgendaWorkspaceCommandKind::Match(AgendaWorkspaceMatchCommand::new(
                AgendaMatchQuery::parse("+work").expect("valid match"),
            )),
        )
        .command(
            "text-search",
            AgendaWorkspaceCommandKind::Search {
                needle: "Citation".to_string(),
                case_sensitive: false,
            },
        )
        .command(
            "stuck-projects",
            AgendaWorkspaceCommandKind::StuckProjects {
                next_keywords: vec!["NEXT".to_string()],
            },
        );
    let plan = builder.finish(&query);

    assert_eq!(plan.documents.len(), 2);
    assert!(plan.commands.iter().any(|command| {
        command.name == "stuck-projects"
            && command
                .cards
                .iter()
                .any(|card| card.title == "Stuck project")
    }));
    assert!(plan.commands.iter().any(|command| {
        command.name == "daily"
            && command.cards.iter().any(|card| {
                card.kind == AgendaWorkspaceCardKind::Agenda
                    && card
                        .urgency
                        .score_for(AgendaUrgencyIngredientKind::Priority)
                        > 0
            })
    }));

    insta::assert_snapshot!(
        "semantic_ast__m25_workspace_agenda_plan",
        render_workspace_plan(&plan)
    );
}

#[test]
fn semantic_ast_projects_m25_citation_capture_publishing_and_attachments() {
    let doc_a = Org::parse(WORKSPACE_A).document();
    let doc_b = Org::parse(WORKSPACE_B).document();
    assert_clean_projection(&doc_a);
    assert_clean_projection(&doc_b);

    let citations = doc_a.citation_export_plan();
    assert_eq!(
        citations.bibliographies[0].files,
        ["refs.bib", "extra.json"]
    );
    assert!(
        citations
            .citations
            .iter()
            .any(|citation| citation.keys == ["doe2024", "roe2025"])
    );
    insta::assert_snapshot!(
        "semantic_ast__m25_citation_export_plan",
        render_citation_plan(&citations)
    );

    let capture = agent_capture_plan(
        &AgentCaptureRequest::new(
            AgentCaptureKind::Idea,
            "Agent capture should target Agent decisions",
        )
        .body("The user corrected the design: capture is no longer a human-template object.")
        .target(
            AgentCaptureTarget::datetree(AgendaDate::new(2026, 5, 16))
                .source_file("notes/inbox.org"),
        )
        .source(AgentCaptureSource {
            kind: AgentCaptureSourceKind::Article,
            actor: Some("user".to_string()),
            uri: Some("https://example.test/agent-capture".to_string()),
            label: Some("Agent capture article".to_string()),
        })
        .captured_at(AgentCaptureTimestamp::with_time(
            AgendaDate::new(2026, 5, 16),
            AgendaTime {
                hour: 10,
                minute: 24,
            },
        ))
        .tag("idea")
        .tag("memory_candidate")
        .property("decision impact", "high")
        .quote("Selected source excerpt goes here.")
        .memory_policy(AgentCaptureMemoryPolicy::Candidate),
    );
    assert!(capture.org_entry.contains(":CAPTURE_KIND: idea"));
    assert!(capture.org_entry.contains("#+begin_quote"));
    insta::assert_snapshot!(
        "semantic_ast__m25_agent_capture_plan",
        render_agent_capture_plan(&capture)
    );

    let publishing = publishing_project_plan(
        PublishingProjectConfig::new("blog", "blog", "public").sitemap(true),
        [("blog/index.org", &doc_a), ("blog/notes.org", &doc_b)],
    );
    assert!(
        publishing
            .dependencies
            .iter()
            .any(|dependency| dependency.target == "intro.org")
    );
    assert!(publishing.sitemap.is_some());
    insta::assert_snapshot!(
        "semantic_ast__m25_publishing_project_plan",
        render_publishing_plan(&publishing)
    );

    let temp = unique_temp_dir("orgize-m25-attachments");
    fs::create_dir_all(temp.join("assets")).expect("create attachment directory");
    fs::write(temp.join("assets/plan.pdf"), b"plan").expect("write attachment file");
    let inventory = doc_a.attachment_inventory(&AttachmentInventoryOptions::new(path_str(&temp)));
    assert!(
        inventory
            .entries
            .iter()
            .any(|entry| entry.path == "assets" && entry.exists)
    );
    assert!(inventory.entries.iter().any(|entry| {
        entry.path == "missing.pdf"
            && !entry.exists
            && entry.vcs.status == AttachmentVcsStatus::NotChecked
    }));
    insta::assert_snapshot!(
        "semantic_ast__m25_attachment_inventory",
        render_attachment_inventory(&inventory)
    );
    let _ = fs::remove_dir_all(temp);
}

#[test]
fn semantic_ast_projects_m25_export_dependency_graph() {
    let doc_a = Org::parse(DEPENDENCY_A).document();
    let doc_b = Org::parse(DEPENDENCY_B).document();
    assert_clean_projection(&doc_a);
    assert_clean_projection(&doc_b);

    let graph = export_dependency_graph(
        [("site/a.org", &doc_a), ("site/b.org", &doc_b)],
        &ExportDependencyGraphOptions::new().validate_paths(true),
    );
    assert!(graph.edges.iter().any(|edge| {
        edge.kind == ExportDependencyEdgeKind::Includes
            && edge.source == "doc:site/a.org"
            && edge.target == "doc:site/b.org"
    }));
    assert!(
        graph
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == ExportDependencyDiagnosticKind::DependencyCycle)
    );
    assert!(graph.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ExportDependencyDiagnosticKind::MissingMacroDefinition
            && diagnostic.subject == "missing_macro"
    }));

    insta::assert_snapshot!(
        "semantic_ast__m25_export_dependency_graph",
        render_export_dependency_graph(&graph)
    );
}

fn render_workspace_plan(plan: &AgendaWorkspacePlan) -> String {
    let mut out = String::new();
    out.push_str(&format!("documents={}\n", plan.documents.len()));
    for command in &plan.commands {
        out.push_str(&format!(
            "command {} kind={} cards={} skipped={} total={}\n",
            command.name,
            command.kind,
            command.cards.len(),
            command.skipped.len(),
            command.total_candidates
        ));
        for card in &command.cards {
            out.push_str(&format!(
                "  card {} {} kind={} urgency={} todo={} path={}\n",
                card.source_file,
                card.title,
                card.kind.as_str(),
                card.urgency.total,
                card.todo
                    .as_ref()
                    .map(|todo| todo.name.as_str())
                    .unwrap_or("none"),
                card.outline_path.join(" > ")
            ));
        }
    }
    out
}

fn render_citation_plan(plan: &CitationExportPlan<orgize::ast::ParsedAnnotation>) -> String {
    let mut out = String::new();
    for bibliography in &plan.bibliographies {
        out.push_str(&format!("bibliography {}\n", bibliography.files.join(",")));
    }
    for processor in &plan.processors {
        out.push_str(&format!(
            "processor {} style={}\n",
            processor.processor,
            processor.style.as_deref().unwrap_or("none")
        ));
    }
    for print in &plan.print_bibliographies {
        out.push_str(&format!("print options={}\n", print.options.len()));
    }
    for citation in &plan.citations {
        out.push_str(&format!(
            "citation style={} variant={} nocite={} keys={}\n",
            citation.style,
            citation.variant,
            citation.nocite,
            citation.keys.join(",")
        ));
    }
    for warning in &plan.warnings {
        out.push_str(&format!("warning {}\n", warning.kind.as_str()));
    }
    out
}

fn render_agent_capture_plan(plan: &AgentCapturePlan) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "target kind={} file={} path={} date={} position={} confirm={}\n",
        plan.target.kind.as_str(),
        plan.target.source_file.as_deref().unwrap_or("none"),
        if plan.target.outline_path.is_empty() {
            "none".to_string()
        } else {
            plan.target.outline_path.join(" > ")
        },
        plan.target
            .date
            .map(|date| format!("{:04}-{:02}-{:02}", date.year, date.month, date.day))
            .unwrap_or_else(|| "none".to_string()),
        plan.target.insert_position.as_str(),
        plan.requires_confirmation
    ));
    out.push_str("entry:\n");
    out.push_str(&plan.org_entry);
    out.push_str("application:\n");
    out.push_str(&format!(
        "  action={} target={} file={} position={}\n",
        plan.application.action.as_str(),
        plan.application.target.kind.as_str(),
        plan.application
            .target
            .source_file
            .as_deref()
            .unwrap_or("none"),
        plan.application.target.insert_position.as_str()
    ));
    for precondition in &plan.application.preconditions {
        out.push_str(&format!(
            "  precondition {} {}\n",
            precondition.kind.as_str(),
            precondition.message
        ));
    }
    out.push_str("receipts:\n");
    for receipt in &plan.receipts {
        out.push_str(&format!(
            "  {} {}\n",
            receipt.kind.as_str(),
            receipt.message
        ));
    }
    out.push_str("warnings:\n");
    for warning in &plan.warnings {
        out.push_str(&format!(
            "  {} {}\n",
            warning.kind.as_str(),
            warning.message
        ));
    }
    out
}

fn render_publishing_plan(plan: &PublishingProjectPlan) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "project {} docs={} deps={} sitemap={}\n",
        plan.config.name,
        plan.documents.len(),
        plan.dependencies.len(),
        plan.sitemap.is_some()
    ));
    for document in &plan.documents {
        out.push_str(&format!(
            "  doc {} -> {} title={}\n",
            document.source_file,
            document.output_file,
            document.title.as_deref().unwrap_or("none")
        ));
    }
    for dependency in &plan.dependencies {
        out.push_str(&format!(
            "  dep {} {} {}\n",
            dependency.source_file,
            dependency.kind.as_str(),
            dependency.target
        ));
    }
    if let Some(sitemap) = &plan.sitemap {
        out.push_str(&format!(
            "  sitemap {} entries={}\n",
            sitemap.output_file,
            sitemap.entries.len()
        ));
    }
    out
}

fn render_attachment_inventory(inventory: &orgize::ast::AttachmentInventory) -> String {
    let mut out = String::new();
    for entry in &inventory.entries {
        out.push_str(&format!(
            "entry {} title={} exists={} vcs={}\n",
            entry.kind.as_str(),
            entry.section_title,
            entry.exists,
            entry.vcs.status.as_str()
        ));
        out.push_str(&format!("  path={}\n", entry.path));
    }
    for warning in &inventory.warnings {
        out.push_str(&format!(
            "warning {} {}\n",
            warning.kind.as_str(),
            warning.message
        ));
    }
    out
}

fn render_export_dependency_graph(graph: &ExportDependencyGraph) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "nodes={} edges={} diagnostics={}\n",
        graph.nodes.len(),
        graph.edges.len(),
        graph.diagnostics.len()
    ));
    for node in &graph.nodes {
        out.push_str(&format!(
            "node {} {} {}\n",
            node.id,
            node.kind.as_str(),
            node.label
        ));
    }
    for edge in &graph.edges {
        out.push_str(&format!(
            "edge {} {} {}\n",
            edge.source,
            edge.kind.as_str(),
            edge.target
        ));
    }
    for diagnostic in &graph.diagnostics {
        out.push_str(&format!(
            "diagnostic {} {} {}\n",
            diagnostic.kind.as_str(),
            diagnostic.subject,
            diagnostic.message
        ));
    }
    out
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{label}-{pid}-{nanos}"))
}

fn path_str(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
