use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{
        RefileAction, RefileInsertPosition, RefileOutlinePathMode, RefileParentCreationMode,
        RefilePlanReceiptKind, RefilePlanRequest, RefileTargetQuery, RefileWarningKind,
    },
};

#[test]
fn semantic_ast_projects_refile_target_index_and_plan() {
    let doc = Org::parse(
        r#"#+TITLE: Work Notes
#+TODO: TODO WAIT | DONE
* Inbox :inbox:
** TODO Capture
:PROPERTIES:
:ID: task-1
:END:
* Projects
** TODO Project A :project:
** WAIT Waiting Room
"#,
    )
    .document();
    assert_clean_projection(&doc);

    let default_index = doc.refile_target_index(&RefileTargetQuery::new());
    assert_eq!(default_index.targets.len(), 2);
    assert_eq!(default_index.targets[0].title, "Inbox");
    assert_eq!(default_index.targets[1].title, "Projects");
    assert_eq!(
        default_index.targets[0].receipts[0].spec.value().as_deref(),
        Some("1")
    );

    let query = RefileTargetQuery::new()
        .source_file("notes/work.org")
        .outline_path_mode(RefileOutlinePathMode::File)
        .tag("project")
        .todo("WAIT");
    let index = doc.refile_target_index(&query);
    assert!(index.warnings.is_empty());
    assert_eq!(index.targets.len(), 2);
    assert_eq!(index.targets[0].display, "work.org/Projects/Project A");
    assert_eq!(index.targets[0].receipts[0].spec.kind().as_str(), "tag");
    assert_eq!(index.targets[1].display, "work.org/Projects/Waiting Room");
    assert_eq!(index.targets[1].receipts[0].spec.kind().as_str(), "todo");

    let regexp_index = doc.refile_target_index(&RefileTargetQuery::new().regexp("^\\* TODO"));
    assert!(regexp_index.targets.is_empty());
    assert_eq!(
        regexp_index.warnings[0].kind,
        RefileWarningKind::UnsupportedRegexp
    );

    let plan = doc.refile_plan(
        &RefilePlanRequest::new(["Inbox", "Capture"], ["Projects", "Project A"])
            .source_file("notes/work.org")
            .action(RefileAction::Copy)
            .insert_position(RefileInsertPosition::FirstChild),
    );
    assert_eq!(plan.source.as_ref().unwrap().title, "Capture");
    assert_eq!(plan.target.as_ref().unwrap().title, "Project A");
    assert_eq!(plan.action, RefileAction::Copy);
    assert_eq!(plan.insert_position, RefileInsertPosition::FirstChild);
    assert!(
        plan.warnings
            .iter()
            .any(|warning| warning.kind == RefileWarningKind::CopyMayDuplicateId)
    );
    assert!(
        plan.receipts
            .iter()
            .any(|receipt| receipt.kind.as_str() == "nonMutating")
    );

    let invalid_plan = doc.refile_plan(&RefilePlanRequest::new(["Inbox"], ["Inbox", "Capture"]));
    assert!(
        invalid_plan
            .warnings
            .iter()
            .any(|warning| warning.kind == RefileWarningKind::TargetInsideSource)
    );
}

#[test]
fn semantic_ast_projects_refile_plans_single_missing_parent_node_creation() {
    let doc = Org::parse(
        r#"* Inbox
** TODO Capture
* Projects
** Project A
"#,
    )
    .document();

    let plan = doc.refile_plan(
        &RefilePlanRequest::new(["Inbox", "Capture"], ["Projects", "Project B"])
            .confirm_creating_parent_nodes(),
    );
    assert_eq!(plan.parent_creation, RefileParentCreationMode::Confirm);
    assert!(plan.target.is_none());
    assert!(plan.warnings.is_empty());

    let created = plan.created_target.as_ref().expect("created target plan");
    assert_eq!(created.existing_parent.title, "Projects");
    assert_eq!(created.target_outline_path, ["Projects", "Project B"]);
    assert!(created.requires_confirmation);
    assert_eq!(created.nodes.len(), 1);
    assert_eq!(created.nodes[0].title, "Project B");
    assert_eq!(created.nodes[0].level, 2);
    assert_eq!(created.nodes[0].display, "Projects/Project B");
    assert!(
        plan.receipts
            .iter()
            .any(|receipt| receipt.kind == RefilePlanReceiptKind::ParentCreationPlanned)
    );
    assert!(
        plan.receipts.iter().any(
            |receipt| receipt.kind == RefilePlanReceiptKind::ParentCreationRequiresConfirmation
        )
    );

    let missing_parent = doc.refile_plan(
        &RefilePlanRequest::new(["Inbox", "Capture"], ["Projects", "Nested", "Project C"])
            .allow_creating_parent_nodes(),
    );
    assert!(missing_parent.created_target.is_none());
    assert!(
        missing_parent
            .warnings
            .iter()
            .any(|warning| warning.kind == RefileWarningKind::ParentNotFound)
    );

    let inside_source = doc.refile_plan(
        &RefilePlanRequest::new(["Inbox"], ["Inbox", "New child"]).allow_creating_parent_nodes(),
    );
    assert!(
        inside_source
            .warnings
            .iter()
            .any(|warning| warning.kind == RefileWarningKind::TargetInsideSource)
    );
}
