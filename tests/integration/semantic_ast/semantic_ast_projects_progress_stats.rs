use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{ProgressStatisticCookieKind, ProgressTodoState, TaskDependencyKind},
};

const SOURCE: &str = r#"* TODO Parent [1/2] [50%]
:PROPERTIES:
:Effort: 1:00
:ORDERED: t
:END:
- [X] done item
- [ ] open item
- [-] partial item
** DONE Child done
:PROPERTIES:
:Effort: 0:30
:END:
** TODO Child open [0/1]
:PROPERTIES:
:Effort: 2h
:END:
- [ ] child checkbox
*** DONE Nested done
:PROPERTIES:
:Effort: 0:15
:END:
"#;

#[test]
fn semantic_ast_projects_progress_stats_for_agent_planning() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let records = doc.progress_stats_records();
    assert_eq!(records.len(), 4);

    let parent = records
        .iter()
        .find(|record| record.title == "Parent [1/2] [50%]")
        .expect("parent progress record");
    assert_eq!(parent.todo, ProgressTodoState::Todo);
    assert_eq!(parent.descendant_todos.total, 3);
    assert_eq!(parent.descendant_todos.done, 2);
    assert_eq!(parent.descendant_todos.open, 1);
    assert_eq!(parent.checkboxes.total, 4);
    assert_eq!(parent.checkboxes.checked, 1);
    assert_eq!(parent.checkboxes.unchecked, 2);
    assert_eq!(parent.checkboxes.partial, 1);
    assert_eq!(parent.statistic_cookies.len(), 2);
    assert!(parent.statistic_cookies.iter().any(|cookie| {
        cookie.kind == ProgressStatisticCookieKind::Fraction
            && cookie.done == Some(1)
            && cookie.total == Some(2)
    }));
    assert!(parent.statistic_cookies.iter().any(|cookie| {
        cookie.kind == ProgressStatisticCookieKind::Percent && cookie.percent == Some(50)
    }));
    assert_eq!(parent.effort.local.as_ref().unwrap().total_seconds, 3_600);
    assert_eq!(parent.effort.subtree_total_seconds, 13_500);
    assert!(parent.dependencies.iter().any(|dependency| dependency.kind
        == TaskDependencyKind::OpenDescendantTodo
        && dependency.count == 1));
    assert!(parent.dependencies.iter().any(|dependency| dependency.kind
        == TaskDependencyKind::OpenCheckbox
        && dependency.count == 3));
    assert!(
        parent
            .dependencies
            .iter()
            .any(|dependency| dependency.kind == TaskDependencyKind::OrderedProperty)
    );

    let done_child = records
        .iter()
        .find(|record| record.title == "Child done")
        .expect("done child progress record");
    assert_eq!(done_child.todo, ProgressTodoState::Done);
    assert_eq!(done_child.descendant_todos.total, 0);
    assert_eq!(
        done_child.effort.local.as_ref().unwrap().total_seconds,
        1_800
    );
    assert_eq!(done_child.effort.subtree_total_seconds, 1_800);
    assert!(done_child.dependencies.is_empty());

    let open_child = records
        .iter()
        .find(|record| record.title == "Child open [0/1]")
        .expect("open child progress record");
    assert_eq!(open_child.todo, ProgressTodoState::Todo);
    assert_eq!(open_child.descendant_todos.total, 1);
    assert_eq!(open_child.descendant_todos.done, 1);
    assert_eq!(open_child.checkboxes.total, 1);
    assert_eq!(open_child.checkboxes.unchecked, 1);
    assert_eq!(
        open_child.effort.local.as_ref().unwrap().total_seconds,
        7_200
    );
    assert_eq!(open_child.effort.subtree_total_seconds, 8_100);
    assert_eq!(open_child.statistic_cookies.len(), 1);
    assert_eq!(open_child.statistic_cookies[0].done, Some(0));
    assert_eq!(open_child.statistic_cookies[0].total, Some(1));
    assert_eq!(open_child.dependencies.len(), 1);
    assert_eq!(
        open_child.dependencies[0].kind,
        TaskDependencyKind::OpenCheckbox
    );

    let nested_done = records
        .iter()
        .find(|record| record.title == "Nested done")
        .expect("nested done progress record");
    assert_eq!(nested_done.todo, ProgressTodoState::Done);
    assert_eq!(
        nested_done.effort.local.as_ref().unwrap().total_seconds,
        900
    );
}
