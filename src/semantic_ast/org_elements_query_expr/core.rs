//! Parser and contract lowering for Elisp-style Org elements query expressions.
//!
//! The same parsed expression tree can lower either into `OrgContractQuery` for
//! contract assertions or into `OrgElementsIndexQuery` for reusable index/search
//! calls.

use std::{error::Error, fmt};

use rowan::{GreenNodeBuilder, Language, NodeOrToken, SyntaxKind, SyntaxNode};

use crate::ast::{
    OrgContractBinding, OrgContractCompareOp, OrgContractExpectation, OrgContractQuery,
    OrgContractRelativeScope, OrgElementQueryPredicate, OrgElementsIndexCategory,
    OrgElementsIndexKind, OrgElementsIndexQuery, OrgElementsIndexSummaryValue,
};

/// Error returned when an Org elements query expression cannot be lowered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementsQueryExpressionError {
    message: String,
}

impl OrgElementsQueryExpressionError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for OrgElementsQueryExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for OrgElementsQueryExpressionError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum QueryExpr {
    Atom(String),
    String(String),
    List(Vec<QueryExpr>),
}

impl QueryExpr {
    pub(super) fn as_atom(&self) -> Option<&str> {
        match self {
            Self::Atom(value) => Some(value),
            Self::String(_) | Self::List(_) => None,
        }
    }

    pub(super) fn as_text(&self) -> Option<String> {
        match self {
            Self::Atom(value) | Self::String(value) => Some(value.clone()),
            Self::List(_) => None,
        }
    }

    pub(super) fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Atom(value) => match value.as_str() {
                "t" | "true" => Some(true),
                "nil" | "false" => Some(false),
                _ => None,
            },
            Self::String(_) | Self::List(_) => None,
        }
    }
}

/// Parses an elisp-style Org elements query expression into the shared index
/// query model.
pub fn org_elements_index_query_from_expr_str(
    value: &str,
) -> Result<OrgElementsIndexQuery, OrgElementsQueryExpressionError> {
    let expressions = parse_expressions(value).ok_or_else(|| {
        OrgElementsQueryExpressionError::new("invalid Org elements query expression syntax")
    })?;
    super::index::compile_index_query_expressions(&expressions).ok_or_else(|| {
        OrgElementsQueryExpressionError::new("unsupported Org elements query expression")
    })
}

/// Parses one expression block as a query-only Org elements IR.
pub(in crate::ast) fn parse_org_elements_query_expression_block(
    value: &str,
) -> Option<OrgContractQuery> {
    let expressions = parse_expressions(value)?;
    match expressions.as_slice() {
        [expression] => compile_query_expression(expression),
        [] => None,
        expressions => {
            let mut query = OrgContractQuery::default();
            for expression in expressions {
                merge_query(&mut query, compile_query_expression(expression)?);
            }
            Some(query)
        }
    }
}

/// Parses one expression block as a contract assertion.
pub(in crate::ast) fn parse_org_contract_expression_block(
    value: &str,
) -> Option<(
    Vec<OrgContractBinding>,
    OrgContractQuery,
    OrgContractExpectation,
)> {
    let expressions = parse_expressions(value)?;
    match expressions.as_slice() {
        [expression] => compile_contract_expression(expression),
        _ => compile_contract_sequence(&expressions),
    }
}

pub(in crate::ast) fn apply_org_elements_query_kind(kind: &str, query: &mut OrgContractQuery) {
    let kind = kind.trim().trim_matches('"');
    match kind {
        "org-data" => {
            query.category = Some(OrgElementsIndexCategory::Document);
            query.kind = Some(OrgElementsIndexKind::new("org-data"));
        }
        "headline" => {
            query.category = Some(OrgElementsIndexCategory::Section);
            query.kind = Some(OrgElementsIndexKind::new("headline"));
        }
        "node-property" => {
            query.category = Some(OrgElementsIndexCategory::Property);
            query.kind = Some(OrgElementsIndexKind::new("node-property"));
        }
        "keyword" => {
            query.category = Some(OrgElementsIndexCategory::Keyword);
            query.kind = Some(OrgElementsIndexKind::new("keyword"));
        }
        "link" | "timestamp" | "bold" | "italic" | "underline" | "strike-through"
        | "superscript" | "subscript" | "code" | "verbatim" | "target" | "radio-target"
        | "footnote-reference" | "citation" | "inline-src-block" | "inline-babel-call"
        | "macro" | "plain-text" | "table-cell" => {
            query.category = Some(OrgElementsIndexCategory::Object);
            query.kind = Some(OrgElementsIndexKind::new(kind));
        }
        _ => {
            query.category = Some(OrgElementsIndexCategory::Element);
            query.kind = Some(OrgElementsIndexKind::new(kind));
        }
    }
}

