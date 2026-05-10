#![allow(clippy::type_complexity)]

use nom::{
    branch::alt,
    bytes::complete::{tag, take_till, take_while1},
    character::complete::space0,
    combinator::{recognize, verify},
    IResult, Parser,
};

use super::{
    combinator::{blank_lines, hash_plus_token, node, trim_line_end, GreenElement},
    input::Input,
    SyntaxKind,
};

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
pub(crate) fn keyword_node(input: Input) -> IResult<Input, GreenElement, ()> {
    fn f(input: Input) -> IResult<Input, GreenElement, ()> {
        let (input, (key, mut nodes)) = keyword_node_base(input)?;
        let (input, post_blank) = blank_lines(input)?;
        nodes.extend(post_blank);
        Ok((
            input,
            node(
                if key == "CALL" {
                    SyntaxKind::BABEL_CALL
                } else {
                    SyntaxKind::KEYWORD
                },
                nodes,
            ),
        ))
    }
    crate::lossless_parser!(f, input)
}

/// Return empty vector if input doesn't contain affiliated keyword, or affiliated keyword is
/// followed by blank lines.
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
pub(crate) fn affiliated_keyword_nodes(input: Input) -> IResult<Input, Vec<GreenElement>, ()> {
    if !starts_with_keyword_prefix(input) {
        return Ok((input, Vec::new()));
    }

    if let Some(key) = peek_keyword_key(input) {
        if input.c.affiliated_keywords.iter().all(|w| w != key) && !key.starts_with("ATTR_") {
            return Ok((input, Vec::new()));
        }
    }

    let mut children = vec![];
    let mut i = input;

    while !i.is_empty() {
        let Ok((input_, (key, nodes))) = keyword_node_base(i) else {
            break;
        };

        let (input_, post_blank) = blank_lines(input_)?;

        // affiliated keyword can not followed by blank lines or eof
        if !post_blank.is_empty() || input_.is_empty() {
            return Ok((input, vec![]));
        }

        if input_.c.affiliated_keywords.iter().all(|w| w != key) && !key.starts_with("ATTR_") {
            break;
        }

        debug_assert!(i.len() > input_.len(), "{} > {}", i.len(), input_.len());
        i = input_;
        children.push(node(SyntaxKind::AFFILIATED_KEYWORD, nodes));
    }

    Ok((i, children))
}

pub(crate) fn tblfm_keyword_nodes(input: Input) -> IResult<Input, Vec<GreenElement>, ()> {
    if !starts_with_keyword_prefix(input) {
        return Ok((input, Vec::new()));
    }

    if peek_keyword_key(input).is_some_and(|key| !key.eq_ignore_ascii_case("TBLFM")) {
        return Ok((input, Vec::new()));
    }

    let mut children = vec![];
    let mut i = input;

    while !i.is_empty() {
        let Ok((input, (key, nodes))) = keyword_node_base(i) else {
            break;
        };

        if !key.eq_ignore_ascii_case("TBLFM") {
            break;
        }

        debug_assert!(i.len() > input.len(), "{} > {}", i.len(), input.len());
        i = input;
        children.push(node(SyntaxKind::KEYWORD, nodes));
    }

    Ok((i, children))
}

