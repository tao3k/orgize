use nom::{
    IResult, Parser,
    bytes::complete::{tag, take_while1},
    combinator::opt,
};

use super::{
    SyntaxKind,
    combinator::{GreenElement, balanced_delimited_tokens, node},
    input::Input,
};

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
pub(crate) fn inline_src_node(input: Input) -> IResult<Input, GreenElement, ()> {
    let mut parser = |input| {
        let (input, src) = tag("src_").parse(input)?;
        let (input, lang) =
            take_while1(|c: char| !c.is_ascii_whitespace() && c != '[' && c != '{').parse(input)?;
        let (input, options) = opt(|input| {
            balanced_delimited_tokens(
                input,
                '[',
                ']',
                SyntaxKind::L_BRACKET,
                SyntaxKind::R_BRACKET,
            )
        })
        .parse(input)?;
        let (input, (l_curly, body, r_curly)) =
            balanced_delimited_tokens(input, '{', '}', SyntaxKind::L_CURLY, SyntaxKind::R_CURLY)?;

        let mut children = vec![src.text_token(), lang.text_token()];
        if let Some((l_bracket, options, r_bracket)) = options {
            children.push(l_bracket);
            children.push(options.text_token());
            children.push(r_bracket);
        }
        children.push(l_curly);
        children.push(body.text_token());
        children.push(r_curly);
        Ok((input, node(SyntaxKind::INLINE_SRC, children)))
    };
    crate::lossless_parser!(parser, input)
}

#[test]
fn parse() {
    use crate::{ParseConfig, syntax_ast::InlineSrc, tests::to_ast};

    let to_inline_src = to_ast::<InlineSrc>(inline_src_node);

    insta::assert_debug_snapshot!(
        to_inline_src("src_C{int a = 0;}").syntax,
        @r###"
    INLINE_SRC@0..17
      TEXT@0..4 "src_"
      TEXT@4..5 "C"
      L_CURLY@5..6 "{"
      TEXT@6..16 "int a = 0;"
      R_CURLY@16..17 "}"
    "###
    );

    insta::assert_debug_snapshot!(
        to_inline_src("src_xml[:exports code]{<tag>text</tag>}").syntax,
        @r###"
    INLINE_SRC@0..39
      TEXT@0..4 "src_"
      TEXT@4..7 "xml"
      L_BRACKET@7..8 "["
      TEXT@8..21 ":exports code"
      R_BRACKET@21..22 "]"
      L_CURLY@22..23 "{"
      TEXT@23..38 "<tag>text</tag>"
      R_CURLY@38..39 "}"
    "###
    );

    let nested = to_inline_src("src_json[:var data=[1,[2]]]{map.get({nested})}");
    assert_eq!(
        nested.syntax.to_string(),
        "src_json[:var data=[1,[2]]]{map.get({nested})}"
    );

    let config = &ParseConfig::default();

    assert!(inline_src_node(("src_xml[:exports code]{<tag>text</tag>", config).into()).is_err());
    assert!(inline_src_node(("src_[:exports code]{<tag>text</tag>}", config).into()).is_err());
    assert!(inline_src_node(("src_xml[:exports code]", config).into()).is_err());
}
