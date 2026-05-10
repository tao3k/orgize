use nom::{
    bytes::complete::{take_while, take_while1},
    combinator::{map, opt, verify},
    IResult,
};

use super::{
    combinator::{
        l_angle_token, l_bracket2_token, l_bracket_token, node, r_angle_token, r_bracket2_token,
        r_bracket_token, GreenElement,
    },
    input::Input,
    object::link_description_object_nodes,
    SyntaxKind,
};

fn is_angle_link_path(path: &str) -> bool {
    let Some((scheme, rest)) = path.split_once(':') else {
        return false;
    };
    if scheme.is_empty() || rest.is_empty() {
        return false;
    }

    let mut chars = scheme.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.' | '_'))
}

fn starts_with_angle_link_scheme(input: &str) -> bool {
    let Some(rest) = input.strip_prefix('<') else {
        return false;
    };
    let mut chars = rest.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }

    for c in chars {
        match c {
            ':' => return true,
            '<' | '>' | '\n' => return false,
            c if c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.' | '_') => {}
            _ => return false,
        }
    }

    false
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
pub(crate) fn link_node(input: Input) -> IResult<Input, GreenElement, ()> {
    let mut parser = map(
        (
            l_bracket2_token,
            take_while(|c: char| c != '<' && c != '>' && c != '\n' && c != ']'),
            opt((
                r_bracket_token,
                l_bracket_token,
                take_while(|c: char| c != '[' && c != ']'),
            )),
            r_bracket2_token,
        ),
        |(l_bracket2, path, desc, r_bracket2)| {
            let mut children = vec![l_bracket2, path.token(SyntaxKind::LINK_PATH)];

            if let Some((r_bracket, l_bracket, desc)) = desc {
                children.extend([r_bracket, l_bracket]);
                children.extend(link_description_object_nodes(desc));
            }

            children.push(r_bracket2);

            node(SyntaxKind::LINK, children)
        },
    );
    crate::lossless_parser!(parser, input)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
pub(crate) fn angle_link_node(input: Input) -> IResult<Input, GreenElement, ()> {
    if !starts_with_angle_link_scheme(input.as_str()) {
        return Err(nom::Err::Error(()));
    }

    let mut parser = map(
        (
            l_angle_token,
            verify(
                take_while1(|c: char| c != '<' && c != '>' && c != '\n'),
                |path: &Input<'_>| is_angle_link_path(path.as_str()),
            ),
            r_angle_token,
        ),
        |(l_angle, path, r_angle)| {
            node(
                SyntaxKind::LINK,
                [l_angle, path.token(SyntaxKind::LINK_PATH), r_angle],
            )
        },
    );
    crate::lossless_parser!(parser, input)
}

#[test]
fn parse() {
    use crate::{syntax_ast::Link, tests::to_ast, ParseConfig};

    let to_link = to_ast::<Link>(link_node);

    let link = to_link("[[#id]]");
    insta::assert_debug_snapshot!(
        link.syntax,
        @r###"
    LINK@0..7
      L_BRACKET2@0..2 "[["
      LINK_PATH@2..5 "#id"
      R_BRACKET2@5..7 "]]"
    "###
    );

    let link = to_link("[[#id][desc]]");
    insta::assert_debug_snapshot!(
        link.syntax,
        @r###"
    LINK@0..13
      L_BRACKET2@0..2 "[["
      LINK_PATH@2..5 "#id"
      R_BRACKET@5..6 "]"
      L_BRACKET@6..7 "["
      TEXT@7..11 "desc"
      R_BRACKET2@11..13 "]]"
    "###
    );

    let link = to_link("[[file:/home/dominik/images/jupiter.jpg]]");
    insta::assert_debug_snapshot!(
        link.syntax,
        @r###"
    LINK@0..41
      L_BRACKET2@0..2 "[["
      LINK_PATH@2..39 "file:/home/dominik/im ..."
      R_BRACKET2@39..41 "]]"
    "###
    );

    let link = to_link("[[https://orgmode.org][*bold* description]]");
    insta::assert_debug_snapshot!(
        link.syntax,
        @r###"
    LINK@0..43
      L_BRACKET2@0..2 "[["
      LINK_PATH@2..21 "https://orgmode.org"
      R_BRACKET@21..22 "]"
      L_BRACKET@22..23 "["
      BOLD@23..29
        STAR@23..24 "*"
        TEXT@24..28 "bold"
        STAR@28..29 "*"
      TEXT@29..41 " description"
      R_BRACKET2@41..43 "]]"
    "###
    );

    let config = &ParseConfig::default();

    assert!(link_node(("[[#id][desc]", config).into()).is_err());
}

#[test]
fn parse_angle_link() {
    use crate::{syntax_ast::Link, tests::to_ast, ParseConfig};

    let to_link = to_ast::<Link>(angle_link_node);

    let link = to_link("<https://orgmode.org/manual>");
    insta::assert_debug_snapshot!(
        link.syntax,
        @r###"
    LINK@0..28
      L_ANGLE@0..1 "<"
      LINK_PATH@1..27 "https://orgmode.org/m ..."
      R_ANGLE@27..28 ">"
    "###
    );
    assert_eq!(link.path(), "https://orgmode.org/manual");
    assert!(!link.has_description());

    let config = &ParseConfig::default();
    assert!(angle_link_node(("<2026-04-30 Thu 10:00>", config).into()).is_err());
    assert!(angle_link_node(("<not-a-link>", config).into()).is_err());
    assert!(angle_link_node(("<https:>", config).into()).is_err());
}