pub(in crate::ast) fn org_elements_query_summary_value(
    value: &str,
) -> OrgElementsIndexSummaryValue {
    match value {
        "t" | "true" => OrgElementsIndexSummaryValue::Bool(true),
        "nil" | "false" => OrgElementsIndexSummaryValue::Bool(false),
        "null" => OrgElementsIndexSummaryValue::Null,
        _ => value
            .parse::<i64>()
            .map(OrgElementsIndexSummaryValue::Integer)
            .unwrap_or_else(|_| OrgElementsIndexSummaryValue::Text(value.to_string())),
    }
}

fn parse_expressions(value: &str) -> Option<Vec<QueryExpr>> {
    let syntax = parse_query_expression_syntax(value)?;
    lower_root(&syntax)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum QueryExpressionLanguage {}

impl Language for QueryExpressionLanguage {
    type Kind = QueryExpressionKind;

    fn kind_from_raw(raw: SyntaxKind) -> Self::Kind {
        QueryExpressionKind::from_raw(raw)
    }

    fn kind_to_raw(kind: Self::Kind) -> SyntaxKind {
        kind.to_raw()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
enum QueryExpressionKind {
    Root,
    List,
    Atom,
    String,
    Whitespace,
    Comment,
    OpenParen,
    CloseParen,
    Error,
}

impl QueryExpressionKind {
    fn from_raw(raw: SyntaxKind) -> Self {
        match raw.0 {
            0 => Self::Root,
            1 => Self::List,
            2 => Self::Atom,
            3 => Self::String,
            4 => Self::Whitespace,
            5 => Self::Comment,
            6 => Self::OpenParen,
            7 => Self::CloseParen,
            _ => Self::Error,
        }
    }

    const fn to_raw(self) -> SyntaxKind {
        SyntaxKind(self as u16)
    }
}

type QuerySyntaxNode = SyntaxNode<QueryExpressionLanguage>;

fn parse_query_expression_syntax(value: &str) -> Option<QuerySyntaxNode> {
    let parser = QueryExpressionParser::new(value);
    let (root, ok) = parser.parse();
    (ok && root.to_string() == value).then_some(root)
}

struct QueryExpressionParser<'a> {
    input: &'a str,
    position: usize,
    builder: GreenNodeBuilder<'static>,
    ok: bool,
}

impl<'a> QueryExpressionParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            builder: GreenNodeBuilder::new(),
            ok: true,
        }
    }

    fn parse(mut self) -> (QuerySyntaxNode, bool) {
        self.start_node(QueryExpressionKind::Root);
        while !self.is_eof() {
            self.parse_trivia();
            if !self.is_eof() {
                self.parse_expression();
            }
        }
        self.finish_node();
        let ok = self.ok;
        (SyntaxNode::new_root(self.builder.finish()), ok)
    }

    fn parse_expression(&mut self) {
        match self.current_char() {
            Some('(') => self.parse_list(),
            Some(')') => {
                self.ok = false;
                self.token(QueryExpressionKind::Error, self.position + 1);
            }
            Some('"') => self.parse_string(),
            Some(_) => self.parse_atom(),
            None => {}
        }
    }

    fn parse_list(&mut self) {
        self.start_node(QueryExpressionKind::List);
        self.token(QueryExpressionKind::OpenParen, self.position + 1);
        loop {
            self.parse_trivia();
            match self.current_char() {
                Some(')') => {
                    self.token(QueryExpressionKind::CloseParen, self.position + 1);
                    self.finish_node();
                    return;
                }
                Some(_) => self.parse_expression(),
                None => {
                    self.ok = false;
                    self.finish_node();
                    return;
                }
            }
        }
    }

    fn parse_trivia(&mut self) {
        loop {
            match self.current_char() {
                Some(ch) if ch.is_whitespace() => self.parse_whitespace(),
                Some(';') => self.parse_comment(),
                _ => return,
            }
        }
    }

    fn parse_whitespace(&mut self) {
        let end = self.take_while(|ch| ch.is_whitespace());
        self.token(QueryExpressionKind::Whitespace, end);
    }

    fn parse_comment(&mut self) {
        let end = self.take_while(|ch| ch != '\n');
        self.token(QueryExpressionKind::Comment, end);
    }

    fn parse_atom(&mut self) {
        let end = self.take_while(|ch| {
            !ch.is_whitespace() && ch != '(' && ch != ')' && ch != ';' && ch != '"'
        });
        if end == self.position {
            self.ok = false;
            self.token(QueryExpressionKind::Error, self.position + 1);
        } else {
            self.token(QueryExpressionKind::Atom, end);
        }
    }

    fn parse_string(&mut self) {
        let start = self.position;
        self.position += 1;
        let mut escaped = false;
        while let Some(ch) = self.current_char() {
            self.position += ch.len_utf8();
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                self.emit(QueryExpressionKind::String, start, self.position);
                return;
            }
        }
        self.ok = false;
        self.emit(QueryExpressionKind::Error, start, self.position);
    }

    fn take_while(&self, predicate: impl Fn(char) -> bool) -> usize {
        self.input[self.position..]
            .char_indices()
            .find_map(|(offset, ch)| (!predicate(ch)).then_some(self.position + offset))
            .unwrap_or(self.input.len())
    }

    fn current_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }

    fn start_node(&mut self, kind: QueryExpressionKind) {
        self.builder.start_node(kind.to_raw());
    }

    fn finish_node(&mut self) {
        self.builder.finish_node();
    }

    fn token(&mut self, kind: QueryExpressionKind, end: usize) {
        self.emit(kind, self.position, end);
        self.position = end;
    }

    fn emit(&mut self, kind: QueryExpressionKind, start: usize, end: usize) {
        self.builder.token(kind.to_raw(), &self.input[start..end]);
    }
}