fn keyword_node_base(input: Input<'_>) -> IResult<Input<'_>, (&str, Vec<GreenElement>), ()> {
    if !starts_with_keyword_prefix(input) {
        return Err(nom::Err::Error(()));
    }

    let (input, (ws, hash_plus)) = (space0, hash_plus_token).parse(input)?;

    let (input, (key, optional, colon)) = alt((key_with_optional, key)).parse(input)?;

    let (input, (value, ws_, nl)) = trim_line_end(input)?;

    let mut children = vec![];
    if !ws.is_empty() {
        children.push(ws.ws_token());
    }
    children.push(hash_plus);
    children.push(key.text_token());
    if let Some((l_bracket, optional, r_bracket)) = optional {
        children.push(l_bracket.token(SyntaxKind::L_BRACKET));
        children.push(optional.text_token());
        children.push(r_bracket.token(SyntaxKind::R_BRACKET));
    }
    children.push(colon.token(SyntaxKind::COLON));
    children.push(value.text_token());
    if !ws_.is_empty() {
        children.push(ws_.ws_token());
    }
    if !nl.is_empty() {
        children.push(nl.nl_token());
    }

    Ok((input, (key.s, children)))
}

#[inline]
fn starts_with_keyword_prefix(input: Input<'_>) -> bool {
    let bytes = input.as_bytes();
    let mut i = 0;

    while matches!(bytes.get(i), Some(b' ' | b'\t')) {
        i += 1;
    }

    bytes.get(i..i + 2) == Some(b"#+")
}

#[inline]
fn peek_keyword_key(input: Input<'_>) -> Option<&str> {
    let bytes = input.as_bytes();
    let mut start = 0;

    while matches!(bytes.get(start), Some(b' ' | b'\t')) {
        start += 1;
    }

    if bytes.get(start..start + 2) != Some(b"#+") {
        return None;
    }

    start += 2;
    let key_start = start;

    while let Some(byte) = bytes.get(start) {
        if byte.is_ascii_whitespace() || matches!(byte, b':' | b'[') {
            break;
        }
        start += 1;
    }

    if start == key_start {
        None
    } else {
        Some(&input.as_str()[key_start..start])
    }
}

fn key(input: Input) -> IResult<Input, (Input, Option<(Input, Input, Input)>, Input), ()> {
    let (input, output) = verify(
        recognize((
            take_till(|c: char| c.is_ascii_whitespace() || c == ':'),
            take_while1(|c: char| c == ':'),
        )),
        |i: &Input| i.len() >= 2,
    )
    .parse(input)?;
    let (colon, key) = output.take_split(output.len() - 1);
    Ok((input, (key, None, colon)))
}

fn key_with_optional(
    input: Input,
) -> IResult<Input, (Input, Option<(Input, Input, Input)>, Input), ()> {
    let (input, (key, r_backer, optional, l_backer, colon)) = (
        alt((tag("CAPTION"), tag("RESULTS"))),
        tag("["),
        take_till(|c| c == '\r' || c == '\n' || c == ']'),
        tag("]"),
        tag(":"),
    )
        .parse(input)?;
    Ok((input, (key, Some((r_backer, optional, l_backer)), colon)))
}

#[test]
fn parse() {
    use crate::{
        syntax_ast::{BabelCall, Keyword},
        tests::to_ast,
        ParseConfig,
    };

    let to_keyword = to_ast::<Keyword>(keyword_node);

    let to_babel_call = to_ast::<BabelCall>(keyword_node);

    to_keyword("#+KEY:");
    to_keyword("#+::");
    to_keyword("#+::");
    to_keyword("#+:: ");
    to_keyword("#+:: \n");
    to_keyword("#+::\n");

    insta::assert_debug_snapshot!(
        to_keyword("#+KEY:").syntax,
        @r###"
    KEYWORD@0..6
      HASH_PLUS@0..2 "#+"
      TEXT@2..5 "KEY"
      COLON@5..6 ":"
      TEXT@6..6 ""
    "###
    );

    insta::assert_debug_snapshot!(
        to_keyword("#+KEY: VALUE").syntax,
        @r###"
    KEYWORD@0..12
      HASH_PLUS@0..2 "#+"
      TEXT@2..5 "KEY"
      COLON@5..6 ":"
      TEXT@6..12 " VALUE"
    "###
    );

    insta::assert_debug_snapshot!(
        to_keyword("#+K_E_Y: VALUE").syntax,
        @r###"
    KEYWORD@0..14
      HASH_PLUS@0..2 "#+"
      TEXT@2..7 "K_E_Y"
      COLON@7..8 ":"
      TEXT@8..14 " VALUE"
    "###
    );

    insta::assert_debug_snapshot!(
        to_keyword("#+KEY:VALUE\n").syntax,
        @r###"
    KEYWORD@0..12
      HASH_PLUS@0..2 "#+"
      TEXT@2..5 "KEY"
      COLON@5..6 ":"
      TEXT@6..11 "VALUE"
      NEW_LINE@11..12 "\n"
    "###
    );

    insta::assert_debug_snapshot!(
        to_keyword("#+RESULTS:").syntax,
        @r###"
    KEYWORD@0..10
      HASH_PLUS@0..2 "#+"
      TEXT@2..9 "RESULTS"
      COLON@9..10 ":"
      TEXT@10..10 ""
    "###
    );

    insta::assert_debug_snapshot!(
        to_keyword("#+ATTR_LATEX: :width 5cm\n").syntax,
        @r###"
    KEYWORD@0..25
      HASH_PLUS@0..2 "#+"
      TEXT@2..12 "ATTR_LATEX"
      COLON@12..13 ":"
      TEXT@13..24 " :width 5cm"
      NEW_LINE@24..25 "\n"
    "###
    );

    insta::assert_debug_snapshot!(
        to_babel_call("#+CALL: double(n=4)").syntax,
        @r###"
    BABEL_CALL@0..19
      HASH_PLUS@0..2 "#+"
      TEXT@2..6 "CALL"
      COLON@6..7 ":"
      TEXT@7..19 " double(n=4)"
    "###
    );

    insta::assert_debug_snapshot!(
        to_keyword("#+ABC[OPTIONAL]: Longer value.").syntax,
        @r###"
    KEYWORD@0..30
      HASH_PLUS@0..2 "#+"
      TEXT@2..15 "ABC[OPTIONAL]"
      COLON@15..16 ":"
      TEXT@16..30 " Longer value."
    "###
    );

    insta::assert_debug_snapshot!(
        to_keyword("#+CAPTION: value").syntax,
        @r###"
    KEYWORD@0..16
      HASH_PLUS@0..2 "#+"
      TEXT@2..9 "CAPTION"
      COLON@9..10 ":"
      TEXT@10..16 " value"
    "###
    );

    insta::assert_debug_snapshot!(
        to_keyword("#+CAPTION[caption optional]: value").syntax,
        @r###"
    KEYWORD@0..34
      HASH_PLUS@0..2 "#+"
      TEXT@2..9 "CAPTION"
      L_BRACKET@9..10 "["
      TEXT@10..26 "caption optional"
      R_BRACKET@26..27 "]"
      COLON@27..28 ":"
      TEXT@28..34 " value"
    "###
    );

    let config = &ParseConfig::default();

    assert!(keyword_node(("#+KE Y: VALUE", config).into()).is_err());
    assert!(keyword_node(("#+ KEY: VALUE", config).into()).is_err());
}

#[test]
fn keyword_prefix_detection() {
    let config = &crate::ParseConfig::default();

    assert!(starts_with_keyword_prefix(("#+KEY: value", config).into()));
    assert!(starts_with_keyword_prefix(
        ("  \t#+KEY: value", config).into()
    ));
    assert_eq!(
        peek_keyword_key(("#+KEY: value", config).into()),
        Some("KEY")
    );
    assert_eq!(
        peek_keyword_key(("#+CAPTION[short]: value", config).into()),
        Some("CAPTION")
    );
    assert_eq!(
        peek_keyword_key(("#+ATTR_HTML: value", config).into()),
        Some("ATTR_HTML")
    );
    assert!(!starts_with_keyword_prefix(
        ("regular paragraph", config).into()
    ));
    assert!(!starts_with_keyword_prefix(
        ("  # not an org keyword", config).into()
    ));
    assert!(!starts_with_keyword_prefix(("", config).into()));
}
