//! Lowering from Org elements query expressions to `OrgElementsIndexQuery`.

use std::collections::BTreeSet;

use super::{
    FieldKind, QueryExpr, apply_org_elements_query_kind, compile_predicate_expression,
    expression_summary_value, list_head, parse_field_ref,
};
use crate::ast::{
    OrgContractQuery, OrgElementId, OrgElementQueryPredicate, OrgElementsIndexCategory,
    OrgElementsIndexQuery, OrgElementsIndexRelation, OrgElementsIndexSummaryPredicate,
    OrgElementsIndexSummaryTextPredicate, OrgElementsIndexSummaryValue,
};

pub(super) fn compile_index_query_expressions(
    expressions: &[QueryExpr],
) -> Option<OrgElementsIndexQuery> {
    match expressions {
        [expression] => compile_index_query_expression(expression),
        [] => None,
        expressions => compile_index_and_query(expressions),
    }
}

fn compile_index_query_expression(expression: &QueryExpr) -> Option<OrgElementsIndexQuery> {
    let QueryExpr::List(items) = expression else {
        return None;
    };
    let head = list_head(items)?;
    match head {
        "org-elements-query" | "query" | "and" => compile_index_and_query(&items[1..]),
        "predicate" => compile_index_predicate_query(compile_predicate_expression(items.get(1)?)?),
        "or" => compile_index_predicate_query(OrgElementQueryPredicate::any(
            items[1..]
                .iter()
                .map(compile_predicate_expression)
                .collect::<Option<Vec<_>>>()?,
        )),
        "not" => compile_index_predicate_query(OrgElementQueryPredicate::negate(
            compile_predicate_expression(items.get(1)?)?,
        )),
        "=" => compile_index_comparison_query(items, false),
        "contains" => compile_index_comparison_query(items, true),
        "category" => {
            let mut query = OrgElementsIndexQuery::new();
            query.category = OrgElementsIndexCategory::from_label(&items.get(1)?.as_text()?);
            Some(query)
        }
        "kind" | "type" => {
            let mut query = OrgElementsIndexQuery::new();
            apply_index_query_kind(&items.get(1)?.as_text()?, &mut query);
            Some(query)
        }
        "affiliated-name" | "affiliatedName" | "name" => {
            Some(OrgElementsIndexQuery::new().affiliated_name(items.get(1)?.as_text()?))
        }
        "context" => Some(OrgElementsIndexQuery::new().context(items.get(1)?.as_text()?)),
        "outline-path-prefix" | "outlinePathPrefix" => Some(
            OrgElementsIndexQuery::new()
                .outline_path_prefix(outline_path_expression(items.get(1)?)?),
        ),
        "outline-path-exact-len" | "outlinePathExactLen" | "outline-depth" => Some(
            OrgElementsIndexQuery::new()
                .outline_path_exact_len(items.get(1)?.as_text()?.parse::<usize>().ok()?),
        ),
        "summary" => compile_index_field_shorthand_query(items, FieldKind::Summary, false),
        "summary-contains" => compile_index_field_shorthand_query(items, FieldKind::Summary, true),
        "property" => compile_index_field_shorthand_query(items, FieldKind::Property, false),
        "property-contains" => {
            compile_index_field_shorthand_query(items, FieldKind::Property, true)
        }
        "relation" => compile_index_relation_query(items.get(1)?.as_atom()?, &items[2..]),
        "child-of" => compile_index_relation_query("child-of", &items[1..]),
        "descendant-of" | "within" => compile_index_relation_query("descendant-of", &items[1..]),
        "ancestor-of" => compile_index_relation_query("ancestor-of", &items[1..]),
        "at" => compile_index_relation_query("at", &items[1..]),
        "limit" => {
            let limit = items.get(1)?.as_text()?.parse::<usize>().ok()?;
            Some(OrgElementsIndexQuery::new().limit(limit))
        }
        _ => compile_index_kind_sugar_query(head, &items[1..]),
    }
}

fn compile_index_and_query(expressions: &[QueryExpr]) -> Option<OrgElementsIndexQuery> {
    let mut query = OrgElementsIndexQuery::new();
    for expression in expressions {
        merge_index_query(&mut query, compile_index_query_expression(expression)?);
    }
    Some(query)
}

fn compile_index_predicate_query(
    predicate: OrgElementQueryPredicate,
) -> Option<OrgElementsIndexQuery> {
    Some(OrgElementsIndexQuery::new().predicate(predicate))
}

fn compile_index_comparison_query(
    items: &[QueryExpr],
    contains: bool,
) -> Option<OrgElementsIndexQuery> {
    let field = parse_field_ref(items.get(1)?)?;
    compile_index_field_query(field.kind, field.key, items.get(2)?, contains)
}

