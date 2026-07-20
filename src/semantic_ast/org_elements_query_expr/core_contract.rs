//! Contract and query expression compilation.

use super::core_predicate::{compile_predicate_expression, parse_field_ref};
use super::core_types::{
    DocumentBoolPredicateKind, DocumentTextPredicateKind, FieldKind, QueryExpr, RelativeKind,
    list_head,
};
use crate::ast::{
    OrgContractBinding, OrgContractCompareOp, OrgContractExpectation, OrgContractQuery,
    OrgElementQueryPredicate, OrgElementsIndexCategory,
};
use crate::ast::{
    OrgContractDocumentPredicate, OrgContractRelativeScope, OrgElementsIndexSummaryValue,
};

pub(super) fn compile_contract_sequence(
    expressions: &[QueryExpr],
) -> Option<(
    Vec<OrgContractBinding>,
    OrgContractQuery,
    OrgContractExpectation,
)> {
    let mut bindings = Vec::new();
    for expression in expressions {
        match expression {
            QueryExpr::List(items) if list_head(items) == Some("let") => {
                bindings.extend(compile_let_bindings(items)?);
            }
            QueryExpr::List(items) if list_head(items) == Some("assert") => {
                let (mut assertion_bindings, query, expectation) = compile_assertion(items)?;
                bindings.append(&mut assertion_bindings);
                return Some((bindings, query, expectation));
            }
            _ => {}
        }
    }
    None
}

pub(super) fn compile_contract_expression(
    expression: &QueryExpr,
) -> Option<(
    Vec<OrgContractBinding>,
    OrgContractQuery,
    OrgContractExpectation,
)> {
    let QueryExpr::List(items) = expression else {
        return None;
    };
    match list_head(items)? {
        "let" => compile_let_contract(items),
        "assert" => compile_assertion(items),
        _ => None,
    }
}

fn compile_let_contract(
    items: &[QueryExpr],
) -> Option<(
    Vec<OrgContractBinding>,
    OrgContractQuery,
    OrgContractExpectation,
)> {
    let mut bindings = compile_let_bindings(items)?;
    for body in &items[2..] {
        if let Some((mut body_bindings, query, expectation)) = compile_contract_expression(body) {
            bindings.append(&mut body_bindings);
            return Some((bindings, query, expectation));
        }
    }
    None
}

fn compile_let_bindings(items: &[QueryExpr]) -> Option<Vec<OrgContractBinding>> {
    let QueryExpr::List(bindings) = items.get(1)? else {
        return None;
    };
    let mut parsed = Vec::new();
    for binding in bindings {
        let QueryExpr::List(binding_items) = binding else {
            return None;
        };
        let [name, query] = binding_items.as_slice() else {
            return None;
        };
        let name = name.as_atom()?.trim_start_matches('$');
        if name.is_empty() {
            return None;
        }
        parsed.push(OrgContractBinding {
            name: name.to_string(),
            query: compile_query_expression(query)?,
        });
    }
    Some(parsed)
}

fn compile_assertion(
    items: &[QueryExpr],
) -> Option<(
    Vec<OrgContractBinding>,
    OrgContractQuery,
    OrgContractExpectation,
)> {
    let (expectation, query_expression) = match items.get(1)?.as_atom()? {
        "exists" => (OrgContractExpectation::Exists, items.get(2)?),
        "not-exists" => (OrgContractExpectation::NotExists, items.get(2)?),
        "count" => {
            let op = parse_compare_op(items.get(2)?.as_atom()?)?;
            let count = items.get(3)?.as_text()?.parse::<usize>().ok()?;
            (OrgContractExpectation::Count(op, count), items.get(4)?)
        }
        _ => return None,
    };
    Some((
        Vec::new(),
        compile_query_expression(query_expression)?,
        expectation,
    ))
}

fn parse_compare_op(value: &str) -> Option<OrgContractCompareOp> {
    match value {
        "==" | "=" => Some(OrgContractCompareOp::Eq),
        "!=" => Some(OrgContractCompareOp::Ne),
        "<" => Some(OrgContractCompareOp::Lt),
        "<=" => Some(OrgContractCompareOp::Le),
        ">" => Some(OrgContractCompareOp::Gt),
        ">=" => Some(OrgContractCompareOp::Ge),
        _ => None,
    }
}

