use nom::{IResult, Parser, bytes::complete::take_until, combinator::opt};

use crate::syntax::{
    combinator::{at_token, l_curly_token, l_curly2_token, r_curly_token},
    parser_contract::ObjectNodesParser,
};

use super::{
    SyntaxKind,
    combinator::{GreenElement, NodeBuilder},
    input::Input,
};

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
pub(crate) fn cloze_node(
    input: Input,
    standard_object_nodes: ObjectNodesParser,
) -> IResult<Input, GreenElement, ()> {
    crate::lossless_parser!(|input| cloze_node_base(input, standard_object_nodes), input)
}

fn cloze_node_base(
    input: Input,
    standard_object_nodes: ObjectNodesParser,
) -> IResult<Input, GreenElement, ()> {
    let (input, l_curly2) = l_curly2_token(input)?;

    let mut inside_latex = false;
    let mut text_end = 0;
    for (index, byte) in input.bytes().enumerate() {
        match byte {
            b'}' if !inside_latex => {
                text_end = index;
                break;
            }
            b'$' => {
                inside_latex = !inside_latex;
            }
            _ => {}
        }
    }

    if text_end == 0 {
        return Err(nom::Err::Error(()));
    }

    let (input, text) = input.take_split(text_end);

    let (input, r_curly) = r_curly_token(input)?;

    let (input, hint) = opt((l_curly_token, take_until("}"), r_curly_token)).parse(input)?;

    let (input, id) = opt((at_token, take_until("}"))).parse(input)?;

    let (input, r_curly_) = r_curly_token(input)?;

    let mut b = NodeBuilder::new();

    b.push(l_curly2);
    b.children.extend(standard_object_nodes(text));
    b.push(r_curly);

    if let Some((l_curly, hint, r_curly)) = hint {
        b.push(l_curly);
        b.token(SyntaxKind::TEXT, hint);
        b.push(r_curly);
    }

    if let Some((at, id)) = id {
        b.push(at);
        b.token(SyntaxKind::TEXT, id);
    }

    b.push(r_curly_);

    Ok((input, b.finish(SyntaxKind::CLOZE)))
}

#[test]
fn parse() {
    use crate::config::ParseConfig;
    use crate::syntax_ast::Cloze;
    use crate::tests::to_ast;

    let to_cloze =
        to_ast::<Cloze>(|input| cloze_node(input, crate::syntax::object::standard_object_nodes));

    insta::assert_debug_snapshot!(
      to_cloze("{{text}}").syntax,
      @r###"
    CLOZE@0..8
      L_CURLY2@0..2 "{{"
      TEXT@2..6 "text"
      R_CURLY@6..7 "}"
      R_CURLY@7..8 "}"
    "###
    );

    insta::assert_debug_snapshot!(
      to_cloze("{{text}@id}").syntax,
      @r###"
    CLOZE@0..11
      L_CURLY2@0..2 "{{"
      TEXT@2..6 "text"
      R_CURLY@6..7 "}"
      AT@7..8 "@"
      TEXT@8..10 "id"
      R_CURLY@10..11 "}"
    "###
    );

    insta::assert_debug_snapshot!(
      to_cloze("{{text}{hint}}").syntax,
      @r###"
    CLOZE@0..14
      L_CURLY2@0..2 "{{"
      TEXT@2..6 "text"
      R_CURLY@6..7 "}"
      L_CURLY@7..8 "{"
      TEXT@8..12 "hint"
      R_CURLY@12..13 "}"
      R_CURLY@13..14 "}"
    "###
    );

    insta::assert_debug_snapshot!(
      to_cloze("{{text}{hint}@id}").syntax,
      @r###"
    CLOZE@0..17
      L_CURLY2@0..2 "{{"
      TEXT@2..6 "text"
      R_CURLY@6..7 "}"
      L_CURLY@7..8 "{"
      TEXT@8..12 "hint"
      R_CURLY@12..13 "}"
      AT@13..14 "@"
      TEXT@14..16 "id"
      R_CURLY@16..17 "}"
    "###
    );

    insta::assert_debug_snapshot!(
      to_cloze("{{$\\frac{a}{b}$}{fractions}}").syntax,
      @r###"
    CLOZE@0..28
      L_CURLY2@0..2 "{{"
      LATEX_FRAGMENT@2..15
        DOLLAR@2..3 "$"
        TEXT@3..14 "\\frac{a}{b}"
        DOLLAR@14..15 "$"
      R_CURLY@15..16 "}"
      L_CURLY@16..17 "{"
      TEXT@17..26 "fractions"
      R_CURLY@26..27 "}"
      R_CURLY@27..28 "}"
    "###
    );

    let config = &ParseConfig::default();

    assert!(
        cloze_node(
            ("{{}}", config).into(),
            crate::syntax::object::standard_object_nodes
        )
        .is_err()
    );
    assert!(
        cloze_node(
            ("{{text}", config).into(),
            crate::syntax::object::standard_object_nodes
        )
        .is_err()
    );
    assert!(
        cloze_node(
            ("{text}}", config).into(),
            crate::syntax::object::standard_object_nodes
        )
        .is_err()
    );
    assert!(
        cloze_node(
            ("{{text}{}", config).into(),
            crate::syntax::object::standard_object_nodes
        )
        .is_err()
    );
    assert!(
        cloze_node(
            ("{{text}a}", config).into(),
            crate::syntax::object::standard_object_nodes
        )
        .is_err()
    );
}