fn compile_index_field_shorthand_query(
    items: &[QueryExpr],
    field_kind: FieldKind,
    contains: bool,
) -> Option<OrgElementsIndexQuery> {
    compile_index_field_query(
        field_kind,
        items.get(1)?.as_text()?,
        items.get(2)?,
        contains,
    )
}

fn compile_index_field_query(
    field_kind: FieldKind,
    key: String,
    value: &QueryExpr,
    contains: bool,
) -> Option<OrgElementsIndexQuery> {
    let query = match (field_kind, contains) {
        (FieldKind::Summary, false) => {
            OrgElementsIndexQuery::new().summary_eq(key, expression_summary_value(value)?)
        }
        (FieldKind::Summary, true) => {
            OrgElementsIndexQuery::new().summary_contains(key, value.as_text()?)
        }
        (FieldKind::Property, false) => {
            OrgElementsIndexQuery::new().property_eq(key, expression_summary_value(value)?)
        }
        (FieldKind::Property, true) => {
            OrgElementsIndexQuery::new().property_contains(key, value.as_text()?)
        }
    };
    Some(query)
}

fn compile_index_relation_query(
    relation: &str,
    arguments: &[QueryExpr],
) -> Option<OrgElementsIndexQuery> {
    let ids = id_set(arguments)?;
    let mut query = OrgElementsIndexQuery::new();
    query.relations.push(match relation {
        "child-of" | "childOf" => OrgElementsIndexRelation::ChildOf(ids),
        "descendant-of" | "descendantOf" | "within" => OrgElementsIndexRelation::DescendantOf(ids),
        "ancestor-of" | "ancestorOf" => OrgElementsIndexRelation::AncestorOf(ids),
        "at" => OrgElementsIndexRelation::At(ids),
        _ => return None,
    });
    Some(query)
}

fn compile_index_kind_sugar_query(
    kind: &str,
    arguments: &[QueryExpr],
) -> Option<OrgElementsIndexQuery> {
    let mut query = OrgElementsIndexQuery::new();
    apply_index_query_kind(kind, &mut query);
    let mut index = 0;
    while index < arguments.len() {
        let keyword = arguments.get(index)?.as_atom()?;
        index += 1;
        let value = arguments.get(index)?;
        index += 1;
        apply_index_keyword_argument(&mut query, keyword, value)?;
    }
    Some(query)
}

fn apply_index_keyword_argument(
    query: &mut OrgElementsIndexQuery,
    keyword: &str,
    value: &QueryExpr,
) -> Option<()> {
    match keyword {
        ":category" => {
            query.category = OrgElementsIndexCategory::from_label(&value.as_text()?);
        }
        ":kind" | ":type" => apply_index_query_kind(&value.as_text()?, query),
        ":name" | ":affiliated-name" | ":affiliatedName" => {
            query.affiliated_name = Some(value.as_text()?);
        }
        ":context" => query.context = Some(value.as_text()?),
        ":outline-path-prefix" | ":outlinePathPrefix" => {
            query.outline_path_prefix = outline_path_expression(value)?;
        }
        ":outline-path-exact-len" | ":outlinePathExactLen" | ":outline-depth" => {
            query.outline_path_exact_len = Some(value.as_text()?.parse::<usize>().ok()?);
        }
        ":summary" => apply_index_plist_field_argument(query, FieldKind::Summary, value, false)?,
        ":summary-contains" => {
            apply_index_plist_field_argument(query, FieldKind::Summary, value, true)?
        }
        ":property" => apply_index_plist_field_argument(query, FieldKind::Property, value, false)?,
        ":property-contains" => {
            apply_index_plist_field_argument(query, FieldKind::Property, value, true)?
        }
        ":column" => push_index_summary_eq(query, "columnName", value)?,
        ":text" => push_index_summary_eq(query, "text", value)?,
        ":language" => push_index_summary_eq(query, "language", value)?,
        ":nonempty" => query.summary_equals.push(OrgElementsIndexSummaryPredicate {
            key: "hasText".to_string(),
            value: OrgElementsIndexSummaryValue::Bool(value.as_bool()?),
        }),
        ":header" => query.summary_equals.push(OrgElementsIndexSummaryPredicate {
            key: "isHeader".to_string(),
            value: OrgElementsIndexSummaryValue::Bool(value.as_bool()?),
        }),
        ":child-of" => query
            .relations
            .push(OrgElementsIndexRelation::ChildOf(id_set(
                std::slice::from_ref(value),
            )?)),
        ":descendant-of" | ":within" => {
            query
                .relations
                .push(OrgElementsIndexRelation::DescendantOf(id_set(
                    std::slice::from_ref(value),
                )?))
        }
        ":ancestor-of" => query
            .relations
            .push(OrgElementsIndexRelation::AncestorOf(id_set(
                std::slice::from_ref(value),
            )?)),
        ":at" => query
            .relations
            .push(OrgElementsIndexRelation::At(id_set(std::slice::from_ref(
                value,
            ))?)),
        ":limit" => query.limit = Some(value.as_text()?.parse::<usize>().ok()?),
        _ => return None,
    }
    Some(())
}