pub(super) fn compile_query_expression(expression: &QueryExpr) -> Option<OrgContractQuery> {
    let QueryExpr::List(items) = expression else {
        return None;
    };
    let head = list_head(items)?;
    match head {
        "and" => compile_and_query(&items[1..]),
        "or" => Some(OrgContractQuery {
            alternatives: items[1..]
                .iter()
                .map(compile_query_expression)
                .collect::<Option<Vec<_>>>()?,
            ..Default::default()
        }),
        "not" => compile_predicate_query(OrgElementQueryPredicate::negate(
            compile_predicate_expression(items.get(1)?)?,
        )),
        "=" => compile_comparison_query(items, false),
        "contains" => compile_comparison_query(items, true),
        "kind" => {
            let mut query = OrgContractQuery::default();
            apply_org_elements_query_kind(&items.get(1)?.as_text()?, &mut query);
            Some(query)
        }
        "category" => Some(OrgContractQuery {
            category: OrgElementsIndexCategory::from_label(&items.get(1)?.as_text()?),
            ..Default::default()
        }),
        "summary" => compile_field_shorthand_query(items, FieldKind::Summary, false),
        "summary-contains" => compile_field_shorthand_query(items, FieldKind::Summary, true),
        "property" => compile_field_shorthand_query(items, FieldKind::Property, false),
        "property-contains" => compile_field_shorthand_query(items, FieldKind::Property, true),
        "source-path" => compile_document_text_query(items, DocumentTextPredicateKind::PathEquals),
        "source-path-contains" => {
            compile_document_text_query(items, DocumentTextPredicateKind::PathContains)
        }
        "source-filename" => {
            compile_document_text_query(items, DocumentTextPredicateKind::FilenameEquals)
        }
        "source-filename-prefix" => {
            compile_document_text_query(items, DocumentTextPredicateKind::FilenamePrefix)
        }
        "source-filename-suffix" => {
            compile_document_text_query(items, DocumentTextPredicateKind::FilenameSuffix)
        }
        "source-filename-stem-uppercase" => {
            compile_document_bool_query(items, DocumentBoolPredicateKind::FilenameStemUppercase)
        }
        "descendant-of" | "within" => compile_relative_query(items, RelativeKind::Descendant),
        "child-of" => compile_relative_query(items, RelativeKind::Child),
        "at" => compile_relative_query(items, RelativeKind::At),
        "limit" => Some(OrgContractQuery {
            limit: items.get(1)?.as_text()?.parse::<usize>().ok(),
            ..Default::default()
        }),
        _ => compile_kind_sugar_query(head, &items[1..]),
    }
}

fn compile_and_query(expressions: &[QueryExpr]) -> Option<OrgContractQuery> {
    let mut query = OrgContractQuery::default();
    for expression in expressions {
        merge_query(&mut query, compile_query_expression(expression)?);
    }
    Some(query)
}

fn compile_predicate_query(predicate: OrgElementQueryPredicate) -> Option<OrgContractQuery> {
    let mut query = OrgContractQuery::default();
    query.predicates.push(predicate);
    Some(query)
}

fn compile_comparison_query(items: &[QueryExpr], contains: bool) -> Option<OrgContractQuery> {
    let field = parse_field_ref(items.get(1)?)?;
    let value = items.get(2)?;
    let predicate = match (field.kind, contains) {
        (FieldKind::Summary, false) => {
            OrgElementQueryPredicate::summary_eq(field.key, expression_summary_value(value)?)
        }
        (FieldKind::Summary, true) => {
            OrgElementQueryPredicate::summary_contains(field.key, value.as_text()?)
        }
        (FieldKind::Property, false) => {
            OrgElementQueryPredicate::property_eq(field.key, expression_summary_value(value)?)
        }
        (FieldKind::Property, true) => {
            OrgElementQueryPredicate::property_contains(field.key, value.as_text()?)
        }
    };
    compile_predicate_query(predicate)
}

