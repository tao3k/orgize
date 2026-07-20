//! S-expression parser and syntax lowering for Org elements query expressions.

use super::core_types::QueryExpr;
use rowan::{GreenNodeBuilder, Language, NodeOrToken, SyntaxKind, SyntaxNode};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum QueryExpressionLanguage {}

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
pub(super) enum QueryExpressionKind {
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

pub(super) fn parse_query_expression_syntax(value: &str) -> Option<QuerySyntaxNode> {
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

pub(super) fn lower_root(root: &QuerySyntaxNode) -> Option<Vec<QueryExpr>> {
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