fn apply_index_plist_field_argument(
    query: &mut OrgElementsIndexQuery,
    kind: FieldKind,
    value: &QueryExpr,
    contains: bool,
) -> Option<()> {
    let QueryExpr::List(items) = value else {
        return None;
    };
    let key = items.first()?.as_text()?;
    let value = items.get(1)?;
    match (kind, contains) {
        (FieldKind::Summary, false) => {
            query.summary_equals.push(OrgElementsIndexSummaryPredicate {
                key,
                value: expression_summary_value(value)?,
            })
        }
        (FieldKind::Summary, true) => {
            query
                .summary_contains
                .push(OrgElementsIndexSummaryTextPredicate {
                    key,
                    needle: value.as_text()?,
                })
        }
        (FieldKind::Property, false) => {
            query
                .property_equals
                .push(OrgElementsIndexSummaryPredicate {
                    key,
                    value: expression_summary_value(value)?,
                })
        }
        (FieldKind::Property, true) => {
            query
                .property_contains
                .push(OrgElementsIndexSummaryTextPredicate {
                    key,
                    needle: value.as_text()?,
                })
        }
    }
    Some(())
}

fn push_index_summary_eq(
    query: &mut OrgElementsIndexQuery,
    key: &str,
    value: &QueryExpr,
) -> Option<()> {
    query.summary_equals.push(OrgElementsIndexSummaryPredicate {
        key: key.to_string(),
        value: expression_summary_value(value)?,
    });
    Some(())
}

fn apply_index_query_kind(kind: &str, query: &mut OrgElementsIndexQuery) {
    let mut contract_query = OrgContractQuery::default();
    apply_org_elements_query_kind(kind, &mut contract_query);
    query.category = contract_query.category;
    query.kind = contract_query.kind;
}

fn merge_index_query(target: &mut OrgElementsIndexQuery, source: OrgElementsIndexQuery) {
    if source.category.is_some() {
        target.category = source.category;
    }
    if source.kind.is_some() {
        target.kind = source.kind;
    }
    if source.affiliated_name.is_some() {
        target.affiliated_name = source.affiliated_name;
    }
    if source.context.is_some() {
        target.context = source.context;
    }
    if !source.outline_path_prefix.is_empty() {
        target.outline_path_prefix = source.outline_path_prefix;
    }
    if source.outline_path_exact_len.is_some() {
        target.outline_path_exact_len = source.outline_path_exact_len;
    }
    target.property_equals.extend(source.property_equals);
    target.property_contains.extend(source.property_contains);
    target.summary_equals.extend(source.summary_equals);
    target.summary_contains.extend(source.summary_contains);
    target.relations.extend(source.relations);
    if source.predicate != OrgElementQueryPredicate::default() {
        target.predicate = if target.predicate == OrgElementQueryPredicate::default() {
            source.predicate
        } else {
            match std::mem::take(&mut target.predicate) {
                OrgElementQueryPredicate::All(mut predicates) => {
                    predicates.push(source.predicate);
                    OrgElementQueryPredicate::All(predicates)
                }
                previous => OrgElementQueryPredicate::All(vec![previous, source.predicate]),
            }
        };
    }
    if source.limit.is_some() {
        target.limit = source.limit;
    }
}

fn outline_path_expression(expression: &QueryExpr) -> Option<Vec<String>> {
    match expression {
        QueryExpr::String(value) | QueryExpr::Atom(value) => Some(outline_path_value(value)),
        QueryExpr::List(items) => items.iter().map(QueryExpr::as_text).collect(),
    }
}

fn outline_path_value(value: &str) -> Vec<String> {
    value
        .split('/')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

fn id_set(arguments: &[QueryExpr]) -> Option<BTreeSet<OrgElementId>> {
    let mut ids = BTreeSet::new();
    for argument in arguments {
        match argument {
            QueryExpr::List(items) => {
                ids.extend(id_set(items)?);
            }
            QueryExpr::Atom(value) | QueryExpr::String(value) => {
                ids.insert(OrgElementId::new(value.parse::<usize>().ok()?));
            }
        }
    }
    (!ids.is_empty()).then_some(ids)
}
