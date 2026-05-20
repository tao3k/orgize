use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{TableVisualizationPlan, TableVisualizationWarningKind},
};

const SOURCE: &str = include_str!("../../fixtures/semantic_ast/m25-table-visualization.org");

#[test]
fn semantic_ast_projects_table_visualization_plans() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let plans = doc.table_visualization_plans();
    assert_eq!(plans.len(), 3);
    assert_eq!(
        plans[0].plot.as_ref().and_then(|plot| plot.index_column),
        Some(1)
    );
    assert_eq!(
        plans[0]
            .plot
            .as_ref()
            .map(|plot| plot.dependent_columns.as_slice()),
        Some(&[3, 4][..])
    );
    assert!(
        plans[1]
            .warnings
            .iter()
            .any(|warning| warning.kind == TableVisualizationWarningKind::InvalidPlotOption)
    );
    assert_eq!(
        plans[2]
            .radio
            .as_ref()
            .and_then(|radio| radio.translator.as_deref()),
        Some("orgtbl-to-latex")
    );
    assert!(
        plans[2]
            .radio
            .as_ref()
            .and_then(|radio| radio.receiver.as_ref())
            .is_some_and(|receiver| receiver.begin_found && receiver.end_found)
    );

    insta::assert_snapshot!(
        "semantic_ast__m25_table_visualization_plans",
        render_table_visualization_plans(&plans)
    );
}

fn render_table_visualization_plans(
    plans: &[TableVisualizationPlan<orgize::ast::ParsedAnnotation>],
) -> String {
    let mut out = String::new();
    out.push_str(&format!("plans={}\n", plans.len()));
    for plan in plans {
        out.push_str(&format!(
            "table {} kind={} rows={} cols={} header={}\n",
            plan.table_index,
            plan.kind.as_str(),
            plan.row_count,
            plan.column_count,
            plan.header.join("|")
        ));
        if let Some(plot) = &plan.plot {
            out.push_str(&format!(
                "  plot title={} type={} with={} file={} ind={} deps={} transpose={}\n",
                plot.title.as_deref().unwrap_or("none"),
                plot.plot_type
                    .as_ref()
                    .map(|plot_type| plot_type.as_str())
                    .unwrap_or("none"),
                plot.with.as_deref().unwrap_or("none"),
                plot.file.as_deref().unwrap_or("none"),
                plot.index_column
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                plot.dependent_columns
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                plot.transpose
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string())
            ));
            for option in &plot.options {
                out.push_str(&format!(
                    "    option {} {}={}\n",
                    option.kind.as_str(),
                    option.key,
                    option.value.as_deref().unwrap_or("none")
                ));
            }
        }
        if let Some(radio) = &plan.radio {
            out.push_str(&format!(
                "  radio {} translator={} receiver={}\n",
                radio.name,
                radio.translator.as_deref().unwrap_or("none"),
                radio
                    .receiver
                    .as_ref()
                    .map(|receiver| format!(
                        "{}:{}:{}",
                        receiver.name, receiver.begin_found, receiver.end_found
                    ))
                    .unwrap_or_else(|| "none".to_string())
            ));
            for parameter in &radio.parameters {
                out.push_str(&format!(
                    "    param {} {}={}\n",
                    parameter.kind.as_str(),
                    parameter.key,
                    parameter.value.as_deref().unwrap_or("none")
                ));
            }
        }
        for warning in &plan.warnings {
            out.push_str(&format!(
                "  warning {} {}\n",
                warning.kind.as_str(),
                warning.message
            ));
        }
    }
    out
}
