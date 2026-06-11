use nom::{
    IResult, Parser,
    bytes::complete::{tag, take_while},
};

use super::{
    SyntaxKind,
    combinator::{GreenElement, NodeBuilder, blank_lines, eol_or_eof},
    input::Input,
    keyword::affiliated_keyword_nodes,
};

fn diary_sexp_node_base(input: Input) -> IResult<Input, GreenElement, ()> {
    let mut b = NodeBuilder::new();

    let (input, keywords) = affiliated_keyword_nodes(input)?;
    b.children.extend(keywords);

    let (input, (percent, paren, text, eol, post_blank)) = (
        tag("%%"),
        tag("("),
        take_while(|c| c != '\r' && c != '\n'),
        eol_or_eof,
        blank_lines,
    )
        .parse(input)?;

    b.token(SyntaxKind::PERCENT2, percent);
    b.token(SyntaxKind::L_PARENS, paren);
    b.text(text);
    b.nl(eol);
    b.children.extend(post_blank);

    Ok((input, b.finish(SyntaxKind::DIARY_SEXP)))
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
pub(crate) fn diary_sexp_node(input: Input) -> IResult<Input, GreenElement, ()> {
    crate::lossless_parser!(diary_sexp_node_base, input)
}

#[test]
fn parse() {
    use crate::{
        ParseConfig,
        syntax::{SyntaxNode, diary_sexp::diary_sexp_node, input::Input},
    };

    let t = |input: &str| {
        SyntaxNode::new_root(
            diary_sexp_node(Input {
                s: input,
                c: &ParseConfig::default(),
            })
            .unwrap()
            .1
            .into_node()
            .unwrap(),
        )
    };

    insta::assert_debug_snapshot!(
        t("%%(org-anniversary 1956 5 14)\n "),
        @r###"
    DIARY_SEXP@0..31
      PERCENT2@0..2 "%%"
      L_PARENS@2..3 "("
      TEXT@3..29 "org-anniversary 1956  ..."
      NEW_LINE@29..30 "\n"
      BLANK_LINE@30..31 " "
    "###
    );

    let config = &ParseConfig::default();
    assert!(diary_sexp_node((" %%(org-bbdb-anniversaries)", config).into()).is_err());
    assert!(diary_sexp_node(("%% org-bbdb-anniversaries", config).into()).is_err());
}
