//! Predicate, selector, and property field compilation.

use super::core_types::{FieldKind, FieldRef, QueryExpr, list_head};
use crate::ast::{
    OrgElementQueryPredicate, OrgElementsIndexCategory, OrgElementsIndexKind,
    OrgElementsIndexSummaryValue,
};

pub(super) fn compile_predicate_expression(
    expression: &QueryExpr,
) -> Option<OrgElementQueryPredicate> {
    let QueryExpr::List(items) = expression else {
        return None;
    };
    let head = list_head(items)?;
    match head {
        "and" => Some(OrgElementQueryPredicate::all(
            items[1..]
                .iter()
                .map(compile_predicate_expression)
                .collect::<Option<Vec<_>>>()?,
        )),
        "or" => Some(OrgElementQueryPredicate::any(
            items[1..]
                .iter()
                .map(compile_predicate_expression)
                .collect::<Option<Vec<_>>>()?,
        )),
        "not" => Some(OrgElementQueryPredicate::negate(
            compile_predicate_expression(items.get(1)?)?,
        )),
        "=" => compile_field_comparison_predicate(items, false),
        "contains" => compile_field_comparison_predicate(items, true),
        "kind" => Some(OrgElementQueryPredicate::Kind(OrgElementsIndexKind::new(
            items.get(1)?.as_text()?,
        ))),
        "category" => OrgElementsIndexCategory::from_label(&items.get(1)?.as_text()?)
            .map(OrgElementQueryPredicate::Category),
        "summary" => compile_field_shorthand_predicate(items, FieldKind::Summary, false),
        "summary-contains" => compile_field_shorthand_predicate(items, FieldKind::Summary, true),
        "property" => compile_field_shorthand_predicate(items, FieldKind::Property, false),
        "property-contains" => compile_field_shorthand_predicate(items, FieldKind::Property, true),
        _ if items.len() == 1 => Some(OrgElementQueryPredicate::Kind(OrgElementsIndexKind::new(
            head,
        ))),
        _ => None,
    }
}

fn compile_field_comparison_predicate(
    items: &[QueryExpr],
    contains: bool,
) -> Option<OrgElementQueryPredicate> {
    let field = parse_field_ref(items.get(1)?)?;
    let value = items.get(2)?;
    match (field.kind, contains) {
        (FieldKind::Summary, false) => Some(OrgElementQueryPredicate::summary_eq(
            field.key,
            expression_summary_value(value)?,
        )),
        (FieldKind::Summary, true) => Some(OrgElementQueryPredicate::summary_contains(
            field.key,
            value.as_text()?,
        )),
        (FieldKind::Property, false) => Some(OrgElementQueryPredicate::property_eq(
            field.key,
            expression_summary_value(value)?,
        )),
        (FieldKind::Property, true) => Some(OrgElementQueryPredicate::property_contains(
            field.key,
            value.as_text()?,
        )),
    }
}

fn compile_field_shorthand_predicate(
    items: &[QueryExpr],
    field_kind: FieldKind,
    contains: bool,
) -> Option<OrgElementQueryPredicate> {
    let key = items.get(1)?.as_text()?;
    let value = items.get(2)?;
    match (field_kind, contains) {
        (FieldKind::Summary, false) => Some(OrgElementQueryPredicate::summary_eq(
            key,
            expression_summary_value(value)?,
        )),
        (FieldKind::Summary, true) => Some(OrgElementQueryPredicate::summary_contains(
            key,
            value.as_text()?,
        )),
        (FieldKind::Property, false) => Some(OrgElementQueryPredicate::property_eq(
            key,
            expression_summary_value(value)?,
        )),
        (FieldKind::Property, true) => Some(OrgElementQueryPredicate::property_contains(
            key,
            value.as_text()?,
        )),
    }
}

pub(super) fn parse_field_ref(expression: &QueryExpr) -> Option<FieldRef> {
    let QueryExpr::List(items) = expression else {
        return None;
    };
    let kind = match list_head(items)? {
        "summary" => FieldKind::Summary,
        "property" => FieldKind::Property,
        _ => return None,
    };
    Some(FieldRef {
        kind,
        key: items.get(1)?.as_text()?,
    })
}

pub(super) fn expression_summary_value(
    expression: &QueryExpr,
) -> Option<OrgElementsIndexSummaryValue> {
    match expression {
        QueryExpr::String(value) => Some(OrgElementsIndexSummaryValue::Text(value.clone())),
        QueryExpr::Atom(value) => Some(org_elements_query_summary_value(value)),
        QueryExpr::List(_) => None,
    }
}
use super::core::org_elements_query_summary_value;