fn lower_root(root: &QuerySyntaxNode) -> Option<Vec<QueryExpr>> {
    lower_children(root)
}

fn lower_list(node: &QuerySyntaxNode) -> Option<QueryExpr> {
    lower_children(node).map(QueryExpr::List)
}

fn lower_children(node: &QuerySyntaxNode) -> Option<Vec<QueryExpr>> {
    node.children_with_tokens()
        .try_fold(Vec::new(), |mut expressions, child| {
            if let Some(expression) = lower_child(child)? {
                expressions.push(expression);
            }
            Some(expressions)
        })
}

fn lower_child(
    child: NodeOrToken<QuerySyntaxNode, rowan::SyntaxToken<QueryExpressionLanguage>>,
) -> Option<Option<QueryExpr>> {
    match child {
        NodeOrToken::Node(node) => Some(Some(lower_list(&node)?)),
        NodeOrToken::Token(token) => match token.kind() {
            QueryExpressionKind::Atom => Some(Some(QueryExpr::Atom(token.text().to_string()))),
            QueryExpressionKind::String => {
                Some(Some(QueryExpr::String(unquote_query_string(token.text())?)))
            }
            QueryExpressionKind::Whitespace
            | QueryExpressionKind::Comment
            | QueryExpressionKind::OpenParen
            | QueryExpressionKind::CloseParen => Some(None),
            QueryExpressionKind::Root | QueryExpressionKind::List | QueryExpressionKind::Error => {
                None
            }
        },
    }
}

fn unquote_query_string(raw: &str) -> Option<String> {
    let body = raw.strip_prefix('"')?.strip_suffix('"')?;
    let mut value = String::new();
    let mut chars = body.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            let escaped = chars.next()?;
            value.push(match escaped {
                'n' => '\n',
                't' => '\t',
                '"' => '"',
                '\\' => '\\',
                other => other,
            });
        } else {
            value.push(ch);
        }
    }
    Some(value)
}

fn compile_contract_sequence(
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

fn compile_contract_expression(
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

fn compile_query_expression(expression: &QueryExpr) -> Option<OrgContractQuery> {
    let QueryExpr::List(items) = expression else {
        return None;
    };
    let head = list_head(items)?;
    match head {
        "and" => compile_and_query(&items[1..]),
        "or" => compile_predicate_query(OrgElementQueryPredicate::any(
            items[1..]
                .iter()
                .map(compile_predicate_expression)
                .collect::<Option<Vec<_>>>()?,
        )),
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
        ":name" | ":affiliated-name" => query.affiliated_name = Some(value.as_text()?),
        ":context" => query.context = Some(value.as_text()?),
        ":limit" => query.limit = value.as_text()?.parse::<usize>().ok(),
        _ => return None,
    }
    Some(())
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

fn apply_relative_scope(query: &mut OrgContractQuery, kind: RelativeKind, target: &str) {
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

fn merge_query(target: &mut OrgContractQuery, source: OrgContractQuery) {
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

pub(super) fn list_head(items: &[QueryExpr]) -> Option<&str> {
    items.first()?.as_atom()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FieldKind {
    Summary,
    Property,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct FieldRef {
    pub(super) kind: FieldKind,
    pub(super) key: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RelativeKind {
    Descendant,
    Child,
    At,
}
