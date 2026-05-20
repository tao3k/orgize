use memchr::memchr2_iter;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while1},
    combinator::opt,
};

use crate::{
    SyntaxKind,
    syntax::{
        combinator::{caret_token, underscore_token},
        parser_contract::ObjectNodesParser,
    },
};

use super::{
    combinator::{GreenElement, l_curly_token, node, r_curly_token},
    input::Input,
};

pub(crate) fn superscript_node(
    input: Input,
    standard_object_nodes: ObjectNodesParser,
) -> IResult<Input, GreenElement, ()> {
    let (input, caret) = caret_token(input)?;

    let mut children = vec![caret];

    if input.c.use_sub_superscript.is_brace() {
        let (input, rest) = template1(input, standard_object_nodes)?;
        children.extend(rest);
        return Ok((input, node(SyntaxKind::SUPERSCRIPT, children)));
    }

    let (input, rest) = alt((
        template0,
        |input| template1(input, standard_object_nodes),
        template2,
    ))
    .parse(input)?;
    children.extend(rest);

    Ok((input, node(SyntaxKind::SUPERSCRIPT, children)))
}

pub(crate) fn subscript_node(
    input: Input,
    standard_object_nodes: ObjectNodesParser,
) -> IResult<Input, GreenElement, ()> {
    let (input, underscore) = underscore_token(input)?;

    let mut children = vec![underscore];

    if input.c.use_sub_superscript.is_brace() {
        let (input, rest) = template1(input, standard_object_nodes)?;
        children.extend(rest);
        return Ok((input, node(SyntaxKind::SUBSCRIPT, children)));
    }

    let (input, rest) = alt((
        template0,
        |input| template1(input, standard_object_nodes),
        template2,
    ))
    .parse(input)?;
    children.extend(rest);

    Ok((input, node(SyntaxKind::SUBSCRIPT, children)))
}

fn template0(input: Input) -> IResult<Input, Vec<GreenElement>, ()> {
    let (input, star) = tag("*").parse(input)?;
    Ok((input, vec![star.text_token()]))
}

fn template1(
    input: Input,
    standard_object_nodes: ObjectNodesParser,
) -> IResult<Input, Vec<GreenElement>, ()> {
    let (input, l) = l_curly_token(input)?;
    let (input, contents) = balanced_brackets(input)?;
    let (input, r) = r_curly_token(input)?;
    let mut children = vec![];
    children.push(l);
    children.extend(standard_object_nodes(contents));
    children.push(r);
    Ok((input, children))
}

fn template2(input: Input) -> IResult<Input, Vec<GreenElement>, ()> {
    let (input, sign) = opt(alt((tag("+"), tag("-")))).parse(input)?;

    let (input, contents) =
        take_while1(|c: char| c.is_alphanumeric() || c == ',' || c == '\\' || c == '.')
            .parse(input)?;

    if contents.s.ends_with(|c: char| !c.is_alphanumeric()) {
        return Err(nom::Err::Error(()));
    }

    let mut children = vec![];

    if let Some(s) = sign {
        children.push(s.text_token())
    }

    children.push(contents.text_token());

    Ok((input, children))
}

fn balanced_brackets(input: Input) -> IResult<Input, Input, ()> {
    let mut pairs = 1;
    let bytes = input.as_bytes();
    for i in memchr2_iter(b'{', b'}', bytes) {
        if bytes[i] == b'{' {
            pairs += 1;
        } else if pairs != 1 {
            pairs -= 1;
        } else {
            return Ok(input.take_split(i));
        }
    }
    Err(nom::Err::Error(()))
}

pub(crate) fn verify_pre(i: &Input) -> bool {
    if i.c.use_sub_superscript.is_nil() {
        return false;
    }
    let s = i.s;
    if s.is_empty() {
        return false;
    }
    let last = s.as_bytes()[s.len() - 1];
    last != b' ' && last != b'\t'
}

#[test]
fn parse() {
    use crate::config::{ParseConfig, UseSubSuperscript};
    use crate::syntax_ast::Subscript;
    use crate::tests::to_ast;

    let to_subscript = to_ast::<Subscript>(|input| {
        subscript_node(input, crate::syntax::object::standard_object_nodes)
    });

    insta::assert_debug_snapshot!(
        to_subscript("_*").syntax,
        @r###"
    SUBSCRIPT@0..2
      UNDERSCORE@0..1 "_"
      TEXT@1..2 "*"
    "###
    );

    insta::assert_debug_snapshot!(
        to_subscript("_{*bo\nld*}").syntax,
        @r###"
    SUBSCRIPT@0..10
      UNDERSCORE@0..1 "_"
      L_CURLY@1..2 "{"
      BOLD@2..9
        STAR@2..3 "*"
        TEXT@3..8 "bo\nld"
        STAR@8..9 "*"
      R_CURLY@9..10 "}"
    "###
    );

    insta::assert_debug_snapshot!(
        to_subscript("_+123").syntax,
        @r###"
    SUBSCRIPT@0..5
      UNDERSCORE@0..1 "_"
      TEXT@1..2 "+"
      TEXT@2..5 "123"
    "###
    );

    insta::assert_debug_snapshot!(
        to_subscript("_abc").syntax,
        @r###"
    SUBSCRIPT@0..4
      UNDERSCORE@0..1 "_"
      TEXT@1..4 "abc"
    "###
    );

    let with_brace = ParseConfig {
        use_sub_superscript: UseSubSuperscript::Brace,
        ..Default::default()
    };

    debug_assert!(
        subscript_node(
            ("_*", &with_brace).into(),
            crate::syntax::object::standard_object_nodes
        )
        .is_err()
    );
    debug_assert!(
        subscript_node(
            ("_abc", &with_brace).into(),
            crate::syntax::object::standard_object_nodes
        )
        .is_err()
    );
    debug_assert!(
        subscript_node(
            ("_+123", &with_brace).into(),
            crate::syntax::object::standard_object_nodes
        )
        .is_err()
    );
    debug_assert!(
        subscript_node(
            ("_{*bo\nld*}", &with_brace).into(),
            crate::syntax::object::standard_object_nodes
        )
        .is_ok()
    );
}