fn compile_field_shorthand_query(
    items: &[QueryExpr],
    field_kind: FieldKind,
    contains: bool,
) -> Option<OrgContractQuery> {
    let key = items.get(1)?.as_text()?;
    let value = items.get(2)?;
    let predicate = match (field_kind, contains) {
        (FieldKind::Summary, false) => {
            OrgElementQueryPredicate::summary_eq(key, expression_summary_value(value)?)
        }
        (FieldKind::Summary, true) => {
            OrgElementQueryPredicate::summary_contains(key, value.as_text()?)
        }
        (FieldKind::Property, false) => {
            OrgElementQueryPredicate::property_eq(key, expression_summary_value(value)?)
        }
        (FieldKind::Property, true) => {
            OrgElementQueryPredicate::property_contains(key, value.as_text()?)
        }
    };
    compile_predicate_query(predicate)
}

fn compile_relative_query(items: &[QueryExpr], kind: RelativeKind) -> Option<OrgContractQuery> {
    let mut query = OrgContractQuery::default();
    let target = items.get(1)?.as_text()?;
    apply_relative_scope(&mut query, kind, &target);
    Some(query)
}

fn compile_kind_sugar_query(kind: &str, arguments: &[QueryExpr]) -> Option<OrgContractQuery> {
    let mut query = OrgContractQuery::default();
    apply_org_elements_query_kind(kind, &mut query);
    let mut index = 0;
    while index < arguments.len() {
        let keyword = arguments.get(index)?.as_atom()?;
        index += 1;
        let value = arguments.get(index)?;
        index += 1;
        apply_keyword_argument(&mut query, keyword, value)?;
    }
    Some(query)
}

fn apply_keyword_argument(
    query: &mut OrgContractQuery,
    keyword: &str,
    value: &QueryExpr,
) -> Option<()> {
    match keyword {
        ":descendant-of" | ":within" => {
            apply_relative_scope(query, RelativeKind::Descendant, &value.as_text()?);
        }
        ":child-of" => {
            apply_relative_scope(query, RelativeKind::Child, &value.as_text()?);
        }
        ":at" => {
            apply_relative_scope(query, RelativeKind::At, &value.as_text()?);
        }
        ":column" => query.predicates.push(OrgElementQueryPredicate::summary_eq(
            "columnName",
            expression_summary_value(value)?,
        )),
        ":text" => query.predicates.push(OrgElementQueryPredicate::summary_eq(
            "text",
            expression_summary_value(value)?,
        )),
        ":nonempty" => query.predicates.push(OrgElementQueryPredicate::summary_eq(
            "hasText",
            OrgElementsIndexSummaryValue::Bool(value.as_bool()?),
        )),
        ":header" => query.predicates.push(OrgElementQueryPredicate::summary_eq(
            "isHeader",
            OrgElementsIndexSummaryValue::Bool(value.as_bool()?),
        )),
        ":language" => query.predicates.push(OrgElementQueryPredicate::summary_eq(
            "language",
            expression_summary_value(value)?,
        )),
        ":summary" => apply_plist_field_argument(query, FieldKind::Summary, value, false)?,
        ":summary-contains" => apply_plist_field_argument(query, FieldKind::Summary, value, true)?,
        ":property" => apply_plist_field_argument(query, FieldKind::Property, value, false)?,
        ":property-contains" => {
            apply_plist_field_argument(query, FieldKind::Property, value, true)?
        }
        ":path" | ":source-path" => {
            query
                .document_predicates
                .push(OrgContractDocumentPredicate::SourcePathEquals(
                    value.as_text()?,
                ))
        }
        ":path-contains" | ":source-path-contains" => {
            query
                .document_predicates
                .push(OrgContractDocumentPredicate::SourcePathContains(
                    value.as_text()?,
                ))
        }
        ":filename" | ":source-filename" => {
            query
                .document_predicates
                .push(OrgContractDocumentPredicate::SourceFilenameEquals(
                    value.as_text()?,
                ))
        }
        ":filename-prefix" | ":source-filename-prefix" => {
            query
                .document_predicates
                .push(OrgContractDocumentPredicate::SourceFilenamePrefix(
                    value.as_text()?,
                ))
        }
        ":filename-suffix" | ":source-filename-suffix" => {
            query
                .document_predicates
                .push(OrgContractDocumentPredicate::SourceFilenameSuffix(
                    value.as_text()?,
                ))
        }
        ":filename-stem-uppercase" | ":source-filename-stem-uppercase" => {
            query.document_predicates.push(
                OrgContractDocumentPredicate::SourceFilenameStemUppercase(value.as_bool()?),
            )
        }
        ":name" | ":affiliated-name" => query.affiliated_name = Some(value.as_text()?),
        ":context" => query.context = Some(value.as_text()?),
        ":limit" => query.limit = value.as_text()?.parse::<usize>().ok(),
        _ => return None,
    }
    Some(())
}

