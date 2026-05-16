use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{
        ColumnSummaryOperatorKind, ColumnSummaryPlan, ColumnSummaryStatus,
        ColumnSummaryValueSource, ColumnViewScope,
    },
    Org as OrgParser,
};

#[test]
fn semantic_ast_projects_document_and_section_column_views() {
    let doc = OrgParser::parse(
        r#"#+COLUMNS: %25ITEM(Task) %TODO %3PRIORITY %Effort{:}

* Project
:PROPERTIES:
:COLUMNS: %ITEM %Effort(Effort){:}
:END:
"#,
    )
    .document();

    assert_clean_projection(&doc);
    let records = doc.column_view_records();
    assert_eq!(records.len(), 2);

    assert_eq!(records[0].scope, ColumnViewScope::DocumentKeyword);
    assert_eq!(records[0].columns[0].property, "ITEM");
    assert_eq!(records[0].columns[0].width, Some(25));
    assert_eq!(records[0].columns[0].title.as_deref(), Some("Task"));
    assert_eq!(records[0].columns[3].summary_operator.as_deref(), Some(":"));

    match &records[1].scope {
        ColumnViewScope::SectionProperty {
            level,
            title,
            outline_path,
        } => {
            assert_eq!(*level, 1);
            assert_eq!(title, "Project");
            assert_eq!(outline_path, &["Project"]);
        }
        other => panic!("expected section-scoped COLUMNS property, got {other:#?}"),
    }
    assert_eq!(records[1].columns[1].title.as_deref(), Some("Effort"));
}

#[test]
fn semantic_ast_projects_column_summary_plans() {
    let doc = OrgParser::parse(
        r#"#+COLUMNS: %25ITEM %Score{+;%.1f} %Effort{:} %Approved{X} %Progress{X%} %Estimate{est+} %TODO{+}

* Project
:PROPERTIES:
:Score: 100
:END:
** Task A
:PROPERTIES:
:Score: 2
:Effort: 1:30
:Approved: [X]
:Progress: [X]
:Estimate: 1-3
:END:
** Task B
:PROPERTIES:
:Score: 3.5
:Effort: 45
:Approved: [ ]
:Progress: [ ]
:Estimate: 2-4
:END:
* Scoped
:PROPERTIES:
:COLUMNS: %ITEM %Points{max} %Effort{:mean}
:END:
** Child one
:PROPERTIES:
:Points: 8
:Effort: 0:30
:END:
** Child two
:PROPERTIES:
:Points: 13
:Effort: 1:30
:END:
"#,
    )
    .document();

    assert_clean_projection(&doc);
    let plans = doc.column_summary_plans();
    assert_eq!(plans.len(), 2);

    let document_score = plans[0]
        .summaries
        .iter()
        .find(|summary| summary.column.property == "SCORE")
        .expect("document score summary");
    assert_eq!(document_score.kind, ColumnSummaryOperatorKind::NumericSum);
    assert_eq!(document_score.status, ColumnSummaryStatus::Computed);
    assert_eq!(document_score.value.as_deref(), Some("5.5"));

    let project = &plans[0].rows[0];
    let project_effort = project
        .summaries
        .iter()
        .find(|summary| summary.column.property == "EFFORT")
        .expect("project effort summary");
    assert_eq!(project_effort.value.as_deref(), Some("2:15"));
    let ignored_todo = project
        .summaries
        .iter()
        .find(|summary| summary.column.property == "TODO")
        .expect("TODO special summary");
    assert_eq!(
        ignored_todo.status,
        ColumnSummaryStatus::IgnoredSpecialProperty
    );

    let task_score = &project.children[0].cells[1];
    assert_eq!(task_score.source, ColumnSummaryValueSource::LocalProperty);
    assert_eq!(task_score.value.as_deref(), Some("2"));

    let scoped_points = plans[1]
        .summaries
        .iter()
        .find(|summary| summary.column.property == "POINTS")
        .expect("scoped points summary");
    assert_eq!(scoped_points.kind, ColumnSummaryOperatorKind::NumericMax);
    assert_eq!(scoped_points.value.as_deref(), Some("13"));

    insta::assert_snapshot!(
        "semantic_ast__semantic_column_summary_plans",
        render_column_summary_plans(&plans)
    );
}

fn render_column_summary_plans(plans: &[ColumnSummaryPlan]) -> String {
    let mut output = String::new();
    for (plan_index, plan) in plans.iter().enumerate() {
        output.push_str(&format!(
            "plan {} scope={:?} raw={}\n",
            plan_index + 1,
            plan.declaration.scope,
            plan.declaration.raw
        ));
        for summary in &plan.summaries {
            output.push_str(&format!(
                "  summary {} {} kind={} status={} inputs={}/{} value={}\n",
                summary.column.property,
                summary.operator,
                summary.kind.as_str(),
                summary.status.as_str(),
                summary.parsed_input_count,
                summary.input_count,
                summary.value.as_deref().unwrap_or("<none>")
            ));
        }
        for row in &plan.rows {
            render_column_summary_row(row, 1, &mut output);
        }
        for warning in &plan.warnings {
            output.push_str(&format!(
                "  warning {} {}\n",
                warning.kind.as_str(),
                warning.message
            ));
        }
    }
    output
}

fn render_column_summary_row(
    row: &orgize::ast::ColumnSummaryRow,
    depth: usize,
    output: &mut String,
) {
    let indent = "  ".repeat(depth);
    output.push_str(&format!(
        "{indent}row level={} path={} title={}\n",
        row.level,
        row.outline_path.join(" > "),
        row.title
    ));
    for cell in &row.cells {
        output.push_str(&format!(
            "{indent}  cell {} source={} value={}\n",
            cell.property,
            cell.source.as_str(),
            cell.value.as_deref().unwrap_or("<none>")
        ));
    }
    for summary in &row.summaries {
        if summary.status == ColumnSummaryStatus::NoInputs {
            continue;
        }
        output.push_str(&format!(
            "{indent}  row-summary {} {} kind={} status={} inputs={}/{} value={}\n",
            summary.column.property,
            summary.operator,
            summary.kind.as_str(),
            summary.status.as_str(),
            summary.parsed_input_count,
            summary.input_count,
            summary.value.as_deref().unwrap_or("<none>")
        ));
    }
    for child in &row.children {
        render_column_summary_row(child, depth + 1, output);
    }
}
