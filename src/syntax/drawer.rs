use nom::{
    bytes::complete::{tag_no_case, take_while1},
    character::complete::{space0, space1},
    combinator::{iterator, map, verify},
    IResult, Parser,
};

use super::{
    combinator::{
        blank_lines, colon_token, eol_or_eof, line_starts_iter, node, trim_line_end, GreenElement,
        NodeBuilder,
    },
    element::element_nodes,
    input::Input,
    SyntaxKind,
};

fn drawer_begin_node(input: Input<'_>) -> IResult<Input<'_>, (GreenElement, &str), ()> {
    let mut b = NodeBuilder::new();

    let (input, (ws, colon, name, colon_, ws_, nl)) = (
        space0,
        colon_token,
        take_while1(|c: char| c.is_ascii_alphabetic() || c == '-' || c == '_'),
        colon_token,
        space0,
        eol_or_eof,
    )
        .parse(input)?;

    b.ws(ws);
    b.push(colon);
    b.text(name);
    b.push(colon_);
    b.ws(ws_);
    b.nl(nl);

    Ok((input, (b.finish(SyntaxKind::DRAWER_BEGIN), name.as_str())))
}

fn drawer_end_node(input: Input) -> IResult<Input, GreenElement, ()> {
    let (input, (ws, colon, end, colon_, ws_, nl)) = (
        space0,
        colon_token,
        tag_no_case("END"),
        colon_token,
        space0,
        eol_or_eof,
    )
        .parse(input)?;

    let mut b = NodeBuilder::new();
    b.ws(ws);
    b.push(colon);
    b.text(end);
    b.push(colon_);
    b.ws(ws_);
    b.nl(nl);

    Ok((input, b.finish(SyntaxKind::DRAWER_END)))
}

fn drawer_node_base(input: Input) -> IResult<Input, GreenElement, ()> {
    let (input, (begin, _)) = drawer_begin_node(input)?;

    let (input, pre_blank) = blank_lines(input)?;

    for (input, contents) in line_starts_iter(input.as_str()).map(|i| input.take_split(i)) {
        if let Ok((input, end)) = drawer_end_node(input) {
            let (input, post_blank) = blank_lines(input)?;
            let mut children = vec![begin];
            children.extend(pre_blank);
            if !contents.is_empty() {
                children.push(node(SyntaxKind::DRAWER_CONTENT, element_nodes(contents)?));
            } else {
                children.push(node(SyntaxKind::DRAWER_CONTENT, []));
            }
            children.push(end);
            children.extend(post_blank);

            return Ok((input, node(SyntaxKind::DRAWER, children)));
        }
    }

    Err(nom::Err::Error(()))
}

fn property_drawer_node_base(input: Input) -> IResult<Input, GreenElement, ()> {
    let (input, (begin, name)) = drawer_begin_node(input)?;

    if !name.eq_ignore_ascii_case("properties") {
        return Err(nom::Err::Error(()));
    }

    let mut children = vec![begin];

    let mut it = iterator(input, node_property_node);
    children.extend(&mut it);
    let (input, _) = it.finish()?;
    let (input, end) = drawer_end_node(input)?;
    let (input, post_blank) = blank_lines(input)?;

    children.push(end);
    children.extend(post_blank);

    Ok((input, node(SyntaxKind::PROPERTY_DRAWER, children)))
}

fn node_property_node(input: Input) -> IResult<Input, GreenElement, ()> {
    let (input, ws1) = space0(input)?;
    let (input, colon1) = colon_token(input)?;
    let (input, (colon2, name)) = map(
        verify(
            take_while1(|c| c != ' ' && c != '\t' && c != '\n' && c != '\r'),
            |i: &Input| i.ends_with(':'),
        ),
        |input: Input| input.take_split(input.len() - 1),
    )
    .parse(input)?;
    let (input, ws2) = space1(input)?;
    let (input, (value, ws3, nl)) = trim_line_end(input)?;

    let mut b = NodeBuilder::new();

    b.ws(ws1);
    b.push(colon1);

    if name.ends_with('+') {
        let (plus, name) = name.take_split(name.len() - 1);
        b.text(name);
        b.token(SyntaxKind::PLUS, plus);
    } else {
        b.text(name);
    }

    b.token(SyntaxKind::COLON, colon2);
    b.ws(ws2);
    b.text(value);
    b.ws(ws3);
    b.nl(nl);

    Ok((input, b.finish(SyntaxKind::NODE_PROPERTY)))
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
pub(crate) fn property_drawer_node(input: Input) -> IResult<Input, GreenElement, ()> {
    debug_assert!(!input.is_empty());
    crate::lossless_parser!(property_drawer_node_base, input)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
pub(crate) fn drawer_node(input: Input) -> IResult<Input, GreenElement, ()> {
    crate::lossless_parser!(drawer_node_base, input)
}

#[test]
fn parse() {
    use crate::{
        syntax_ast::{Drawer, PropertyDrawer},
        tests::to_ast,
        ParseConfig,
    };

    let to_drawer = to_ast::<Drawer>(drawer_node);
    let to_property_drawer = to_ast::<PropertyDrawer>(property_drawer_node);

    insta::assert_debug_snapshot!(
        to_drawer(
            r#":DRAWER:
  :CUSTOM_ID: id
  :END:"#
        ).syntax,
       @r###"
    DRAWER@0..33
      DRAWER_BEGIN@0..9
        COLON@0..1 ":"
        TEXT@1..7 "DRAWER"
        COLON@7..8 ":"
        NEW_LINE@8..9 "\n"
      DRAWER_CONTENT@9..26
        PARAGRAPH@9..26
          TEXT@9..18 "  :CUSTOM"
          SUBSCRIPT@18..21
            UNDERSCORE@18..19 "_"
            TEXT@19..21 "ID"
          TEXT@21..26 ": id\n"
      DRAWER_END@26..33
        WHITESPACE@26..28 "  "
        COLON@28..29 ":"
        TEXT@29..32 "END"
        COLON@32..33 ":"
    "###
    );

    insta::assert_debug_snapshot!(
        to_drawer(
            r#":DRAWER:

  :END:

"#
        ).syntax,
        @r###"
    DRAWER@0..19
      DRAWER_BEGIN@0..9
        COLON@0..1 ":"
        TEXT@1..7 "DRAWER"
        COLON@7..8 ":"
        NEW_LINE@8..9 "\n"
      BLANK_LINE@9..10 "\n"
      DRAWER_CONTENT@10..10
      DRAWER_END@10..18
        WHITESPACE@10..12 "  "
        COLON@12..13 ":"
        TEXT@13..16 "END"
        COLON@16..17 ":"
        NEW_LINE@17..18 "\n"
      BLANK_LINE@18..19 "\n"
    "###
    );

    // https://github.com/PoiScript/orgize/issues/70#issuecomment-2099671563
    insta::assert_debug_snapshot!(
        to_property_drawer(r#":PROPERTIES:
:header-args:clojure:    :session *clojure-1*
:NAME: VALUE
:NAME+: VALUE
:END:"#).syntax,
        @r###"
    PROPERTY_DRAWER@0..91
      DRAWER_BEGIN@0..13
        COLON@0..1 ":"
        TEXT@1..11 "PROPERTIES"
        COLON@11..12 ":"
        NEW_LINE@12..13 "\n"
      NODE_PROPERTY@13..59
        COLON@13..14 ":"
        TEXT@14..33 "header-args:clojure"
        COLON@33..34 ":"
        WHITESPACE@34..38 "    "
        TEXT@38..58 ":session *clojure-1*"
        NEW_LINE@58..59 "\n"
      NODE_PROPERTY@59..72
        COLON@59..60 ":"
        TEXT@60..64 "NAME"
        COLON@64..65 ":"
        WHITESPACE@65..66 " "
        TEXT@66..71 "VALUE"
        NEW_LINE@71..72 "\n"
      NODE_PROPERTY@72..86
        COLON@72..73 ":"
        TEXT@73..77 "NAME"
        PLUS@77..78 "+"
        COLON@78..79 ":"
        WHITESPACE@79..80 " "
        TEXT@80..85 "VALUE"
        NEW_LINE@85..86 "\n"
      DRAWER_END@86..91
        COLON@86..87 ":"
        TEXT@87..90 "END"
        COLON@90..91 ":"
    "###
    );

    let config = &ParseConfig::default();

    // https://github.com/PoiScript/orgize/issues/9
    assert!(drawer_node((":SPAGHETTI:\n", config).into()).is_err());

    assert!(property_drawer_node((":PROPERTIES:\n:NAME:VALUE\n:END:", config).into()).is_err());
}
