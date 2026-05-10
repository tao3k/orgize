use nom::{Err, IResult};

use super::{
    combinator::{l_bracket_token, node, r_bracket_token, GreenElement},
    input::Input,
    SyntaxKind,
};

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
pub(crate) fn citation_node(input: Input) -> IResult<Input, GreenElement, ()> {
    crate::lossless_parser!(citation_node_base, input)
}

fn citation_node_base(input: Input) -> IResult<Input, GreenElement, ()> {
    let (input, l_bracket) = l_bracket_token(input)?;
    let Some(body_end) = input.as_str().find(']') else {
        return Err(Err::Error(()));
    };
    let body = input.slice(..body_end);

    if !is_valid_citation_body(body.as_str()) {
        return Err(Err::Error(()));
    }

    let (input, body) = input.take_split(body_end);
    let (input, r_bracket) = r_bracket_token(input)?;

    Ok((
        input,
        node(
            SyntaxKind::CITATION,
            [l_bracket, body.text_token(), r_bracket],
        ),
    ))
}

fn is_valid_citation_body(body: &str) -> bool {
    if body.contains(['\n', '\r']) {
        return false;
    }

    let Some((head, references)) = body.split_once(':') else {
        return false;
    };

    is_valid_citation_head(head)
        && references
            .split(';')
            .any(|reference| citation_key_range(reference).is_some())
}

fn is_valid_citation_head(head: &str) -> bool {
    if head == "cite" {
        return true;
    }

    let Some(rest) = head.strip_prefix("cite/") else {
        return false;
    };

    !rest.is_empty()
        && rest
            .split('/')
            .all(|part| !part.is_empty() && !part.chars().any(char::is_whitespace))
}

fn citation_key_range(reference: &str) -> Option<(usize, usize)> {
    let at = reference.find('@')?;
    let start = at + 1;
    let end = reference[start..]
        .find(char::is_whitespace)
        .map(|offset| start + offset)
        .unwrap_or(reference.len());

    (start < end).then_some((start, end))
}

#[test]
fn parse() {
    use crate::{syntax_ast::Citation, tests::to_ast, ParseConfig};

    let to_citation = to_ast::<Citation>(citation_node);

    insta::assert_debug_snapshot!(
        to_citation("[cite:@doe2020]").syntax,
        @r###"
    CITATION@0..15
      L_BRACKET@0..1 "["
      TEXT@1..14 "cite:@doe2020"
      R_BRACKET@14..15 "]"
    "###
    );

    insta::assert_debug_snapshot!(
        to_citation("[cite/text:see @doe2020 p. 42; cf. @roe2021]").syntax,
        @r###"
    CITATION@0..44
      L_BRACKET@0..1 "["
      TEXT@1..43 "cite/text:see @doe202 ..."
      R_BRACKET@43..44 "]"
    "###
    );

    let config = &ParseConfig::default();

    assert!(citation_node(("[cite:no key]", config).into()).is_err());
    assert!(citation_node(("[citation:@key]", config).into()).is_err());
    assert!(citation_node(("[cite @key]", config).into()).is_err());
    assert!(citation_node(("[cite/:@key]", config).into()).is_err());
}