fn compile_document_text_query(
    items: &[QueryExpr],
    kind: DocumentTextPredicateKind,
) -> Option<OrgContractQuery> {
    let value = items.get(1)?.as_text()?;
    let mut query = OrgContractQuery::default();
    query.document_predicates.push(match kind {
        DocumentTextPredicateKind::PathEquals => {
            OrgContractDocumentPredicate::SourcePathEquals(value)
        }
        DocumentTextPredicateKind::PathContains => {
            OrgContractDocumentPredicate::SourcePathContains(value)
        }
        DocumentTextPredicateKind::FilenameEquals => {
            OrgContractDocumentPredicate::SourceFilenameEquals(value)
        }
        DocumentTextPredicateKind::FilenamePrefix => {
            OrgContractDocumentPredicate::SourceFilenamePrefix(value)
        }
        DocumentTextPredicateKind::FilenameSuffix => {
            OrgContractDocumentPredicate::SourceFilenameSuffix(value)
        }
    });
    Some(query)
}

fn compile_document_bool_query(
    items: &[QueryExpr],
    kind: DocumentBoolPredicateKind,
) -> Option<OrgContractQuery> {
    let value = items.get(1)?.as_bool()?;
    let mut query = OrgContractQuery::default();
    query.document_predicates.push(match kind {
        DocumentBoolPredicateKind::FilenameStemUppercase => {
            OrgContractDocumentPredicate::SourceFilenameStemUppercase(value)
        }
    });
    Some(query)
}

fn apply_plist_field_argument(
    query: &mut OrgContractQuery,
    kind: FieldKind,
    value: &QueryExpr,
    contains: bool,
) -> Option<()> {
    let QueryExpr::List(items) = value else {
        return None;
    };
    let key = items.first()?.as_text()?;
    let value = items.get(1)?;
    let predicate = match (kind, contains) {
        (FieldKind::Summary, false) => {
            OrgElementQueryPredicate::summary_eq(key, expression_summary_value(value)?)
        }
        (FieldKind::Summary, true) => {
            OrgElementQueryPredicate::summary_contains(key, value.as_text()?)
        }
        (FieldKind::Property, false) => {
            OrgElementQueryPredicate::property_eq(key, expression_summary_value(value)?)
        }
        (FieldKind::Property, true) => {
            OrgElementQueryPredicate::property_contains(key, value.as_text()?)
        }
    };
    query.predicates.push(predicate);
    Some(())
}

pub(super) fn apply_relative_scope(query: &mut OrgContractQuery, kind: RelativeKind, target: &str) {
    if target == "$scope" {
        query.use_scope_outline_path = true;
        query.scope_outline_depth = match kind {
            RelativeKind::Descendant => None,
            RelativeKind::Child => Some(1),
            RelativeKind::At => Some(0),
        };
        return;
    }

    let binding = target.trim_start_matches('$').to_string();
    query.relative_to = Some(match kind {
        RelativeKind::Descendant => OrgContractRelativeScope::DescendantOfBinding(binding),
        RelativeKind::Child => OrgContractRelativeScope::ChildOfBinding(binding),
        RelativeKind::At => OrgContractRelativeScope::AtBinding(binding),
    });
}

pub(super) fn merge_query(target: &mut OrgContractQuery, source: OrgContractQuery) {
    target.alternatives.extend(source.alternatives);
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
    target.predicates.extend(source.predicates);
    target
        .document_predicates
        .extend(source.document_predicates);
    if source.limit.is_some() {
        target.limit = source.limit;
    }
    target.use_scope_outline_path |= source.use_scope_outline_path;
    target.has_outline_path_prefix |= source.has_outline_path_prefix;
    if source.scope_outline_depth.is_some() {
        target.scope_outline_depth = source.scope_outline_depth;
    }
    if source.relative_to.is_some() {
        target.relative_to = source.relative_to;
    }
}
use super::core::apply_org_elements_query_kind;
use super::core_predicate::expression_summary_value;
